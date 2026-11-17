//! Draft entry skeleton for a new release.
//!
//! [`draft`] never writes a file: it prints one `[[entries]]` table, newest
//! first, so a human or agent pastes it at the top of the entries and fills it.
//! `summary = ""` is deliberate, so the unchecked draft fails [`crate::parse`]
//! and cannot be committed by accident.

use std::fmt::Write as _;
use std::path::Path;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use semver::Version;

use crate::canonical_version;
use crate::parse_date;
use crate::{Diagnostic, Progress, Report};

/// Prints a TOML entry skeleton for `version` and returns it with warnings.
///
/// Commits come from `git log` in the directory of `progress`'s source, after
/// `since` or the newest tag. Every commit subject becomes a comment line, so
/// control characters are stripped and no subject can end the comment.
pub fn draft(
    progress: &Progress,
    version: &str,
    since: Option<&str>,
    date: Option<&str>,
) -> Result<(String, Vec<String>), Report> {
    let source = &progress.source;
    let parsed: Version = canonical_version(version).map_err(|message| error(source, message))?;

    if progress
        .entries
        .iter()
        .any(|entry| entry.version == version)
    {
        return Err(error(
            source,
            format!("version \"{version}\" already has an entry"),
        ));
    }
    if let Some(newest) = progress.entries.first() {
        let newest_version =
            canonical_version(&newest.version).map_err(|message| error(source, message))?;
        if parsed.cmp_precedence(&newest_version) != std::cmp::Ordering::Greater {
            return Err(error(
                source,
                format!(
                    "version \"{version}\" must be newer than \"{}\"",
                    newest.version
                ),
            ));
        }
    }

    let date = match date {
        Some(text) => {
            parse_date(text).map_err(|message| error(source, message))?;
            text.to_string()
        }
        None => today_utc(),
    };

    let directory = source
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let (range, since_label, warnings) = commit_range(source, directory, since)?;
    let commits = commits(source, directory, &range)?;

    let mut out = String::new();
    let _ = writeln!(out, "[[entries]]");
    let _ = writeln!(out, "version = \"{version}\"");
    let _ = writeln!(out, "date = \"{date}\"");
    let _ = writeln!(
        out,
        "# Fill every field from the commits below and the changelog, then run:"
    );
    let _ = writeln!(
        out,
        "#   plinth-progress check --version {version} {}",
        comment_safe(&source.display().to_string())
    );
    let _ = writeln!(out, "summary = \"\"");
    let _ = writeln!(out, "done = []");
    let _ = writeln!(out, "next = []");
    let _ = writeln!(out, "risks = []");
    let _ = writeln!(out, "milestones = []");
    let _ = writeln!(out, "sections = []");
    let _ = writeln!(
        out,
        "# Commits since {} ({}):",
        comment_safe(&since_label),
        commits.len()
    );
    for (hash, subject) in &commits {
        let hash = comment_safe(hash);
        let subject = comment_safe(subject);
        if subject.is_empty() {
            let _ = writeln!(out, "#   {hash}");
        } else {
            let _ = writeln!(out, "#   {hash} {subject}");
        }
    }

    Ok((out, warnings))
}

/// The `git log` range and its human label, plus the no-tag warning.
fn commit_range(
    source: &Path,
    directory: &Path,
    since: Option<&str>,
) -> Result<(String, String, Vec<String>), Report> {
    let inside = run_git(directory, &["rev-parse", "--is-inside-work-tree"])
        .map_err(|message| error(source, message))?;
    if !inside.status.success() {
        return Err(error(
            source,
            format!("not a git repository: {}", stderr_text(&inside)),
        ));
    }

    if let Some(reference) = since {
        if reference.starts_with('-') {
            return Err(error(source, "--since must not start with \"-\""));
        }
        return Ok((
            format!("{reference}..HEAD"),
            reference.to_string(),
            Vec::new(),
        ));
    }

    let described = run_git(directory, &["describe", "--tags", "--abbrev=0"])
        .map_err(|message| error(source, message))?;
    if described.status.success() {
        let tag = String::from_utf8_lossy(&described.stdout)
            .trim()
            .to_string();
        Ok((format!("{tag}..HEAD"), tag, Vec::new()))
    } else {
        Ok((
            "HEAD".to_string(),
            "the first commit".to_string(),
            vec!["no git tag found; listing all commits".to_string()],
        ))
    }
}

/// Newest-first `(hash, subject)` pairs for `range`.
fn commits(source: &Path, directory: &Path, range: &str) -> Result<Vec<(String, String)>, Report> {
    let output = run_git(
        directory,
        // A tag picked by `git describe` may itself look like an option
        // (`--output=...`), so the range is pinned as a revision.
        &[
            "log",
            "--no-merges",
            "--format=%h%x09%s",
            "--end-of-options",
            range,
            "--",
        ],
    )
    .map_err(|message| error(source, message))?;
    if !output.status.success() {
        return Err(error(source, stderr_text(&output)));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| match line.split_once('\t') {
            Some((hash, subject)) => (hash.to_string(), subject.to_string()),
            None => (line.to_string(), String::new()),
        })
        .collect())
}

fn run_git(directory: &Path, args: &[&str]) -> Result<Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|error| format!("could not run git: {error}"))
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

/// Removes every control character, so a commit subject can never end the
/// comment it is printed in or inject TOML.
fn comment_safe(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_control()).collect()
}

/// Today's date in UTC, as `YYYY-MM-DD`.
fn today_utc() -> String {
    let seconds = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(error) => -(error.duration().as_secs() as i64),
    };
    let (year, month, day) = days_to_civil(seconds.div_euclid(86_400));
    format!("{year:04}-{month:02}-{day:02}")
}

/// Converts days since the Unix epoch to a proleptic Gregorian `(year, month,
/// day)` in UTC.
///
/// Howard Hinnant's `civil_from_days`, which stays exact for every `i64`
/// without a calendar table or a crate.
fn days_to_civil(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// One position-less diagnostic, like an invalid guard argument.
fn error(source: &Path, message: impl Into<String>) -> Report {
    Report {
        source: source.to_path_buf(),
        diagnostics: vec![Diagnostic {
            position: None,
            path: "$".to_string(),
            message: message.into(),
        }],
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn days_to_civil() {
        assert_eq!(super::days_to_civil(0), (1970, 1, 1));
        assert_eq!(super::days_to_civil(11_016), (2000, 2, 29));
        assert_eq!(super::days_to_civil(20_774), (2026, 11, 17));
    }
}
