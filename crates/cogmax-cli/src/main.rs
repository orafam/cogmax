use sha2::{Digest, Sha256};
use std::{
    env,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use cogmax_api::router;
use cogmax_core::{LearnRequest, MemoryService};
use cogmax_discovery::{
    discover, inferred_confidence, AgentKind, JsonSource, MarkdownSource, MemorySource,
};
use cogmax_domain::{
    memory::{Authority, Confidence},
    scope::MemoryScope,
};
use cogmax_export::git::sync_snapshot;
use cogmax_export::{manifest, markdown, parquet, verify_manifest, SnapshotManifest};
use cogmax_storage::SqliteStore;

mod install;
mod onboarding;

fn data_path() -> PathBuf {
    data_dir().join("cogmax.sqlite3")
}

fn data_dir() -> PathBuf {
    env::var_os("COGMAX_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".cogmax"))
}

#[tokio::main]
async fn main() {
    let command = env::args().nth(1).unwrap_or_else(|| "serve".into());
    match command.as_str() {
        "serve" => serve().await,
        "install" => {
            if let Err(error) = install::install() {
                exit_error(error);
            }
        }
        "uninstall" => {
            if let Err(error) = install::uninstall() {
                exit_error(error);
            }
        }
        "start" | "stop" | "restart" | "status" => {
            if let Err(error) = install::lifecycle(&command) {
                exit_error(error);
            }
        }
        "inspect" => println!("Cogmax data: {}", data_path().display()),
        "snapshot" => snapshot_command(),
        "discover" => discover_command(),
        "import" => import_command(),
        "onboard" => onboarding_command(),
        "export" => export_command(),
        "restore" => restore_command(),
        "sync" => sync_command(),
        _ => {
            eprintln!("usage: cogmax [serve|install|uninstall|start|stop|restart|status|inspect|discover|import --preview|import --apply|import --rebuild|export <dir>|restore <dir>|snapshot inspect <dir>|sync git <repo> <snapshot>]");
            std::process::exit(2);
        }
    }
}

fn exit_error(error: String) -> ! {
    eprintln!("Cogmax error: {error}");
    std::process::exit(1);
}

fn discover_command() {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let sources = discover(&home);
    let mut projects_by_agent = std::collections::BTreeMap::new();
    for source in &sources {
        let projects = MarkdownSource::new(source.agent, &source.root)
            .scan()
            .map(|items| {
                items
                    .into_iter()
                    .filter_map(|item| item.project_scope)
                    .collect::<std::collections::BTreeSet<_>>()
            })
            .unwrap_or_default();
        projects_by_agent.insert(source.agent, projects.len());
    }
    println!(
        "{}",
        onboarding::discovery_report(&sources, &projects_by_agent)
    );
}

fn source_pairs(home: &Path) -> Vec<(Box<dyn MemorySource>, Box<dyn MemorySource>)> {
    [
        (AgentKind::Codex, home.join(".codex/memories")),
        (AgentKind::Claude, home.join(".claude/projects")),
        (AgentKind::Kimi, home.join(".kimi-code/memory")),
        (AgentKind::Generic, home.join(".agentmemory")),
    ]
    .into_iter()
    .map(|(agent, root)| {
        (
            Box::new(MarkdownSource::new(agent, root.clone())) as Box<dyn MemorySource>,
            Box::new(JsonSource::new(agent, root)) as Box<dyn MemorySource>,
        )
    })
    .collect()
}

fn import_command() {
    let mode = env::args().nth(2).unwrap_or_else(|| "--preview".into());
    import_mode(&mode);
}

fn import_mode(mode: &str) {
    if !matches!(mode, "--preview" | "--apply" | "--rebuild") {
        eprintln!("usage: cogmax import [--preview|--apply|--rebuild]");
        std::process::exit(2);
    }
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let scope =
        MemoryScope::new(env::var("COGMAX_IMPORT_SCOPE").unwrap_or_else(|_| "user:local".into()))
            .expect("invalid COGMAX_IMPORT_SCOPE");
    let store = Arc::new(Mutex::new(
        SqliteStore::open_path(data_path()).expect("cannot open Cogmax data"),
    ));
    if mode == "--rebuild" {
        let removed = store
            .lock()
            .expect("cannot lock Cogmax data")
            .reset_inferred_imports()
            .expect("cannot reset inferred imports");
        println!("Rebuild: {removed} inferred memories removed.");
    }
    let service = MemoryService::new(store);
    let mut total = 0usize;
    let mut skipped_low_confidence = 0usize;
    let mut by_agent = std::collections::BTreeMap::<String, usize>::new();
    let mut by_project_kind = std::collections::BTreeMap::<(String, String), usize>::new();
    for (markdown, json) in source_pairs(&home) {
        for source in [markdown, json] {
            let Ok(items) = source.scan() else { continue };
            for item in items {
                total += 1;
                *by_agent.entry(format!("{:?}", item.agent)).or_default() += 1;
                let item_scope = item
                    .project_scope
                    .as_ref()
                    .map(|project| {
                        MemoryScope::new(format!("{}/project:{}", scope.as_str(), project))
                            .expect("invalid project scope")
                    })
                    .unwrap_or_else(|| scope.clone());
                let candidate = source.normalize(&item, item_scope);
                let confidence = inferred_confidence(candidate.kind, &candidate.content);
                if confidence == Confidence::Low {
                    skipped_low_confidence += 1;
                }
                let project = item.project_scope.unwrap_or_else(|| "local".into());
                *by_project_kind
                    .entry((project, format!("{:?}", candidate.kind)))
                    .or_default() += 1;
                if mode == "--apply" || mode == "--rebuild" {
                    let _ = service
                        .learn(LearnRequest {
                            candidate,
                            confidence,
                            authority: Authority::Inferred,
                        })
                        .expect("import failed");
                }
            }
        }
    }
    match mode {
        "--preview" => {
            println!(
                "{}",
                onboarding::import_preview(total, scope.as_str(), &by_agent, &by_project_kind)
            );
            println!("  {skipped_low_confidence} files skipped: low confidence candidates");
        }
        "--apply" => println!("{} memory files processed into {}.", total, scope.as_str()),
        "--rebuild" => println!("{} memory files rebuilt into {}.", total, scope.as_str()),
        _ => unreachable!(),
    }
}

fn onboarding_command() {
    let mode = env::args().nth(2).unwrap_or_else(|| "--summary".into());
    let marker = data_dir().join("onboarding.json");
    match mode.as_str() {
        "--summary" => {
            let initialized = marker.exists();
            let source_count = discover(
                env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(".")),
            )
            .len();
            let confirmation_digest = onboarding_digest();
            println!(
                "{{\"initialized\":{},\"sources\":{},\"confirmation_digest\":\"{}\"}}",
                initialized, source_count, confirmation_digest
            );
        }
        "--apply" => {
            if marker.exists() {
                println!("{{\"already_initialized\":true}}");
                return;
            }
            let expected = env::args()
                .position(|arg| arg == "--confirm")
                .and_then(|index| env::args().nth(index + 1));
            if expected.as_deref() != Some(onboarding_digest().as_str()) {
                eprintln!("onboarding confirmation is missing or stale");
                std::process::exit(1);
            }
            std::fs::create_dir_all(data_dir()).expect("cannot create Cogmax data directory");
            import_mode("--apply");
            std::fs::write(&marker, b"{\"version\":1}\n").expect("cannot write onboarding marker");
            println!("{{\"initialized\":true}}");
        }
        _ => {
            eprintln!("usage: cogmax onboard [--summary|--apply]");
            std::process::exit(2);
        }
    }
}

fn onboarding_digest() -> String {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut entries = Vec::new();
    for (markdown, json) in source_pairs(&home) {
        for source in [markdown, json] {
            if let Ok(items) = source.scan() {
                for item in items {
                    entries.push(format!(
                        "{:?}|{}|{}",
                        item.agent,
                        item.path.display(),
                        item.content_sha256
                    ));
                }
            }
        }
    }
    entries.sort();
    let mut hasher = Sha256::new();
    hasher.update(entries.join("\n").as_bytes());
    format!("{:x}", hasher.finalize())
}

fn export_command() {
    let directory = env::args()
        .nth(2)
        .unwrap_or_else(|| "cogmax-snapshot".into());
    let directory = PathBuf::from(directory);
    std::fs::create_dir_all(&directory).expect("cannot create snapshot directory");
    let store = SqliteStore::open_path(data_path()).expect("cannot open Cogmax data");
    let memories = store.list_all().expect("cannot read Cogmax memories");
    let markdown_text = markdown(&memories);
    let manifest = manifest(&markdown_text, memories.len());
    std::fs::write(directory.join("memories.md"), &markdown_text)
        .expect("cannot write Markdown snapshot");
    std::fs::write(
        directory.join("memories.json"),
        serde_json::to_vec_pretty(&memories).expect("cannot encode JSON snapshot"),
    )
    .expect("cannot write JSON snapshot");
    std::fs::write(
        directory.join("memories.parquet"),
        parquet(&memories).expect("cannot encode Parquet snapshot"),
    )
    .expect("cannot write Parquet snapshot");
    std::fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("cannot encode snapshot manifest"),
    )
    .expect("cannot write snapshot manifest");
    println!(
        "Exported {} memories to {}.",
        memories.len(),
        directory.display()
    );
}

fn restore_command() {
    let directory = PathBuf::from(
        env::args()
            .nth(2)
            .unwrap_or_else(|| "cogmax-snapshot".into()),
    );
    let markdown_text = std::fs::read_to_string(directory.join("memories.md"))
        .expect("cannot read Markdown snapshot");
    let manifest: SnapshotManifest = serde_json::from_slice(
        &std::fs::read(directory.join("manifest.json")).expect("cannot read snapshot manifest"),
    )
    .expect("invalid snapshot manifest");
    verify_manifest(&markdown_text, &manifest).expect("snapshot manifest verification failed");
    let memories: Vec<cogmax_domain::memory::Memory> = serde_json::from_slice(
        &std::fs::read(directory.join("memories.json")).expect("cannot read JSON snapshot"),
    )
    .expect("invalid JSON snapshot");
    if memories.len() != manifest.memory_count {
        panic!("snapshot memory count does not match manifest");
    }
    let mut store = SqliteStore::open_path(data_path()).expect("cannot open Cogmax data");
    store
        .insert_memories_transactional(&memories)
        .expect("cannot restore memory");
    println!(
        "Restored {} memories from {}.",
        memories.len(),
        directory.display()
    );
}

fn snapshot_command() {
    if env::args().nth(2).as_deref() != Some("inspect") {
        eprintln!("usage: cogmax snapshot inspect <dir>");
        std::process::exit(2);
    }
    let directory = PathBuf::from(
        env::args()
            .nth(3)
            .unwrap_or_else(|| "cogmax-snapshot".into()),
    );
    let manifest: SnapshotManifest = serde_json::from_slice(
        &std::fs::read(directory.join("manifest.json")).expect("cannot read snapshot manifest"),
    )
    .expect("invalid snapshot manifest");
    let markdown_text = std::fs::read_to_string(directory.join("memories.md"))
        .expect("cannot read Markdown snapshot");
    verify_manifest(&markdown_text, &manifest).expect("snapshot manifest verification failed");
    println!("Snapshot: {}", directory.display());
    println!("Schema: {}", manifest.schema_version);
    println!("Memories: {}", manifest.memory_count);
    println!("SHA-256: {}", manifest.sha256);
    println!("Status: verified");
}

fn sync_command() {
    if env::args().nth(2).as_deref() != Some("git") {
        eprintln!("usage: cogmax sync git <repo> <snapshot>");
        std::process::exit(2);
    }
    let repository = env::args().nth(3).unwrap_or_else(|| ".".into());
    let snapshot = env::args()
        .nth(4)
        .unwrap_or_else(|| "cogmax-snapshot".into());
    sync_snapshot(
        repository,
        snapshot,
        ".cogmax/snapshot",
        "chore: sync Cogmax memory snapshot",
    )
    .unwrap_or_else(|error| exit_error(error.to_string()));
    println!("Snapshot synchronized into Git. No push was performed.");
}

async fn serve() {
    let store = Arc::new(Mutex::new(
        SqliteStore::open_path(data_path()).expect("cannot open Cogmax data"),
    ));
    let app = router(Arc::new(MemoryService::new(store)));
    let address: SocketAddr = env::var("COGMAX_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8787".into())
        .parse()
        .expect("invalid COGMAX_ADDR");
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("cannot bind Cogmax");
    println!("Cogmax listening on http://{address}");
    axum::serve(listener, app)
        .await
        .expect("Cogmax server failed");
}
