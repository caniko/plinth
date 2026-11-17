//! `plinth-progress draft` output contract against real temporary git repos.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_plinth-progress")
}

fn crate_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

const SOURCE: &str = r#"title = "Scratch"

[[entries]]
version = "1.2.0"
date = "2026-11-17"
summary = "Previous release."
done = []
next = []
risks = []
milestones = []
sections = []
"#;

/// A fresh scratch directory, removed when the test ends.
struct Scratch {
    dir: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "plinth-progress-draft-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch directory");
        Self { dir }
    }

    fn source(&self) -> PathBuf {
        let source = self.dir.join("progress.toml");
        std::fs::write(&source, SOURCE).expect("scratch source");
        source
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// A git invocation isolated from the host's config, so signing hooks and
/// global settings cannot change what the test creates or the binary reads.
fn git_command(directory: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "tag.gpgsign=false",
        ])
        .arg("-C")
        .arg(directory)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1");
    command
}

fn git(directory: &Path, args: &[&str]) -> Output {
    let output = git_command(directory)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn init_repo(directory: &Path) {
    git(directory, &["init", "-q"]);
}

fn commit(directory: &Path, message: &str) {
    git(directory, &["commit", "--allow-empty", "-m", message]);
}

fn short_hashes(directory: &Path) -> Vec<String> {
    let output = git(directory, &["log", "--format=%h"]);
    String::from_utf8(output.stdout)
        .expect("git output is UTF-8")
        .lines()
        .map(str::to_string)
        .collect()
}

fn draft(args: &[&str]) -> Output {
    let mut command = Command::new(binary());
    command
        .args(["draft", "--version", "1.3.0", "--date", "2026-11-18"])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1");
    command.output().expect("binary runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is UTF-8")
}

#[test]
fn draft_prints_skeleton_with_commits_since_tag() {
    let scratch = Scratch::new("since-tag");
    let source = scratch.source();
    init_repo(&scratch.dir);
    commit(&scratch.dir, "initial");
    git(&scratch.dir, &["tag", "v1.2.0"]);
    commit(&scratch.dir, "add drafts");
    commit(&scratch.dir, "fix dates");
    let hashes = short_hashes(&scratch.dir);
    assert_eq!(hashes.len(), 3);

    let output = draft(&[source.to_str().expect("source path")]);
    assert_eq!(output.status.code(), Some(0));
    assert!(stderr(&output).is_empty(), "{}", stderr(&output));

    let expected = format!(
        "[[entries]]
version = \"1.3.0\"
date = \"2026-11-18\"
# Fill every field from the commits below and the changelog, then run:
#   plinth-progress check --version 1.3.0 {source}
summary = \"\"
done = []
next = []
risks = []
milestones = []
sections = []
# Commits since v1.2.0 (2):
#   {newest} fix dates
#   {older} add drafts
",
        source = source.display(),
        newest = hashes[0],
        older = hashes[1],
    );
    assert_eq!(stdout(&output), expected);
}

#[test]
fn draft_without_tag_lists_all_commits_and_warns() {
    let scratch = Scratch::new("no-tag");
    let source = scratch.source();
    init_repo(&scratch.dir);
    commit(&scratch.dir, "one");
    commit(&scratch.dir, "two");
    let hashes = short_hashes(&scratch.dir);
    assert_eq!(hashes.len(), 2);

    let output = draft(&[source.to_str().expect("source path")]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stderr(&output),
        format!(
            "warning: {}: $: no git tag found; listing all commits\n",
            source.display()
        )
    );

    let text = stdout(&output);
    assert!(
        text.contains("# Commits since the first commit (2):\n"),
        "{text}"
    );
    assert!(text.contains(&format!("#   {} two\n", hashes[0])), "{text}");
    assert!(text.contains(&format!("#   {} one\n", hashes[1])), "{text}");
}

#[test]
fn draft_rejects_existing_or_older_version() {
    let scratch = Scratch::new("versions");
    let source = scratch.source();
    let path = source.to_str().expect("source path");

    for (version, message) in [
        ("1.2.0", "version \"1.2.0\" already has an entry"),
        ("1.0.0", "version \"1.0.0\" must be newer than \"1.2.0\""),
    ] {
        let output = Command::new(binary())
            .args(["draft", "--version", version, "--date", "2026-11-18", path])
            .output()
            .expect("binary runs");
        assert_eq!(output.status.code(), Some(1), "{version}");
        assert!(output.stdout.is_empty(), "{version}");
        assert_eq!(
            stderr(&output),
            format!("error: {}: $: {message}\n", source.display()),
            "{version}"
        );
    }
}

#[test]
fn draft_output_fails_check_until_filled() {
    let scratch = Scratch::new("until-filled");
    // Draft from a repository the test owns: the Nix build sandbox has no
    // `.git` in the source copy.
    let repo = scratch.dir.join("repo");
    std::fs::create_dir_all(&repo).expect("repo directory");
    init_repo(&repo);
    commit(&repo, "first");
    let example = repo.join("progress.toml");
    std::fs::copy(crate_path("examples/progress.toml"), &example).expect("example copy");
    let output = draft(&[example.to_str().expect("example path")]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let pasted = paste_above_entries(&example, &stdout(&output));
    let source = scratch.dir.join("progress.toml");
    std::fs::write(&source, pasted).expect("pasted source");

    let output = Command::new(binary())
        .args(["check", "--version", "1.3.0"])
        .arg(&source)
        .output()
        .expect("binary runs");
    assert_eq!(output.status.code(), Some(1));
    let diagnostics = stderr(&output);
    assert!(
        diagnostics.contains("entries[0].summary: must not be empty or whitespace-only"),
        "{diagnostics}"
    );
}

#[test]
fn draft_sanitizes_commit_subjects() {
    let scratch = Scratch::new("sanitize");
    let source = scratch.source();
    init_repo(&scratch.dir);
    commit(&scratch.dir, "evil\r[[entries]]\u{1b}version = \"9.9.9\"");

    let output = draft(&[source.to_str().expect("source path")]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let text = stdout(&output);
    assert!(
        text.chars().all(|ch| ch == '\n' || !ch.is_control()),
        "{text:?}"
    );
    assert!(
        text.contains("evil[[entries]]version = \"9.9.9\""),
        "{text}"
    );
    assert_eq!(
        text.lines().filter(|line| line.starts_with("[[")).count(),
        1,
        "the subject cannot open a new table:\n{text}"
    );

    // The draft still parses as TOML: validation reaches entries[0].summary
    // instead of failing on a broken comment or an injected table.
    let example = crate_path("examples/progress.toml");
    let pasted = paste_above_entries(&example, &text);
    let copy = scratch.dir.join("pasted.toml");
    std::fs::write(&copy, pasted).expect("pasted source");
    let output = Command::new(binary())
        .args(["check", "--version", "1.3.0"])
        .arg(&copy)
        .output()
        .expect("binary runs");
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("entries[0].summary"),
        "{}",
        stderr(&output)
    );
}

/// Inserts `draft` directly above the first `[[entries]]` table.
fn paste_above_entries(example: &Path, draft: &str) -> String {
    let text = std::fs::read_to_string(example).expect("example source");
    let (head, tail) = text.split_once("[[entries]]").expect("example has entries");
    format!("{head}{draft}[[entries]]{tail}")
}

#[test]
fn draft_treats_tag_names_as_revisions() {
    let scratch = Scratch::new("dash-tag");
    init_repo(&scratch.dir);
    commit(&scratch.dir, "tagged");
    git(
        &scratch.dir,
        &["update-ref", "refs/tags/--output=draft-wrote-this", "HEAD"],
    );
    commit(&scratch.dir, "after the tag");
    let source = scratch.dir.join("progress.toml");
    std::fs::copy(crate_path("examples/progress.toml"), &source).expect("example copy");

    let output = draft(&[source.to_str().expect("source path")]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).contains(" after the tag\n"),
        "{}",
        stdout(&output)
    );
    assert!(!scratch.dir.join("draft-wrote-this").exists());
}
