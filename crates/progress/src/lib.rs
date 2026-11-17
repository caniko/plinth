//! Validator and version guard for Plinth project progress sources.
//!
//! A progress source is one UTF-8 TOML document that lists releases newest
//! first. [`parse`] rejects unknown keys, missing fields, blank text, invalid
//! or unordered versions and invalid, increasing or duplicate dates.
//! [`check_version`] then requires one exact version string.
//!
//! `progress.schema.json` describes the same document for editors. Rules that
//! JSON Schema cannot express (calendar validity, version uniqueness and
//! ordering, date ordering) are enforced here at runtime.
//!
//! Diagnostics carry one-based lines and Unicode-character columns.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::ops::Range;
use std::path::{Path, PathBuf};

use semver::Version;
use toml::Spanned;
use toml::de::{DeArray, DeTable, DeValue};

/// One progress document.
#[derive(Debug, Clone)]
pub struct Progress {
    /// Document title.
    pub title: String,
    /// Releases, newest first.
    pub entries: Vec<Entry>,
    /// Source of the document, copied into every [`Report`].
    source: PathBuf,
    /// Position of the `entries` value, used to anchor guard errors.
    entries_position: Position,
}

/// One release entry.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Exact canonical SemVer string, without a leading `v`.
    pub version: String,
    /// Quoted `YYYY-MM-DD` calendar date.
    pub date: String,
    /// Short release summary.
    pub summary: String,
    /// Milestones shown with this release.
    pub milestones: Vec<Milestone>,
    /// Completed work.
    pub done: Vec<String>,
    /// Planned work.
    pub next: Vec<String>,
    /// Known risks.
    pub risks: Vec<String>,
    /// Free-form sections, in display order.
    pub sections: Vec<Section>,
}

/// One milestone of a release.
#[derive(Debug, Clone)]
pub struct Milestone {
    /// Milestone name.
    pub name: String,
    /// Current state.
    pub status: Status,
    /// Optional display-only label.
    pub label: Option<String>,
}

/// Milestone state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Finished.
    Done,
    /// Started.
    InProgress,
    /// Started but blocked.
    Blocked,
    /// Not started.
    Planned,
    /// No longer planned.
    Dropped,
}

impl Status {
    fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "done" => Self::Done,
            "in_progress" => Self::InProgress,
            "blocked" => Self::Blocked,
            "planned" => Self::Planned,
            "dropped" => Self::Dropped,
            _ => return None,
        })
    }
}

const STATUS_NAMES: &str = "\"done\", \"in_progress\", \"blocked\", \"planned\", \"dropped\"";

/// One free-form section of a release.
#[derive(Debug, Clone)]
pub struct Section {
    /// Section heading.
    pub heading: String,
    /// Section body.
    pub body: String,
}

/// A one-based line and Unicode-character column in a progress source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// One-based line number.
    pub line: usize,
    /// One-based column counted in Unicode scalar values.
    pub column: usize,
}

/// One validation failure.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Source position when one is known; `None` for failures without a source
    /// location, such as an invalid guard argument.
    pub position: Option<Position>,
    /// Field path such as `entries[0].milestones[1].status`; `$` is the root.
    pub path: String,
    /// Message without the location or path prefix.
    pub message: String,
}

/// Every diagnostic produced by one failed parse or guard.
#[derive(Debug, Clone)]
pub struct Report {
    source: PathBuf,
    diagnostics: Vec<Diagnostic>,
}

impl Report {
    /// Diagnostics in source order.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Renders all diagnostics, one per line, as
    /// `error: <file>:<line>:<column>: <field-path>: <message>`.
    ///
    /// A diagnostic without a source position renders as
    /// `error: <file>: <field-path>: <message>`.
    pub fn render(&self) -> String {
        self.diagnostics
            .iter()
            .map(|diagnostic| match diagnostic.position {
                Some(position) => format!(
                    "error: {}:{}:{}: {}: {}",
                    self.source.display(),
                    position.line,
                    position.column,
                    diagnostic.path,
                    diagnostic.message
                ),
                None => format!(
                    "error: {}: {}: {}",
                    self.source.display(),
                    diagnostic.path,
                    diagnostic.message
                ),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Parses and fully validates a progress document.
///
/// `path` identifies the document in [`Report`] output. Parsing always runs
/// every check; a failed parse returns all deterministic diagnostics in source
/// order.
pub fn parse(source: &str, path: &Path) -> Result<Progress, Report> {
    let table = match DeTable::parse(source) {
        Ok(table) => table.into_inner(),
        Err(error) => {
            return Err(Report {
                source: path.to_path_buf(),
                diagnostics: vec![syntax_diagnostic(source, &error)],
            });
        }
    };

    let mut validator = Validator::new(source);
    let progress = validator.document(&table, path);

    if validator.diagnostics.is_empty() {
        Ok(progress)
    } else {
        let mut diagnostics = validator.diagnostics;
        diagnostics.sort_by_key(|diagnostic| {
            diagnostic
                .position
                .map(|position| (position.line, position.column))
        });
        Err(Report {
            source: path.to_path_buf(),
            diagnostics,
        })
    }
}

/// Requires `version` to appear verbatim among the entries.
///
/// The argument must itself be canonical SemVer; `v1.2.0` and `1.2` are
/// rejected before the membership test.
pub fn check_version(progress: &Progress, version: &str) -> Result<(), Report> {
    if let Err(message) = canonical_version(version) {
        return Err(Report {
            source: progress.source.clone(),
            diagnostics: vec![Diagnostic {
                position: None,
                path: "$".to_string(),
                message,
            }],
        });
    }

    if progress
        .entries
        .iter()
        .any(|entry| entry.version == version)
    {
        return Ok(());
    }

    Err(Report {
        source: progress.source.clone(),
        diagnostics: vec![Diagnostic {
            position: Some(progress.entries_position),
            path: "entries".to_string(),
            message: format!("version \"{version}\" not found"),
        }],
    })
}

/// One string value with its position in the source.
struct Text {
    value: String,
    span: Range<usize>,
}

/// Walks the parsed document once, building the typed values and collecting
/// every deterministic diagnostic. Field paths come from the walk, never from
/// the source text.
struct Validator<'a> {
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
    versions: HashSet<String>,
    previous_version: Option<(String, Version)>,
    previous_date: Option<(String, (u32, u32, u32))>,
}

impl<'a> Validator<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            diagnostics: Vec::new(),
            versions: HashSet::new(),
            previous_version: None,
            previous_date: None,
        }
    }

    fn push(&mut self, span: Range<usize>, path: &str, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic {
            position: Some(position(self.source, span.start)),
            path: path.to_string(),
            message: message.into(),
        });
    }

    fn document(&mut self, table: &DeTable<'_>, path: &Path) -> Progress {
        self.unknown_keys(table, "$", &["title", "entries"]);

        let title = self.text(table, "title", "$", None);
        let entries_value = self.required(table, "entries", "$", None);
        let entries_position = entries_value.map_or_else(
            || position(self.source, self.source.len()),
            |value| position(self.source, value.span().start),
        );

        let mut entries = Vec::new();
        if let Some(value) = entries_value
            && let Some(array) = self.array(value, "entries")
        {
            if array.is_empty() {
                self.push(value.span(), "entries", "must contain at least one entry");
            }
            for (index, item) in array.iter().enumerate() {
                let item_path = format!("entries[{index}]");
                let Some(entry) = self.table(item, &item_path) else {
                    continue;
                };
                self.unknown_keys(
                    entry,
                    &item_path,
                    &[
                        "version",
                        "date",
                        "summary",
                        "milestones",
                        "done",
                        "next",
                        "risks",
                        "sections",
                    ],
                );
                entries.push(self.entry(entry, &item_path, Some(item.span().start)));
            }
        }

        Progress {
            title: title.map_or_else(String::new, |text| text.value),
            entries,
            source: path.to_path_buf(),
            entries_position,
        }
    }

    fn entry(&mut self, table: &DeTable<'_>, base: &str, anchor: Option<usize>) -> Entry {
        let version = self.text(table, "version", base, anchor);
        if let Some(text) = &version {
            self.version(base, text);
        }
        let date = self.text(table, "date", base, anchor);
        if let Some(text) = &date {
            self.date(base, text);
        }
        let summary = self.text(table, "summary", base, anchor);
        let done = self.text_list(table, "done", base, anchor);
        let next = self.text_list(table, "next", base, anchor);
        let risks = self.text_list(table, "risks", base, anchor);
        let milestones = self.milestones(table, base, anchor);
        let sections = self.sections(table, base, anchor);

        Entry {
            version: version.map_or_else(String::new, |text| text.value),
            date: date.map_or_else(String::new, |text| text.value),
            summary: summary.map_or_else(String::new, |text| text.value),
            milestones,
            done,
            next,
            risks,
            sections,
        }
    }

    fn version(&mut self, base: &str, text: &Text) {
        let path = format!("{base}.version");
        match canonical_version(&text.value) {
            Ok(version) => {
                if !self.versions.insert(text.value.clone()) {
                    self.push(
                        text.span.clone(),
                        &path,
                        format!("duplicate version \"{}\"", text.value),
                    );
                } else if let Some((previous, previous_number)) = &self.previous_version {
                    match previous_number.cmp_precedence(&version) {
                        Ordering::Greater => {}
                        Ordering::Equal => self.push(
                            text.span.clone(),
                            &path,
                            format!(
                                "version \"{}\" has the same SemVer precedence as preceding \"{previous}\"",
                                text.value
                            ),
                        ),
                        Ordering::Less => self.push(
                            text.span.clone(),
                            &path,
                            format!(
                                "version \"{}\" must be lower than preceding \"{previous}\"",
                                text.value
                            ),
                        ),
                    }
                }
                self.previous_version = Some((text.value.clone(), version));
            }
            Err(message) => self.push(text.span.clone(), &path, message),
        }
    }

    fn date(&mut self, base: &str, text: &Text) {
        let path = format!("{base}.date");
        match parse_date(&text.value) {
            Ok(date) => {
                if let Some((previous, previous_parts)) = &self.previous_date
                    && date > *previous_parts
                {
                    self.push(
                        text.span.clone(),
                        &path,
                        format!(
                            "date \"{}\" is later than the preceding entry's \"{previous}\"; entries are newest first",
                            text.value
                        ),
                    );
                }
                self.previous_date = Some((text.value.clone(), date));
            }
            Err(message) => self.push(text.span.clone(), &path, message),
        }
    }

    fn milestones(
        &mut self,
        table: &DeTable<'_>,
        base: &str,
        anchor: Option<usize>,
    ) -> Vec<Milestone> {
        let Some(value) = self.required(table, "milestones", base, anchor) else {
            return Vec::new();
        };
        let path = join(base, "milestones");
        let Some(array) = self.array(value, &path) else {
            return Vec::new();
        };

        let mut milestones = Vec::new();
        for (index, item) in array.iter().enumerate() {
            let item_path = format!("{path}[{index}]");
            let Some(milestone) = self.table(item, &item_path) else {
                continue;
            };
            self.unknown_keys(milestone, &item_path, &["name", "status", "label"]);
            let anchor = Some(item.span().start);
            let name = self.text(milestone, "name", &item_path, anchor);
            let status = self.status(milestone, &item_path, anchor);
            let label = match milestone.get("label") {
                Some(value) => self.string_value(value, &join(&item_path, "label")),
                None => None,
            };
            let (Some(name), Some(status)) = (name, status) else {
                continue;
            };
            milestones.push(Milestone {
                name: name.value,
                status,
                label,
            });
        }
        milestones
    }

    fn sections(&mut self, table: &DeTable<'_>, base: &str, anchor: Option<usize>) -> Vec<Section> {
        let Some(value) = self.required(table, "sections", base, anchor) else {
            return Vec::new();
        };
        let path = join(base, "sections");
        let Some(array) = self.array(value, &path) else {
            return Vec::new();
        };

        let mut sections = Vec::new();
        for (index, item) in array.iter().enumerate() {
            let item_path = format!("{path}[{index}]");
            let Some(section) = self.table(item, &item_path) else {
                continue;
            };
            self.unknown_keys(section, &item_path, &["heading", "body"]);
            let anchor = Some(item.span().start);
            let heading = self.text(section, "heading", &item_path, anchor);
            let body = self.text(section, "body", &item_path, anchor);
            let (Some(heading), Some(body)) = (heading, body) else {
                continue;
            };
            sections.push(Section {
                heading: heading.value,
                body: body.value,
            });
        }
        sections
    }

    fn text_list(
        &mut self,
        table: &DeTable<'_>,
        key: &str,
        base: &str,
        anchor: Option<usize>,
    ) -> Vec<String> {
        let Some(value) = self.required(table, key, base, anchor) else {
            return Vec::new();
        };
        let path = join(base, key);
        let Some(array) = self.array(value, &path) else {
            return Vec::new();
        };

        let mut items = Vec::new();
        for (index, item) in array.iter().enumerate() {
            let item_path = format!("{path}[{index}]");
            if let Some(text) = self.string_value(item, &item_path) {
                items.push(text);
            }
        }
        items
    }

    fn status(&mut self, table: &DeTable<'_>, base: &str, anchor: Option<usize>) -> Option<Status> {
        let value = self.required(table, "status", base, anchor)?;
        let path = join(base, "status");
        let DeValue::String(text) = value.get_ref() else {
            self.push(value.span(), &path, "expected a string");
            return None;
        };
        let status = Status::parse(text);
        if status.is_none() {
            self.push(
                value.span(),
                &path,
                format!("unknown variant \"{text}\"; expected one of {STATUS_NAMES}"),
            );
        }
        status
    }

    /// Reads one required string field and rejects blank text.
    fn text(
        &mut self,
        table: &DeTable<'_>,
        key: &str,
        base: &str,
        anchor: Option<usize>,
    ) -> Option<Text> {
        let value = self.required(table, key, base, anchor)?;
        let path = join(base, key);
        let value_text = self.string_value(value, &path)?;
        Some(Text {
            value: value_text,
            span: value.span(),
        })
    }

    /// Reads one string value at `path` and rejects blank text.
    fn string_value(&mut self, value: &Spanned<DeValue<'_>>, path: &str) -> Option<String> {
        let DeValue::String(text) = value.get_ref() else {
            self.push(value.span(), path, "expected a string");
            return None;
        };
        if text.trim().is_empty() {
            self.push(value.span(), path, "must not be empty or whitespace-only");
            return None;
        }
        Some(text.to_string())
    }

    fn array<'t, 'i>(
        &mut self,
        value: &'t Spanned<DeValue<'i>>,
        path: &str,
    ) -> Option<&'t DeArray<'i>> {
        match value.get_ref() {
            DeValue::Array(array) => Some(array),
            _ => {
                self.push(value.span(), path, "expected an array");
                None
            }
        }
    }

    fn table<'t, 'i>(
        &mut self,
        value: &'t Spanned<DeValue<'i>>,
        path: &str,
    ) -> Option<&'t DeTable<'i>> {
        match value.get_ref() {
            DeValue::Table(table) => Some(table),
            _ => {
                self.push(value.span(), path, "expected a table");
                None
            }
        }
    }

    fn required<'t, 'i>(
        &mut self,
        table: &'t DeTable<'i>,
        key: &str,
        base: &str,
        anchor: Option<usize>,
    ) -> Option<&'t Spanned<DeValue<'i>>> {
        match table.get(key) {
            Some(value) => Some(value),
            None => {
                let at = anchor.unwrap_or(self.source.len());
                self.push(at..at, base, format!("missing required field \"{key}\""));
                None
            }
        }
    }

    fn unknown_keys(&mut self, table: &DeTable<'_>, base: &str, allowed: &[&str]) {
        for (key, _) in table.iter() {
            let name = key.get_ref().as_ref();
            if allowed.contains(&name) {
                continue;
            }
            self.push(
                key.span(),
                &join(base, name),
                format!("unknown field \"{name}\""),
            );
        }
    }
}

fn join(base: &str, key: &str) -> String {
    if base == "$" {
        key.to_string()
    } else {
        format!("{base}.{key}")
    }
}

/// One diagnostic for a document that does not parse as TOML.
///
/// This is the only failure that stops before validation, so it keeps the
/// parser span and the parser's UTF-8 message.
fn syntax_diagnostic(source: &str, error: &toml::de::Error) -> Diagnostic {
    let span = error.span().unwrap_or(source.len()..source.len());
    Diagnostic {
        position: Some(position(source, span.start)),
        path: syntax_path(source, span.start),
        message: error.message().to_string(),
    }
}

/// Field path of a syntax error, from the parser's own recovered tree.
///
/// Recovery keeps the partially built value whose span reaches the error, so
/// the deepest containing node names the key that was being read.
fn syntax_path(source: &str, offset: usize) -> String {
    let (table, _) = DeTable::parse_recoverable(source);
    locate(table.get_ref(), "$", offset).map_or_else(|| "$".to_string(), |found| found.0)
}

/// Deepest value containing `offset`: path, depth and span start.
///
/// Children are always visited: an array-of-tables element's span covers only
/// its `[[header]]`, so container spans cannot prune the search.
fn locate(table: &DeTable<'_>, path: &str, offset: usize) -> Option<(String, usize, usize)> {
    let mut best: Option<(String, usize, usize)> = None;
    let contains = |span: std::ops::Range<usize>| span.start <= offset && offset <= span.end;

    for (key, value) in table.iter() {
        let span = value.span();
        let child = join(path, key.get_ref().as_ref());
        if contains(span.clone()) {
            consider(&mut best, child.clone(), 1, span.start);
        }
        match value.get_ref() {
            DeValue::Table(inner) => {
                if let Some((found, depth, start)) = locate(inner, &child, offset) {
                    consider(&mut best, found, depth + 1, start);
                }
            }
            DeValue::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    let item_span = item.span();
                    let item_path = format!("{child}[{index}]");
                    if contains(item_span.clone()) {
                        consider(&mut best, item_path.clone(), 2, item_span.start);
                    }
                    if let DeValue::Table(inner) = item.get_ref()
                        && let Some((found, depth, start)) = locate(inner, &item_path, offset)
                    {
                        consider(&mut best, found, depth + 2, start);
                    }
                }
            }
            _ => {}
        }
    }

    best
}

fn consider(best: &mut Option<(String, usize, usize)>, path: String, depth: usize, start: usize) {
    let better = match best {
        Some((_, best_depth, best_start)) => {
            depth > *best_depth || (depth == *best_depth && start >= *best_start)
        }
        None => true,
    };
    if better {
        *best = Some((path, depth, start));
    }
}

fn canonical_version(text: &str) -> Result<Version, String> {
    if text.starts_with('v') {
        return Err(format!("version \"{text}\" must not have a leading \"v\""));
    }

    let version =
        Version::parse(text).map_err(|error| format!("invalid version \"{text}\": {error}"))?;
    if version.to_string() == text {
        Ok(version)
    } else {
        Err(format!("version \"{text}\" is not canonical SemVer"))
    }
}

fn parse_date(text: &str) -> Result<(u32, u32, u32), String> {
    let invalid = || format!("invalid date \"{text}\": expected a real YYYY-MM-DD calendar date");
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(invalid());
    }

    let number = |slice: &[u8]| {
        slice.iter().try_fold(0u32, |value, byte| {
            byte.is_ascii_digit()
                .then(|| value * 10 + u32::from(*byte - b'0'))
        })
    };

    let year = number(&bytes[0..4]).ok_or_else(&invalid)?;
    let month = number(&bytes[5..7]).ok_or_else(&invalid)?;
    let day = number(&bytes[8..10]).ok_or_else(&invalid)?;
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return Err(invalid());
    }
    Ok((year, month, day))
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        _ => 0,
    }
}

fn position(source: &str, offset: usize) -> Position {
    let mut offset = offset.min(source.len());
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let column = source[line_start..offset].chars().count() + 1;
    Position { line, column }
}
