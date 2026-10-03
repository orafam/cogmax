use std::collections::BTreeMap;

use cogmax_discovery::{AgentKind, DiscoveredSource};

pub fn discovery_report(
    sources: &[DiscoveredSource],
    projects_by_agent: &BTreeMap<AgentKind, usize>,
) -> String {
    if sources.is_empty() {
        return "No agent memory sources found.".into();
    }

    sources
        .iter()
        .map(|source| {
            let projects = projects_by_agent
                .get(&source.agent)
                .copied()
                .unwrap_or_default();
            if projects == 0 {
                format!(
                    "{:?}: {} files at {}",
                    source.agent,
                    source.file_count,
                    source.root.display()
                )
            } else {
                format!(
                    "{:?}: {} files at {} ({} projects)",
                    source.agent,
                    source.file_count,
                    source.root.display(),
                    projects
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn import_preview(
    total: usize,
    scope: &str,
    by_agent: &BTreeMap<String, usize>,
    by_project_kind: &BTreeMap<(String, String), usize>,
) -> String {
    let mut lines = vec![format!(
        "{} memory files would be imported into {}.",
        total, scope
    )];
    lines.extend(
        by_agent
            .iter()
            .map(|(agent, count)| format!("  {agent}: {count} files")),
    );
    lines.push("By project and kind:".into());
    lines.extend(
        by_project_kind
            .iter()
            .map(|((project, kind), count)| format!("  {project} / {kind}: {count}")),
    );
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use super::*;

    #[test]
    fn renders_discovery_without_fabricating_project_counts() {
        let sources = vec![DiscoveredSource {
            agent: AgentKind::Claude,
            root: PathBuf::from("/tmp/.claude/projects"),
            file_count: 3,
        }];
        let report = discovery_report(&sources, &BTreeMap::new());
        assert_eq!(report, "Claude: 3 files at /tmp/.claude/projects");
    }

    #[test]
    fn renders_import_preview_in_deterministic_order() {
        let mut agents = BTreeMap::new();
        agents.insert("Codex".into(), 2);
        let mut projects = BTreeMap::new();
        projects.insert(("rrc".into(), "Decision".into()), 1);
        let report = import_preview(2, "user:local", &agents, &projects);
        assert_eq!(
            report,
            "2 memory files would be imported into user:local.\n  Codex: 2 files\nBy project and kind:\n  rrc / Decision: 1"
        );
    }
}
