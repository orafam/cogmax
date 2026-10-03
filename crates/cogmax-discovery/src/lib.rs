use std::{
    fs,
    path::{Path, PathBuf},
};

use cogmax_domain::{candidate::MemoryCandidate, memory::MemoryKind, scope::MemoryScope};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgentKind {
    Codex,
    Claude,
    Kimi,
    Generic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredSource {
    pub agent: AgentKind,
    pub root: PathBuf,
    pub file_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalMemory {
    pub agent: AgentKind,
    pub path: PathBuf,
    pub content: String,
    pub content_sha256: String,
    pub project_scope: Option<String>,
}

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("cannot read source {0}: {1}")]
    Read(PathBuf, std::io::Error),
}

pub trait MemorySource {
    fn agent(&self) -> AgentKind;
    fn detect(&self) -> Option<DiscoveredSource>;
    fn scan(&self) -> Result<Vec<ExternalMemory>, DiscoveryError>;
    fn normalize(&self, item: &ExternalMemory, scope: MemoryScope) -> MemoryCandidate;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedDecision {
    pub decision: String,
    pub rationale: Option<String>,
}

pub struct MarkdownSource {
    agent: AgentKind,
    root: PathBuf,
}

impl MarkdownSource {
    pub fn new(agent: AgentKind, root: impl Into<PathBuf>) -> Self {
        Self {
            agent,
            root: root.into(),
        }
    }
}

impl MemorySource for MarkdownSource {
    fn agent(&self) -> AgentKind {
        self.agent
    }

    fn detect(&self) -> Option<DiscoveredSource> {
        if !self.root.is_dir() {
            return None;
        }
        let file_count = collect_files(&self.root).len();
        Some(DiscoveredSource {
            agent: self.agent,
            root: self.root.clone(),
            file_count,
        })
    }

    fn scan(&self) -> Result<Vec<ExternalMemory>, DiscoveryError> {
        let mut items = Vec::new();
        for path in collect_files(&self.root) {
            let is_markdown = path
                .extension()
                .is_some_and(|extension| extension == "md" || extension == "markdown");
            if !is_markdown {
                continue;
            }
            let content = fs::read_to_string(&path)
                .map_err(|error| DiscoveryError::Read(path.clone(), error))?;
            if looks_sensitive(&content) || content.trim().is_empty() {
                continue;
            }
            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            let project_scope =
                project_scope_from_content(&content).or_else(|| project_scope_from_path(&path));
            items.push(ExternalMemory {
                agent: self.agent,
                path,
                content,
                content_sha256: format!("{:x}", hasher.finalize()),
                project_scope,
            });
        }
        Ok(items)
    }

    fn normalize(&self, item: &ExternalMemory, scope: MemoryScope) -> MemoryCandidate {
        let kind = classify_memory(&item.content);
        let content = if kind == MemoryKind::Decision {
            let decision = extract_decision(&item.content)
                .or_else(|| extract_codex_outcome(&item.content))
                .expect("decision classification must have an extractable decision");
            format_decision(&decision)
        } else {
            item.content.trim().to_owned()
        };
        MemoryCandidate::new(item.content_sha256.clone(), scope, kind, content)
    }
}

pub struct JsonSource {
    agent: AgentKind,
    root: PathBuf,
}

impl JsonSource {
    pub fn new(agent: AgentKind, root: impl Into<PathBuf>) -> Self {
        Self {
            agent,
            root: root.into(),
        }
    }
}

impl MemorySource for JsonSource {
    fn agent(&self) -> AgentKind {
        self.agent
    }

    fn detect(&self) -> Option<DiscoveredSource> {
        self.root.is_dir().then(|| DiscoveredSource {
            agent: self.agent,
            root: self.root.clone(),
            file_count: collect_files(&self.root)
                .iter()
                .filter(|path| path.extension().is_some_and(|e| e == "json"))
                .count(),
        })
    }

    fn scan(&self) -> Result<Vec<ExternalMemory>, DiscoveryError> {
        let mut items = Vec::new();
        for path in collect_files(&self.root) {
            if !path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                continue;
            }
            let raw = fs::read_to_string(&path)
                .map_err(|error| DiscoveryError::Read(path.clone(), error))?;
            let value: serde_json::Value = match serde_json::from_str(&raw) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let content = value
                .get("content")
                .or_else(|| value.get("text"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if content.trim().is_empty() || looks_sensitive(content) || looks_sensitive(&raw) {
                continue;
            }
            let mut hasher = Sha256::new();
            hasher.update(raw.as_bytes());
            let project_scope = value
                .get("project")
                .or_else(|| value.get("cwd"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .or_else(|| project_scope_from_path(&path));
            items.push(ExternalMemory {
                agent: self.agent,
                path,
                content: content.to_owned(),
                content_sha256: format!("{:x}", hasher.finalize()),
                project_scope,
            });
        }
        Ok(items)
    }

    fn normalize(&self, item: &ExternalMemory, scope: MemoryScope) -> MemoryCandidate {
        MarkdownSource {
            agent: self.agent,
            root: self.root.clone(),
        }
        .normalize(item, scope)
    }
}

pub fn discover(home: impl AsRef<Path>) -> Vec<DiscoveredSource> {
    let home = home.as_ref();
    let roots = [
        (AgentKind::Codex, home.join(".codex/memories")),
        (AgentKind::Claude, home.join(".claude/projects")),
        (AgentKind::Kimi, home.join(".kimi-code/memory")),
        (AgentKind::Generic, home.join(".agentmemory")),
    ];
    let mut sources = roots
        .iter()
        .filter_map(|(agent, root)| MarkdownSource::new(*agent, root).detect())
        .collect::<Vec<_>>();
    sources.extend(roots.into_iter().filter_map(|(agent, root)| {
        JsonSource::new(agent, root)
            .detect()
            .filter(|source| source.file_count > 0)
    }));
    sources
}

fn collect_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_files(&path));
        } else if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn looks_sensitive(content: &str) -> bool {
    let lower = content.to_lowercase();
    [
        "private_key",
        "-----begin",
        "authorization: bearer",
        "aws_secret_access_key",
        "client_secret",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn project_scope_from_content(content: &str) -> Option<String> {
    for line in content.lines().take(80) {
        let value = line
            .split_once("cwd=")
            .or_else(|| line.split_once("applies_to:"))
            .map(|(_, value)| {
                value
                    .trim()
                    .trim_matches('`')
                    .trim_end_matches(',')
                    .to_owned()
            });
        if let Some(value) = value.filter(|value| !value.is_empty() && value != "unknown") {
            return Some(value);
        }
    }
    None
}

fn project_scope_from_path(path: &Path) -> Option<String> {
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| {
            let value = name.to_string_lossy().to_string();
            (!value.is_empty() && value != "memories" && value != "memory").then_some(value)
        })
}

pub fn extract_decision(content: &str) -> Option<ExtractedDecision> {
    let lines: Vec<&str> = content.lines().collect();
    let decision_index = lines.iter().position(|line| {
        let normalized = line.trim().to_lowercase();
        normalized.starts_with("# decisão")
            || normalized.starts_with("## decisão")
            || normalized.starts_with("# decision")
            || normalized.starts_with("## decision")
            || normalized.starts_with("decision:")
            || normalized.starts_with("decisão:")
    })?;
    let decision = lines[decision_index]
        .split_once(':')
        .map(|(_, value)| value.trim().to_owned())
        .or_else(|| {
            lines
                .get(decision_index + 1)
                .map(|line| line.trim().to_owned())
        })
        .filter(|value| !value.is_empty())?;
    let rationale = lines.iter().skip(decision_index + 1).find_map(|line| {
        let normalized = line.trim().to_lowercase();
        if normalized.starts_with("motivo:")
            || normalized.starts_with("rationale:")
            || normalized.starts_with("why:")
        {
            line.split_once(':')
                .map(|(_, value)| value.trim().to_owned())
                .filter(|value| !value.is_empty())
        } else {
            None
        }
    });
    Some(ExtractedDecision {
        decision,
        rationale,
    })
}

fn classify_memory(content: &str) -> MemoryKind {
    if is_correction(content) {
        return MemoryKind::Correction;
    }
    if extract_decision(content).is_some() || extract_codex_outcome(content).is_some() {
        return MemoryKind::Decision;
    }
    let normalized = content
        .lines()
        .take(12)
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    if normalized.contains("# projeto")
        || normalized.contains("## projeto")
        || normalized.contains("project:")
    {
        MemoryKind::Project
    } else if normalized.contains("# procedimento")
        || normalized.contains("## procedimento")
        || normalized.contains("runbook")
        || normalized.contains("how to")
    {
        MemoryKind::Procedure
    } else if normalized.contains("# fato")
        || normalized.contains("## fato")
        || normalized.contains("fact:")
    {
        MemoryKind::Fact
    } else {
        MemoryKind::Reference
    }
}

fn is_correction(content: &str) -> bool {
    content.lines().take(12).any(|line| {
        let normalized = line.trim().to_lowercase();
        normalized.starts_with("correção:")
            || normalized.starts_with("correction:")
            || normalized.starts_with("# correção")
            || normalized.starts_with("## correção")
            || normalized.starts_with("# correction")
            || normalized.starts_with("## correction")
    })
}

fn extract_codex_outcome(content: &str) -> Option<ExtractedDecision> {
    let lines: Vec<&str> = content.lines().collect();
    let title = lines.first()?.strip_prefix("# ")?.trim();
    if title.is_empty() || title.starts_with("What's in") || title.starts_with("Task ") {
        return None;
    }
    let knowledge_index = lines
        .iter()
        .position(|line| line.trim().eq_ignore_ascii_case("## Reusable knowledge"))?;
    let context = lines
        .iter()
        .skip(knowledge_index + 1)
        .take_while(|line| !line.trim_start().starts_with('#'))
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let rationale =
        (!context.is_empty()).then_some(format!("Evidência/contexto importado: {context}"));
    Some(ExtractedDecision {
        decision: title.to_owned(),
        rationale,
    })
}

fn format_decision(decision: &ExtractedDecision) -> String {
    match &decision.rationale {
        Some(rationale) => format!("Decisão: {}\nMotivo: {}", decision.decision, rationale),
        None => format!(
            "Decisão: {}\nMotivo: não identificado na fonte importada",
            decision.decision
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detects_sources_without_reading_content() {
        let home = tempdir().unwrap();
        fs::create_dir_all(home.path().join(".claude/projects")).unwrap();
        let found = discover(home.path());
        assert_eq!(
            found,
            vec![DiscoveredSource {
                agent: AgentKind::Claude,
                root: home.path().join(".claude/projects"),
                file_count: 0
            }]
        );
    }

    #[test]
    fn scans_markdown_and_skips_sensitive_content() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("memory.md"), "Prefere respostas curtas").unwrap();
        fs::write(root.path().join("secret.md"), "client_secret=do-not-import").unwrap();
        let source = MarkdownSource::new(AgentKind::Codex, root.path());
        let items = source.scan().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].agent, AgentKind::Codex);
        assert!(!items[0].content_sha256.is_empty());
        assert!(items[0].project_scope.is_some());
    }

    #[test]
    fn normalization_uses_content_hash_as_event_id() {
        let root = tempdir().unwrap();
        let path = root.path().join("memory.md");
        fs::write(&path, "Prefere respostas curtas").unwrap();
        let source = MarkdownSource::new(AgentKind::Claude, root.path());
        let item = source.scan().unwrap().remove(0);
        let candidate = source.normalize(&item, MemoryScope::new("user:alice").unwrap());
        assert_eq!(candidate.source_event_id, item.content_sha256);
        assert_eq!(candidate.kind, MemoryKind::Reference);
    }

    #[test]
    fn extracts_decision_and_rationale_without_inventing_missing_reason() {
        let extracted =
            extract_decision("# Decisão: usar offline-first\nMotivo: testar sem depender da nuvem")
                .unwrap();
        assert_eq!(extracted.decision, "usar offline-first");
        assert_eq!(
            extracted.rationale.as_deref(),
            Some("testar sem depender da nuvem")
        );

        let missing = extract_decision("Decision: usar SQLite").unwrap();
        assert_eq!(missing.rationale, None);
    }

    #[test]
    fn normalizes_decision_as_decision_kind() {
        let root = tempdir().unwrap();
        fs::write(
            root.path().join("decision.md"),
            "Decisão: usar Cogmax\nMotivo: skill-first",
        )
        .unwrap();
        let source = MarkdownSource::new(AgentKind::Codex, root.path());
        let item = source.scan().unwrap().remove(0);
        let candidate = source.normalize(&item, MemoryScope::new("user:alice").unwrap());
        assert_eq!(candidate.kind, MemoryKind::Decision);
        assert!(candidate.content.contains("Motivo: skill-first"));
    }

    #[test]
    fn extracts_codex_outcome_with_explicit_context_label() {
        let extracted = extract_codex_outcome(
            "# Escolhemos o runtime offline-first\n\n## Reusable knowledge\n- A validação local não depende de rede.\n\n## Failures and how to do differently\n- nada",
        )
        .unwrap();
        assert_eq!(extracted.decision, "Escolhemos o runtime offline-first");
        assert!(extracted
            .rationale
            .unwrap()
            .contains("Evidência/contexto importado"));
    }

    #[test]
    fn scans_json_memory_with_project_scope_and_skips_sensitive_payloads() {
        let root = tempdir().unwrap();
        fs::write(
            root.path().join("memory.json"),
            r##"{"content":"# Projeto: Susu\nEscopo local","cwd":"/work/susu"}"##,
        )
        .unwrap();
        fs::write(
            root.path().join("secret.json"),
            r#"{"content":"client_secret=do-not-import"}"#,
        )
        .unwrap();
        let source = JsonSource::new(AgentKind::Claude, root.path());
        let items = source.scan().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].project_scope.as_deref(), Some("/work/susu"));
        assert_eq!(items[0].content, "# Projeto: Susu\nEscopo local");
    }

    #[test]
    fn classifies_project_and_procedure_without_overriding_decisions() {
        assert_eq!(
            classify_memory("# Projeto: Cogmax\nEscopo do produto"),
            MemoryKind::Project
        );
        assert_eq!(
            classify_memory("## Procedimento\n1. Execute os testes"),
            MemoryKind::Procedure
        );
        assert_eq!(
            classify_memory("Decisão: usar DuckDB\nMotivo: portabilidade"),
            MemoryKind::Decision
        );
    }

    #[test]
    fn classifies_corrections_and_leaves_ambiguous_notes_as_references() {
        assert_eq!(
            classify_memory("Correção: o serviço deve continuar offline-first"),
            MemoryKind::Correction
        );
        assert_eq!(
            classify_memory("Uma anotação sobre o serviço e suas possibilidades"),
            MemoryKind::Reference
        );
    }

    #[test]
    fn correction_takes_precedence_over_decision_language() {
        assert_eq!(
            classify_memory("Correção: a decisão anterior sobre o banco foi revogada"),
            MemoryKind::Correction
        );
    }
}
