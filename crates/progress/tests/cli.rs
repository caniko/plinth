//! CLI exit codes and output contract.

use std::path::Path;
use std::process::Command;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_plinth-progress")
}

fn crate_path(relative: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(relative)
        .display()
        .to_string()
}

#[test]
fn check_cli_exit_codes() {
    let example = crate_path("examples/progress.toml");

    let output = Command::new(binary())
        .args(["check", "--version", "1.1.0", example.as_str()])
        .output()
        .expect("binary runs");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty(), "check prints nothing on success");
    assert!(output.stderr.is_empty(), "check prints nothing on success");

    let output = Command::new(binary())
        .args(["check", "--version", "9.9.9", example.as_str()])
        .output()
        .expect("binary runs");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert_eq!(
        stderr.trim_end(),
        format!("error: {example}:4:1: entries: version \"9.9.9\" not found")
    );

    // Usage errors exit 2 without reaching validation.
    for args in [
        vec![],
        vec!["check"],
        vec!["check", "--version"],
        vec!["unknown-subcommand"],
    ] {
        let output = Command::new(binary())
            .args(&args)
            .output()
            .expect("binary runs");
        assert_eq!(output.status.code(), Some(2), "args: {args:?}");
    }

    // Help and program version are successful and do not need a source.
    for args in [vec!["--help"], vec!["--version"]] {
        let output = Command::new(binary())
            .args(&args)
            .output()
            .expect("binary runs");
        assert_eq!(output.status.code(), Some(0), "args: {args:?}");
        assert!(!output.stdout.is_empty(), "args: {args:?}");
    }

    // I/O failures use the position-less diagnostic form.
    let absent = crate_path("tests/fixtures/does-not-exist.toml");
    let output = Command::new(binary())
        .args(["check", "--version", "1.0.0", absent.as_str()])
        .output()
        .expect("binary runs");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(
        stderr.starts_with(&format!("error: {absent}: $: ")),
        "{stderr}"
    );

    // Validation failures keep the whole document before the guard.
    for fixture in [
        "tests/fixtures/schema/bad_status.toml",
        "tests/fixtures/schema/missing_field.toml",
        "tests/fixtures/schema/empty_string.toml",
        "tests/fixtures/semantic/equal_precedence_build.toml",
    ] {
        let path = crate_path(fixture);
        let output = Command::new(binary())
            .args(["check", "--version", "1.0.0", path.as_str()])
            .output()
            .expect("binary runs");
        assert_eq!(output.status.code(), Some(1), "{fixture}");
        let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
        assert!(
            stderr.starts_with(&format!("error: {path}:")),
            "{fixture}: {stderr}"
        );
    }
}

#[test]
fn build_does_not_clobber_on_error() {
    let dir = std::env::temp_dir().join(format!("plinth-progress-build-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    let out = dir.join("index.html");
    std::fs::write(&out, "previous page").expect("sentinel output");

    let invalid = crate_path("tests/fixtures/schema/missing_field.toml");
    let output = Command::new(binary())
        .args(["build", invalid.as_str(), "--out", out.to_str().unwrap()])
        .output()
        .expect("binary runs");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        std::fs::read_to_string(&out).expect("output is readable"),
        "previous page",
        "a failed build leaves --out untouched"
    );
    assert_eq!(
        std::fs::read_dir(&dir).expect("scratch directory").count(),
        1,
        "no temporary sibling is left behind"
    );

    std::fs::remove_dir_all(&dir).expect("scratch directory is removable");
}
