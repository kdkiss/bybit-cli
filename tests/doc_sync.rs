use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use bybit_cli::command_inventory::{leaf_commands, top_level_commands};

#[test]
fn readme_top_level_table_matches_cli() {
    let expected: BTreeSet<String> = top_level_commands().into_iter().collect();
    let documented = extract_top_level_commands(&readme_text());

    assert_eq!(
        documented, expected,
        "README top-level command table drifted from the clap surface"
    );
}

#[test]
fn readme_documents_every_live_command_path() {
    let inventory = leaf_commands();
    let inventory_set: BTreeSet<String> = inventory.iter().cloned().collect();
    let prefixes = inventory_prefixes(&inventory);
    let extracted = extract_documented_commands(&readme_text(), &inventory_set, &prefixes);
    let documented = extracted.documented;

    let missing: Vec<String> = inventory_set.difference(&documented).cloned().collect();
    let extra: Vec<String> = documented.difference(&inventory_set).cloned().collect();

    assert!(
        extracted.unresolved.is_empty(),
        "README references unresolved commands: {:?}",
        extracted.unresolved
    );
    assert!(
        missing.is_empty(),
        "README is missing implemented commands: {missing:?}"
    );
    assert!(
        extra.is_empty(),
        "README references commands that do not exist in clap: {extra:?}"
    );
}

#[test]
fn agent_docs_only_reference_live_command_paths() {
    let inventory = leaf_commands();
    let inventory_set: BTreeSet<String> = inventory.iter().cloned().collect();
    let prefixes = inventory_prefixes(&inventory);

    for relative_path in [
        "AGENTS.md",
        "CLAUDE.md",
        "CONTEXT.md",
        "llms.txt",
        "agents/README.md",
    ] {
        let extracted =
            extract_documented_commands(&doc_text(relative_path), &inventory_set, &prefixes);
        let extra: Vec<String> = extracted
            .documented
            .difference(&inventory_set)
            .cloned()
            .collect();

        assert!(
            extracted.unresolved.is_empty(),
            "{relative_path} references unresolved commands: {:?}",
            extracted.unresolved
        );
        assert!(
            extra.is_empty(),
            "{relative_path} references commands that do not exist in clap: {extra:?}"
        );
    }
}

fn readme_text() -> String {
    doc_text("README.md")
}

fn doc_text(relative_path: &str) -> String {
    fs::read_to_string(repo_root().join(relative_path))
        .unwrap_or_else(|_| panic!("failed to read {relative_path}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn extract_top_level_commands(readme: &str) -> BTreeSet<String> {
    let mut in_section = false;
    let mut commands = BTreeSet::new();

    for line in readme.lines() {
        let trimmed = line.trim();
        if trimmed == "## Commands" {
            in_section = true;
            continue;
        }
        if in_section && trimmed == "<details>" {
            break;
        }
        if !in_section || !trimmed.starts_with('|') {
            continue;
        }

        let columns: Vec<&str> = trimmed.split('|').map(str::trim).collect();
        let Some(name) = columns.get(1).copied() else {
            continue;
        };
        if name.is_empty()
            || name.eq_ignore_ascii_case("command")
            || name.eq_ignore_ascii_case("group")
            || name.starts_with('-')
        {
            continue;
        }

        commands.insert(name.trim_matches('`').to_string());
    }

    commands
}

struct ExtractedCommands {
    documented: BTreeSet<String>,
    unresolved: BTreeSet<String>,
}

fn extract_documented_commands(
    readme: &str,
    inventory_commands: &BTreeSet<String>,
    inventory_prefixes: &BTreeSet<String>,
) -> ExtractedCommands {
    let mut commands = BTreeSet::new();
    let mut unresolved = BTreeSet::new();

    for line in readme.lines() {
        let mut search_from = 0;
        while let Some(offset) = line[search_from..].find("bybit ") {
            let start = search_from + offset + "bybit ".len();
            let candidate = normalize_documented_command(
                &line[start..],
                inventory_commands,
                inventory_prefixes,
            );
            commands.extend(candidate.documented);
            unresolved.extend(candidate.unresolved);
            search_from = start;
        }
    }

    ExtractedCommands {
        documented: commands,
        unresolved,
    }
}

fn normalize_documented_command(
    text: &str,
    inventory_commands: &BTreeSet<String>,
    inventory_prefixes: &BTreeSet<String>,
) -> ExtractedCommands {
    let candidate = text.split('#').next().unwrap_or(text).trim();
    let mut path_tokens = Vec::new();
    let mut last_match = None;
    let mut unresolved = BTreeSet::new();

    for raw_token in candidate.split_whitespace() {
        let token = raw_token.trim_matches(|c: char| matches!(c, '`' | '|' | ',' | ';' | '.'));
        if token.is_empty() || token.starts_with('-') {
            break;
        }
        if token.starts_with('<')
            || token.starts_with('[')
            || token.starts_with('{')
            || token.starts_with('(')
            || token.starts_with(')')
            || token.contains("...")
        {
            break;
        }

        let next = if path_tokens.is_empty() {
            token.to_string()
        } else {
            format!("{} {}", path_tokens.join(" "), token)
        };

        if !inventory_prefixes.contains(&next) {
            if last_match.is_none()
                && (!path_tokens.is_empty()
                    || token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
            {
                unresolved.insert(next);
            }
            break;
        }

        path_tokens.push(token.to_string());
        if inventory_commands.contains(&next) {
            last_match = Some(next);
        }
        if raw_token.ends_with('`')
            || raw_token.ends_with('.')
            || raw_token.ends_with(')')
            || raw_token.ends_with(':')
        {
            break;
        }
    }

    let mut documented = BTreeSet::new();
    if let Some(command) = last_match {
        documented.insert(command);
    }

    ExtractedCommands {
        documented,
        unresolved,
    }
}

fn inventory_prefixes(commands: &[String]) -> BTreeSet<String> {
    let mut prefixes = BTreeSet::new();
    for command in commands {
        let parts: Vec<&str> = command.split_whitespace().collect();
        for end in 1..=parts.len() {
            prefixes.insert(parts[..end].join(" "));
        }
    }
    prefixes
}
