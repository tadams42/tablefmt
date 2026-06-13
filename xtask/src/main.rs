use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("update-changelog") => {
            update_changelog();
        }
        _ => {
            eprintln!("Available tasks:");
            eprintln!("  update-changelog  Regenerate ## unreleased changelog section");
        }
    }
}

const NOISE_PREFIXES: &[&str] = &[
    "build:", "ci:", "docs:", "chore:", "refact:", "refactor", "test",
];

fn update_changelog() {
    let tag_output = std::process::Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .current_dir(project_root())
        .output()
        .expect("failed to run git describe");

    // When no tags exist yet, fall back to the very first commit so that all
    // commits since the repo was created are included in the changelog.
    let tag = if tag_output.status.success() {
        String::from_utf8(tag_output.stdout)
            .expect("git describe output is not UTF-8")
            .trim()
            .to_string()
    } else {
        let first = std::process::Command::new("git")
            .args(["rev-list", "--max-parents=0", "HEAD"])
            .current_dir(project_root())
            .output()
            .expect("failed to run git rev-list");
        if !first.status.success() {
            eprintln!("git rev-list failed — empty repository?");
            std::process::exit(1);
        }
        String::from_utf8(first.stdout)
            .expect("git rev-list output is not UTF-8")
            .trim()
            .to_string()
    };

    let log_output = std::process::Command::new("git")
        .args(["log", &format!("{}..HEAD", tag), "--format=%s"])
        .current_dir(project_root())
        .output()
        .expect("failed to run git log");

    if !log_output.status.success() {
        eprintln!("git log failed");
        std::process::exit(1);
    }

    let log_text = String::from_utf8(log_output.stdout).expect("git log output is not UTF-8");

    let bullet_lines: Vec<String> = log_text
        .lines()
        .filter(|l| !l.is_empty())
        .filter(|l| !NOISE_PREFIXES.iter().any(|p| l.starts_with(p)))
        .map(|l| format!("- {}", l))
        .collect();

    let changelog_path = project_root().join("CHANGELOG.md");
    let changelog = fs::read_to_string(&changelog_path).expect("failed to read CHANGELOG.md");
    let lines: Vec<&str> = changelog.lines().collect();

    let idx_start = lines
        .iter()
        .position(|l| *l == "## unreleased")
        .expect("'## unreleased' heading not found in CHANGELOG.md");

    let idx_end = lines[idx_start + 1..]
        .iter()
        .position(|l| l.starts_with("## "))
        .map(|rel| idx_start + 1 + rel)
        .unwrap_or(lines.len());

    let mut new_lines: Vec<String> = lines[..=idx_start].iter().map(|l| l.to_string()).collect();
    new_lines.push(String::new());
    new_lines.extend(bullet_lines);
    new_lines.push(String::new());
    new_lines.extend(lines[idx_end..].iter().map(|l| l.to_string()));

    fs::write(&changelog_path, new_lines.join("\n") + "\n").expect("failed to write CHANGELOG.md");
    println!("CHANGELOG.md updated (commits since {}).", tag);
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}
