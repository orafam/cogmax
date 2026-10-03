use std::{
    env,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use cogmax_api::router;
use cogmax_core::{LearnRequest, MemoryService};
use cogmax_discovery::{discover, AgentKind, JsonSource, MarkdownSource, MemorySource};
use cogmax_domain::{
    memory::{Authority, Confidence},
    scope::MemoryScope,
};
use cogmax_storage::SqliteStore;

mod install;

fn data_path() -> PathBuf {
    env::var_os("COGMAX_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".cogmax"))
        .join("cogmax.sqlite3")
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
        "discover" => discover_command(),
        "import" => import_command(),
        _ => {
            eprintln!("usage: cogmax [serve|install|uninstall|start|stop|restart|status|inspect|discover|import --preview|import --apply|import --rebuild]");
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
    let sources = discover(home);
    if sources.is_empty() {
        println!("No agent memory sources found.");
        return;
    }
    for source in sources {
        let projects = MarkdownSource::new(source.agent, &source.root)
            .scan()
            .map(|items| {
                items
                    .into_iter()
                    .filter_map(|item| item.project_scope)
                    .collect::<std::collections::BTreeSet<_>>()
            })
            .unwrap_or_default();
        println!(
            "{:?}: {} files at {}{}",
            source.agent,
            source.file_count,
            source.root.display(),
            if projects.is_empty() {
                String::new()
            } else {
                format!(" ({} projects)", projects.len())
            }
        );
    }
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
    if !matches!(mode.as_str(), "--preview" | "--apply" | "--rebuild") {
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
                let project = item.project_scope.unwrap_or_else(|| "local".into());
                *by_project_kind
                    .entry((project, format!("{:?}", candidate.kind)))
                    .or_default() += 1;
                if mode == "--apply" || mode == "--rebuild" {
                    let _ = service
                        .learn(LearnRequest {
                            candidate,
                            confidence: Confidence::Medium,
                            authority: Authority::Inferred,
                        })
                        .expect("import failed");
                }
            }
        }
    }
    match mode.as_str() {
        "--preview" => {
            println!(
                "{} memory files would be imported into {}.",
                total,
                scope.as_str()
            );
            for (agent, count) in by_agent {
                println!("  {agent}: {count} files");
            }
            println!("By project and kind:");
            for ((project, kind), count) in by_project_kind {
                println!("  {project} / {kind}: {count}");
            }
        }
        "--apply" => println!("{} memory files processed into {}.", total, scope.as_str()),
        "--rebuild" => println!("{} memory files rebuilt into {}.", total, scope.as_str()),
        _ => unreachable!(),
    }
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
