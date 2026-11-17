//! Runtime validation, locations and the version guard.

use std::path::{Path, PathBuf};

use plinth_progress::{Position, Report, check_version, parse};

const EXAMPLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/progress.toml");

fn example_path() -> PathBuf {
    PathBuf::from(EXAMPLE)
}

fn example_source() -> String {
    std::fs::read_to_string(EXAMPLE).expect("example is readable")
}

fn parse_with_name(source: &str) -> Result<plinth_progress::Progress, Report> {
    parse(source, Path::new("progress.toml"))
}

fn document(entries: &[(&str, &str)]) -> String {
    let mut source = String::from("title = \"Title\"\n");
    for (version, date) in entries {
        source.push_str(&format!(
            "\n[[entries]]\nversion = \"{version}\"\ndate = \"{date}\"\nsummary = \"Summary.\"\ndone = []\nnext = []\nrisks = []\nmilestones = []\nsections = []\n"
        ));
    }
    source
}

const ENTRY_DOCUMENT: &str = r#"title = "Title"

[[entries]]
version = "1.0.0"
date = "2026-11-17"
summary = "Summary."
done = []
next = []
risks = []
milestones = []
sections = []
"#;

fn single_error(report: Report) -> plinth_progress::Diagnostic {
    assert_eq!(report.diagnostics().len(), 1, "{report:?}");
    report.diagnostics()[0].clone()
}

fn prefixes(report: &Report) -> Vec<String> {
    report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            let position = diagnostic.position.expect("source position");
            format!(
                "progress.toml:{}:{}: {}",
                position.line, position.column, diagnostic.path
            )
        })
        .collect()
}

#[test]
fn parse_reports_all_errors_in_order() {
    let source = "title = \"   \"\n\
\n\
[[entries]]\n\
version = \"1.0.0\"\n\
date = \"2026-11-17\"\n\
summary = \" \"\n\
extra = 1\n\
other = 2\n\
done = []\n\
next = []\n\
risks = []\n\
milestones = []\n\
sections = []\n";

    let report = parse_with_name(source).unwrap_err();
    assert_eq!(
        prefixes(&report),
        [
            "progress.toml:1:9: title",
            "progress.toml:6:11: entries[0].summary",
            "progress.toml:7:1: entries[0].extra",
            "progress.toml:8:1: entries[0].other",
        ]
    );
    assert!(
        report.diagnostics()[2].message.contains("unknown field"),
        "{}",
        report.diagnostics()[2].message
    );
}

#[test]
fn parse_paths_follow_document_structure() {
    // Interleaved array-of-tables keep their per-array indices.
    let interleaved = ENTRY_DOCUMENT.replace(
        "milestones = []\nsections = []",
        "[[entries.milestones]]\nname = \"First\"\nstatus = \"done\"\n[[entries.sections]]\nheading = \"H\"\nbody = \"B\"\n[[entries.milestones]]\nname = \"Second\"\nstatus = \"invalid\"",
    );
    assert_eq!(
        prefixes(&parse_with_name(&interleaved).unwrap_err()),
        ["progress.toml:18:10: entries[0].milestones[1].status"]
    );

    // A spaced header still anchors missing fields at that header.
    let spaced = ENTRY_DOCUMENT
        .replace("[[entries]]", "[[ entries ]]")
        .replace("risks = []\n", "");
    let report = parse_with_name(&spaced).unwrap_err();
    assert_eq!(prefixes(&report), ["progress.toml:3:1: entries[0]"]);
    assert_eq!(
        report.diagnostics()[0].message,
        "missing required field \"risks\""
    );

    // Inline tables carry their own value spans and paths.
    let inline_milestone = ENTRY_DOCUMENT.replace(
        "milestones = []",
        "milestones = [{name = \"M\", status = \"invalid\"}]",
    );
    assert_eq!(
        prefixes(&parse_with_name(&inline_milestone).unwrap_err()),
        ["progress.toml:10:37: entries[0].milestones[0].status"]
    );

    let inline_entry = "title = \"Title\"\n\
entries = [{version = \"1.0.0\", date = \"2026-11-17\", summary = \"S\", done = [], next = [], risks = [], milestones = [{name = \"M\", status = \"invalid\"}], sections = []}]\n";
    assert_eq!(
        prefixes(&parse_with_name(inline_entry).unwrap_err()),
        ["progress.toml:2:138: entries[0].milestones[0].status"]
    );

    // Multiline arrays report the item index, not the array line.
    let multiline = ENTRY_DOCUMENT.replace("done = []", "done = [\n\"okay\",\n42\n]");
    assert_eq!(
        prefixes(&parse_with_name(&multiline).unwrap_err()),
        ["progress.toml:9:1: entries[0].done[1]"]
    );

    // Quoted keys resolve to the same path as plain keys.
    let quoted = ENTRY_DOCUMENT.replace("summary = \"Summary.\"", "\"summary\" = 42");
    assert_eq!(
        prefixes(&parse_with_name(&quoted).unwrap_err()),
        ["progress.toml:6:13: entries[0].summary"]
    );
}

#[test]
fn parse_reports_field_locations() {
    // The blank item sits after a multi-byte character, so a byte column would
    // be larger than the Unicode-character column.
    let source = "title = \"Überblick — 進捗\"\n\
[[entries]]\n\
version = \"1.2.0\"\n\
date = \"2026-11-17\"\n\
summary = \"Release\"\n\
done = [\"ok\", \"😀\", \"  \"]\n\
next = []\n\
risks = []\n\
milestones = []\n\
sections = []\n";
    let diagnostic = single_error(parse_with_name(source).unwrap_err());
    assert_eq!(diagnostic.path, "entries[0].done[2]");
    assert_eq!(
        diagnostic.position,
        Some(Position {
            line: 6,
            column: 20
        })
    );
    assert!(
        diagnostic.message.contains("whitespace"),
        "{}",
        diagnostic.message
    );

    // A missing field points at the enclosing table header. The blank item is
    // replaced so this case has exactly one diagnostic.
    let source = source
        .replace("risks = []\n", "")
        .replace("\"  \"", "\"ok\"");
    let diagnostic = single_error(parse_with_name(&source).unwrap_err());
    assert_eq!(diagnostic.path, "entries[0]");
    assert_eq!(diagnostic.position, Some(Position { line: 2, column: 1 }));
    assert_eq!(
        diagnostic.message, "missing required field \"risks\"",
        "missing fields keep the field name"
    );

    // A syntax error keeps the parser span, here at EOF.
    let diagnostic = single_error(parse_with_name("title = \"x\"\nentries = [\n").unwrap_err());
    assert_eq!(diagnostic.path, "entries");
    assert_eq!(
        diagnostic.position,
        Some(Position {
            line: 2,
            column: 12
        })
    );

    // A missing root field cannot point at a value, so it points at EOF. The
    // empty entries list is reported too, so pick the root diagnostic.
    let report = parse_with_name("entries = []\n").unwrap_err();
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.path == "$")
        .expect("missing title is reported");
    assert_eq!(diagnostic.position, Some(Position { line: 2, column: 1 }));
    assert_eq!(diagnostic.message, "missing required field \"title\"");
}

#[test]
fn parse_rejects_unknown_fields() {
    let cases = [
        (
            ENTRY_DOCUMENT.replace("title = \"Title\"\n", "title = \"Title\"\nbogus = 1\n"),
            "bogus",
        ),
        (
            ENTRY_DOCUMENT.replace("summary = \"Summary.\"\n", "summary = \"Summary.\"\nbogus = 1\n"),
            "entries[0].bogus",
        ),
        (
            ENTRY_DOCUMENT.replace(
                "milestones = []\nsections = []\n",
                "[[entries.milestones]]\nname = \"Milestone\"\nstatus = \"done\"\nbogus = 1\n\n[[entries.sections]]\nheading = \"Heading\"\nbody = \"Body.\"\n",
            ),
            "entries[0].milestones[0].bogus",
        ),
        (
            ENTRY_DOCUMENT.replace(
                "milestones = []\nsections = []\n",
                "[[entries.milestones]]\nname = \"Milestone\"\nstatus = \"done\"\n\n[[entries.sections]]\nheading = \"Heading\"\nbody = \"Body.\"\nbogus = 1\n",
            ),
            "entries[0].sections[0].bogus",
        ),
    ];

    for (source, path) in cases {
        let diagnostic = single_error(parse_with_name(&source).unwrap_err());
        assert_eq!(diagnostic.path, path);
        assert!(
            diagnostic.message.contains("unknown field"),
            "{}",
            diagnostic.message
        );
    }
}

#[test]
fn parse_rejects_semver_order_and_dates() {
    let rejected = [
        (
            document(&[("1.0.0", "2026-11-17"), ("1.0.0", "2026-11-16")]),
            "entries[1].version",
            "duplicate version",
        ),
        (
            document(&[("1.0.0", "2026-11-17"), ("1.2.0", "2026-11-16")]),
            "entries[1].version",
            "must be lower than preceding",
        ),
        (
            document(&[("1.0.0+a", "2026-11-17"), ("1.0.0+b", "2026-11-16")]),
            "entries[1].version",
            "same SemVer precedence",
        ),
        (
            document(&[("v1.2.0", "2026-11-17")]),
            "entries[0].version",
            "leading \"v\"",
        ),
        (
            document(&[("1.2", "2026-11-17")]),
            "entries[0].version",
            "invalid version",
        ),
        (
            document(&[("2.0.0", "2026-11-17"), ("1.0.0", "2026-02-30")]),
            "entries[1].date",
            "invalid date",
        ),
        (
            document(&[("2.0.0", "2026-01-01"), ("1.0.0", "2026-02-01")]),
            "entries[1].date",
            "later than the preceding entry",
        ),
        (
            document(&[("1.0.0", "2027-02-29")]),
            "entries[0].date",
            "invalid date",
        ),
    ];

    for (source, path, message) in rejected {
        let diagnostic = single_error(parse_with_name(&source).unwrap_err());
        assert_eq!(diagnostic.path, path);
        assert!(
            diagnostic.message.contains(message),
            "{}",
            diagnostic.message
        );
    }

    // Prereleases order by SemVer precedence and same-day entries are allowed.
    parse_with_name(&document(&[
        ("2.0.0", "2026-11-17"),
        ("1.0.0", "2026-11-17"),
        ("1.0.0-rc.1", "2026-11-16"),
    ]))
    .expect("descending versions with same-day dates are valid");

    // Leap days are calendar-checked.
    parse_with_name(&document(&[("1.0.0", "2024-02-29")])).expect("2024-02-29 exists");
}

#[test]
fn check_version_present() {
    let source = example_source();
    let progress = parse(&source, &example_path()).expect("example parses");

    check_version(&progress, "1.2.0").expect("newest entry is listed");
    check_version(&progress, "1.1.0").expect("membership does not require the newest entry");
}

#[test]
fn check_version_missing() {
    let source = example_source();
    let path = example_path();
    let progress = parse(&source, &path).expect("example parses");

    let report = check_version(&progress, "9.9.9").unwrap_err();
    let diagnostic = single_error(report.clone());
    assert_eq!(diagnostic.path, "entries");
    assert_eq!(diagnostic.position, Some(Position { line: 4, column: 1 }));
    assert_eq!(diagnostic.message, "version \"9.9.9\" not found");
    assert_eq!(
        report.render(),
        format!(
            "error: {}:4:1: entries: version \"9.9.9\" not found",
            path.display()
        )
    );

    for (candidate, needle) in [("v1.2.0", "leading \"v\""), ("1.2", "invalid version")] {
        let report = check_version(&progress, candidate).unwrap_err();
        let diagnostic = single_error(report.clone());
        assert_eq!(diagnostic.position, None);
        assert_eq!(diagnostic.path, "$");
        assert!(
            diagnostic.message.contains(needle),
            "{}",
            diagnostic.message
        );
        assert!(
            report
                .render()
                .starts_with(&format!("error: {}: $: ", path.display())),
            "{}",
            report.render()
        );
    }
}

#[test]
fn parse_syntax_error_paths_inside_array_of_tables() {
    // `[[entries]]` spans only its header, so the syntax-error path search
    // must still descend into the entry's values.
    let source = ENTRY_DOCUMENT.replace("risks = []", "risks = [\"ok\", @]");
    let path = parse_with_name(&source).unwrap_err().diagnostics()[0]
        .path
        .clone();
    assert_eq!(path, "entries[0].risks[1]");
}
