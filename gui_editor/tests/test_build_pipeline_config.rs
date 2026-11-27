//! Static checks of the build pipeline configuration for GEDT-STORY-00019:
//! T-01, T-54, T-55, and T-60.
//! Covers GEDT-AR-00024, GEDT-AR-00025, GEDT-AR-00030, GEDT-AR-00032, GEDT-AR-00033,
//! GEDT-AR-00034, and GEDT-AR-00055, by inspecting `deny.toml`, `deny-gui_editor.toml`,
//! `.github/workflows/ci.yml`, the tool image files in `container/`,
//! and the languages of all files that git tracks in the repository.
//! The pipeline script tests the behaviour of `build.sh`, `container/tool_image.sh`,
//! and `check_advisory_ignores.sh`:
//! `tests/pipeline/test_build_pipeline.sh` runs them against a fake container engine.
//! CI runs this file through `cargo test --locked` in the `test` job.
//!
//! Product owner rule for T-01:
//! the tests use only the Rust standard library, no parser crate.
//! The files are therefore read as text through two small subset readers below.
//! Each reader accepts only the constructs the committed files use,
//! and panics on anything else, such as dotted keys, duplicate tables or keys,
//! YAML anchors, aliases, merge keys, or tags.
//! A construct the readers cannot see through therefore fails a test,
//! instead of hiding a setting.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CI_CRATES: [&str; 4] = ["gui_editor", "sad2md", "sad_xml_sql", "tui_editor"];
// Crates outside the CI gate whose path dependencies still carry a version,
// by product owner decision.
// Kept apart from CI_CRATES because CI does not build or test these crates.
const VERSION_GUARDED_NON_CI_CRATES: [&str; 1] = ["aws2xml"];
const CRATES_IO_INDEX: &str = "https://github.com/rust-lang/crates.io-index";

fn gui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    gui_dir()
        .parent()
        .expect("gui_editor has a parent directory")
        .to_path_buf()
}

/// Panics when the file is missing, so a deleted `ci.yml` fails every CI test,
/// instead of skipping it (product owner decision).
fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

/// Rejects bytes that the line-based readers would misinterpret.
fn assert_plain_text(name: &str, text: &str) {
    assert!(
        !text.starts_with('\u{feff}'),
        "{name} starts with a byte order mark"
    );
    assert!(!text.contains('\r'), "{name} has carriage returns");
    assert!(!text.contains('\t'), "{name} has tabs");
    assert!(text.is_ascii(), "{name} has non-ASCII characters");
}

fn is_bare_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

// --- TOML subset reader for deny.toml ---

/// One `[table]` of `deny.toml`, with each value kept as its raw text.
struct TomlTable {
    name: String,
    entries: Vec<(String, String)>,
}

/// Cuts a TOML comment, honouring `#` inside basic strings.
fn strip_toml_comment(line: &str) -> &str {
    let mut in_string = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..i],
            _ => {}
        }
    }
    line
}

fn bracket_depth(text: &str) -> i32 {
    let mut in_string = false;
    let mut depth = 0;
    for c in text.chars() {
        match c {
            '"' => in_string = !in_string,
            '[' | '{' if !in_string => depth += 1,
            ']' | '}' if !in_string => depth -= 1,
            _ => {}
        }
    }
    depth
}

/// Reads `deny.toml` into tables.
/// Accepts bare `[name]` headers, bare `key = value` lines, and multi-line arrays.
/// Panics on dotted or quoted keys, nested or array tables, literal strings, escapes,
/// keys before the first header, and duplicate tables or keys.
fn parse_toml(text: &str) -> Vec<TomlTable> {
    assert_plain_text("deny.toml", text);
    let mut tables: Vec<TomlTable> = Vec::new();
    let mut lines = text.lines();
    while let Some(raw) = lines.next() {
        let line = strip_toml_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        assert!(
            !line.contains('\''),
            "deny.toml literal strings are not supported: {raw}"
        );
        assert!(
            !line.contains('\\'),
            "deny.toml escapes are not supported: {raw}"
        );
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            assert!(
                is_bare_key(name),
                "deny.toml unsupported table header: {raw}"
            );
            assert!(
                tables.iter().all(|t| t.name != name),
                "deny.toml duplicate table [{name}]"
            );
            tables.push(TomlTable {
                name: name.to_string(),
                entries: Vec::new(),
            });
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("deny.toml line is not key = value: {raw}"));
        let key = key.trim();
        assert!(is_bare_key(key), "deny.toml unsupported key: {raw}");
        let mut value = value.trim().to_string();
        while bracket_depth(&value) > 0 {
            let next = lines
                .next()
                .unwrap_or_else(|| panic!("deny.toml array {key} is not closed"));
            value.push(' ');
            value.push_str(strip_toml_comment(next).trim());
        }
        assert_eq!(
            bracket_depth(&value),
            0,
            "deny.toml unbalanced value for {key}"
        );
        let table = tables
            .last_mut()
            .unwrap_or_else(|| panic!("deny.toml key {key} before the first table"));
        assert!(
            table.entries.iter().all(|(k, _)| k != key),
            "deny.toml duplicate key {key} in [{}]",
            table.name
        );
        table.entries.push((key.to_string(), value));
    }
    tables
}

/// The shared policy of sad2md, sad_xml_sql, and tui_editor.
fn deny_toml() -> Vec<TomlTable> {
    parse_toml(&read(&gui_dir().join("deny.toml")))
}

/// The gui_editor policy: the shared policy,
/// plus the gui_editor `licenses.exceptions` and `bans.skip` entries (T-02).
fn gui_deny_toml() -> Vec<TomlTable> {
    parse_toml(&read(&gui_dir().join("deny-gui_editor.toml")))
}

fn table<'a>(cfg: &'a [TomlTable], name: &str) -> &'a TomlTable {
    cfg.iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("deny.toml has no [{name}] table"))
}

fn toml_raw<'a>(section: &'a TomlTable, key: &str) -> Option<&'a str> {
    section
        .entries
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

fn toml_str<'a>(section: &'a TomlTable, key: &str) -> Option<&'a str> {
    toml_raw(section, key)
        .and_then(|v| v.strip_prefix('"'))
        .and_then(|v| v.strip_suffix('"'))
        .filter(|v| !v.contains('"'))
}

fn toml_bool(section: &TomlTable, key: &str) -> Option<bool> {
    match toml_raw(section, key) {
        Some("true") => Some(true),
        Some("false") => Some(false),
        _ => None,
    }
}

/// True when `key` is absent or is the empty array `[]`.
fn toml_array_is_empty(section: &TomlTable, key: &str) -> bool {
    toml_raw(section, key).is_none_or(|v| v.replace(' ', "") == "[]")
}

/// Returns the strings of the array at `key`, or an empty list when the key is absent.
/// Panics when the value is not an array of basic strings.
fn toml_string_array(section: &TomlTable, key: &str) -> Vec<String> {
    toml_raw(section, key).map_or_else(Vec::new, |raw| parse_string_array(key, raw))
}

/// Reads a raw one-line array of basic strings, such as `["a", "b"]`.
fn parse_string_array(key: &str, raw: &str) -> Vec<String> {
    let inner = raw
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .unwrap_or_else(|| panic!("{key} is not an array"));
    inner
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| {
            item.strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .filter(|v| !v.contains('"'))
                .unwrap_or_else(|| panic!("{key} holds a non-string item: {item}"))
                .to_string()
        })
        .collect()
}

/// Returns the fields of each inline table in the array at `key`,
/// or an empty list when the key is absent.
/// Example: `[{ crate = "a", reason = "b" }, { crate = "c", reason = "d" }]`.
/// Panics on items that are not inline tables.
fn toml_inline_table_array(section: &TomlTable, key: &str) -> Vec<Vec<(String, String)>> {
    let Some(raw) = toml_raw(section, key) else {
        return Vec::new();
    };
    let inner = raw
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .unwrap_or_else(|| panic!("{key} is not an array"));
    let mut items = Vec::new();
    let mut item = String::new();
    let mut in_string = false;
    let mut depth = 0;
    // Splits only at commas outside strings and nested brackets,
    // so a reason or an inner array with commas stays in one item.
    for c in inner.chars().chain(std::iter::once(',')) {
        match c {
            '"' => in_string = !in_string,
            '[' | '{' if !in_string => depth += 1,
            ']' | '}' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                if !item.trim().is_empty() {
                    items.push(inline_table_fields(&item));
                }
                item.clear();
                continue;
            }
            _ => {}
        }
        item.push(c);
    }
    items
}

// --- YAML subset reader for ci.yml ---

#[derive(Debug)]
enum Yaml {
    Null,
    Scalar { text: String, quoted: bool },
    Seq(Vec<Yaml>),
    Map(Vec<(String, Yaml)>),
}

static NULL: Yaml = Yaml::Null;

impl Yaml {
    /// Mirrors YAML indexing: a missing key or a non-mapping yields `Null`.
    fn at(&self, key: &str) -> &Yaml {
        self.get(key).unwrap_or(&NULL)
    }

    fn get(&self, key: &str) -> Option<&Yaml> {
        match self {
            Yaml::Map(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn as_map(&self) -> Option<&[(String, Yaml)]> {
        match self {
            Yaml::Map(entries) => Some(entries),
            _ => None,
        }
    }

    fn as_seq(&self) -> Option<&[Yaml]> {
        match self {
            Yaml::Seq(items) => Some(items),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Yaml::Scalar { text, .. } => Some(text),
            _ => None,
        }
    }

    /// Only an unquoted `true` or `false` is a boolean.
    fn as_bool(&self) -> Option<bool> {
        match self {
            Yaml::Scalar {
                text,
                quoted: false,
            } if text == "true" => Some(true),
            Yaml::Scalar {
                text,
                quoted: false,
            } if text == "false" => Some(false),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            Yaml::Scalar {
                text,
                quoted: false,
            } if text.bytes().all(|b| b.is_ascii_digit()) => text.parse().ok(),
            _ => None,
        }
    }
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn is_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// Cuts a YAML comment: a `#` at the start or after a space, outside quotes.
fn strip_yaml_comment(line: &str) -> &str {
    let mut quote = None;
    let mut prev = ' ';
    for (i, c) in line.char_indices() {
        match (quote, c) {
            (None, '"' | '\'') => quote = Some(c),
            (Some(q), _) if c == q => quote = None,
            (None, '#') if prev == ' ' => return line[..i].trim_end(),
            _ => {}
        }
        prev = c;
    }
    line.trim_end()
}

/// Reads the block-style YAML subset that `ci.yml` uses.
/// Accepts mappings with bare keys, block sequences,
/// plain and quoted single-line scalars,
/// folded and literal block scalars, and flow sequences of plain scalars.
/// Panics on anchors, aliases, tags, merge keys, flow mappings, multi-line plain scalars,
/// document markers, and duplicate keys.
struct YamlReader {
    lines: Vec<String>,
    pos: usize,
}

impl YamlReader {
    fn parse(text: &str) -> Yaml {
        assert_plain_text("ci.yml", text);
        let mut reader = YamlReader {
            lines: text.lines().map(str::to_string).collect(),
            pos: 0,
        };
        reader.skip_blank();
        let root = reader.node(0);
        reader.skip_blank();
        assert!(
            reader.pos == reader.lines.len(),
            "ci.yml unsupported line {}: {}",
            reader.pos + 1,
            reader.lines[reader.pos]
        );
        root
    }

    fn skip_blank(&mut self) {
        while self.pos < self.lines.len() && is_blank_or_comment(&self.lines[self.pos]) {
            self.pos += 1;
        }
    }

    fn peek_indent(&mut self) -> Option<usize> {
        self.skip_blank();
        self.lines.get(self.pos).map(|l| indent_of(l))
    }

    fn is_seq_item(line: &str) -> bool {
        let trimmed = line.trim_start();
        trimmed == "-" || trimmed.starts_with("- ")
    }

    fn node(&mut self, indent: usize) -> Yaml {
        if Self::is_seq_item(&self.lines[self.pos]) {
            self.seq(indent)
        } else {
            self.map(indent)
        }
    }

    fn seq(&mut self, indent: usize) -> Yaml {
        let mut items = Vec::new();
        while self.peek_indent() == Some(indent) && Self::is_seq_item(&self.lines[self.pos]) {
            let line = self.lines[self.pos].clone();
            let rest = line.trim_start()[1..].trim_start();
            if rest.is_empty() || rest.starts_with('#') {
                self.pos += 1;
                let child = self
                    .peek_indent()
                    .filter(|i| *i > indent)
                    .unwrap_or_else(|| panic!("ci.yml empty sequence item: {line}"));
                items.push(self.node(child));
            } else if Self::split_key(strip_yaml_comment(rest)).is_some() {
                // Re-reads the item as a mapping that starts at the column after "- ".
                let column = indent + (line.trim_start().len() - rest.len());
                self.lines[self.pos] = format!("{}{rest}", " ".repeat(column));
                items.push(self.map(column));
            } else {
                self.pos += 1;
                items.push(Self::scalar(strip_yaml_comment(rest)));
            }
        }
        Yaml::Seq(items)
    }

    /// Splits `key: value` or `key:`;
    /// returns `None` when the line is not a mapping entry.
    fn split_key(line: &str) -> Option<(&str, &str)> {
        if let Some(key) = line.strip_suffix(':') {
            return Some((key, ""));
        }
        line.split_once(": ").map(|(k, v)| (k, v.trim()))
    }

    fn map(&mut self, indent: usize) -> Yaml {
        let mut entries: Vec<(String, Yaml)> = Vec::new();
        while let Some(current) = self.peek_indent() {
            if current < indent {
                break;
            }
            let line = self.lines[self.pos].clone();
            assert!(
                current == indent && !Self::is_seq_item(&line),
                "ci.yml unexpected line {}: {line}",
                self.pos + 1
            );
            let content = strip_yaml_comment(line.trim_start());
            let (key, value) = Self::split_key(content)
                .unwrap_or_else(|| panic!("ci.yml line is not key: value: {line}"));
            assert!(is_bare_key(key), "ci.yml unsupported key: {line}");
            assert!(
                entries.iter().all(|(k, _)| k != key),
                "ci.yml duplicate key {key} at line {}",
                self.pos + 1
            );
            self.pos += 1;
            let node = if value.is_empty() {
                match self.peek_indent() {
                    Some(child) if child > indent => self.node(child),
                    _ => Yaml::Null,
                }
            } else if matches!(value, ">" | ">-" | "|" | "|-") {
                self.block_scalar(indent, value.starts_with('>'))
            } else {
                Self::scalar(value)
            };
            entries.push((key.to_string(), node));
        }
        Yaml::Map(entries)
    }

    /// Joins the more-indented lines after `key: >` or `key: |`.
    /// Folding is approximated by joining with a space; the tests only search the text.
    fn block_scalar(&mut self, indent: usize, folded: bool) -> Yaml {
        let mut parts = Vec::new();
        while let Some(line) = self.lines.get(self.pos) {
            if !line.trim().is_empty() && indent_of(line) <= indent {
                break;
            }
            parts.push(line.trim().to_string());
            self.pos += 1;
        }
        let text = parts.join(if folded { " " } else { "\n" });
        Yaml::Scalar {
            text: text.trim().to_string(),
            quoted: false,
        }
    }

    fn scalar(value: &str) -> Yaml {
        let first = value.chars().next().unwrap_or(' ');
        assert!(
            !matches!(first, '&' | '*' | '!' | '{' | '|' | '>' | '%' | '@' | '`'),
            "ci.yml unsupported value: {value}"
        );
        if first == '[' {
            let inner = value
                .strip_prefix('[')
                .and_then(|v| v.strip_suffix(']'))
                .unwrap_or_else(|| panic!("ci.yml unsupported flow sequence: {value}"));
            let items = inner
                .split(',')
                .map(str::trim)
                .map(|item| {
                    assert!(
                        is_bare_key(item),
                        "ci.yml unsupported flow sequence item: {item}"
                    );
                    Yaml::Scalar {
                        text: item.to_string(),
                        quoted: false,
                    }
                })
                .collect();
            return Yaml::Seq(items);
        }
        for quote in ['"', '\''] {
            if first == quote {
                let inner = value
                    .strip_prefix(quote)
                    .and_then(|v| v.strip_suffix(quote))
                    .filter(|v| !v.contains(quote) && !v.contains('\\'))
                    .unwrap_or_else(|| panic!("ci.yml unsupported quoted value: {value}"));
                return Yaml::Scalar {
                    text: inner.to_string(),
                    quoted: true,
                };
            }
        }
        Yaml::Scalar {
            text: value.to_string(),
            quoted: false,
        }
    }
}

fn workflow_text() -> String {
    let path = repo_dir().join(".github/workflows/ci.yml");
    assert!(
        path.is_file(),
        "{} is missing; the CI checks of T-01 cannot run without it",
        path.display()
    );
    read(&path)
}

fn workflow() -> Yaml {
    YamlReader::parse(&workflow_text())
}

fn jobs(wf: &Yaml) -> &[(String, Yaml)] {
    wf.at("jobs").as_map().expect("ci.yml has a jobs mapping")
}

fn steps(job: &Yaml) -> &[Yaml] {
    job.at("steps").as_seq().unwrap_or(&[])
}

fn run_text(step: &Yaml) -> &str {
    step.at("run").as_str().unwrap_or("")
}

// --- Pinning predicates ---

fn is_lower_hex(text: &str, len: usize) -> bool {
    text.len() == len
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn is_sha256_pinned(image: &str) -> bool {
    image
        .split_once("@sha256:")
        .is_some_and(|(_, digest)| is_lower_hex(digest, 64))
}

/// Matches `<name>:<major>.<minor>.<patch>-<variant>@sha256:<64 hex>`,
/// so the image carries both a toolchain version and a digest.
fn is_version_and_digest_pinned(image: &str) -> bool {
    let Some((name_tag, _)) = image.split_once("@sha256:") else {
        return false;
    };
    let Some((_, tag)) = name_tag.rsplit_once(':') else {
        return false;
    };
    let Some((version, variant)) = tag.split_once('-') else {
        return false;
    };
    let parts: Vec<&str> = version.split('.').collect();
    is_sha256_pinned(image)
        && parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        && !variant.is_empty()
        && variant
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Matches `owner/repo@<40 hex>`, a full commit SHA that a moved tag cannot change.
fn is_sha_pinned_action(uses: &str) -> bool {
    let Some((path, sha)) = uses.split_once('@') else {
        return false;
    };
    let segments: Vec<&str> = path.split('/').collect();
    segments.len() == 2
        && segments.iter().all(|s| {
            !s.is_empty()
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        })
        && is_lower_hex(sha, 40)
}

fn read_only_permissions(value: &Yaml) -> bool {
    value
        .as_map()
        .is_some_and(|m| m.len() == 1 && value.at("contents").as_str() == Some("read"))
}

// --- GEDT-AR-00025 / T-01 AC4: deny.toml denies every finding ---

#[test]
fn test_ar_00025_ac4_deny_advisories_deny_yanked_and_unmaintained() {
    let cfg = deny_toml();
    let advisories = table(&cfg, "advisories");
    assert_eq!(toml_raw(advisories, "version"), Some("2"));
    assert_eq!(toml_str(advisories, "yanked"), Some("deny"));
    assert_eq!(toml_str(advisories, "unmaintained"), Some("all"));
}

// GEDT-AR-00025, T-54 R-T54-01:
// cargo-deny keeps its advisory databases in the container /tmp.
// An earlier container could edit a database in the writable .cargo-home,
// which would then hide advisories from a later build.
// db-path names ADVISORY_DB_DIR, which build.sh and the CI deny step set;
// cargo-deny fails when the variable is unset, so no run falls back to another path.
// TEST_JOB_COMMANDS pins the CI commands, including their database paths.
#[test]
fn test_ar_00025_t54_deny_advisory_db_in_container_tmp() {
    for (file, cfg) in [
        ("deny.toml", deny_toml()),
        ("deny-gui_editor.toml", gui_deny_toml()),
    ] {
        assert_eq!(
            toml_str(table(&cfg, "advisories"), "db-path"),
            Some("$ADVISORY_DB_DIR/cargo-deny"),
            "{file} advisories.db-path does not use ADVISORY_DB_DIR"
        );
    }
    for (tool, db_arg) in [
        ("cargo-audit", " --db /tmp/advisory-dbs/cargo-audit "),
        ("cargo-deny", " env ADVISORY_DB_DIR=/tmp/advisory-dbs "),
    ] {
        let command = TEST_JOB_COMMANDS
            .iter()
            .find(|c| c.contains(&format!("/usr/local/cargo/bin/{tool} ")))
            .unwrap_or_else(|| panic!("no CI command runs {tool}"));
        assert!(
            command.contains(db_arg),
            "the CI {tool} command does not keep its database in /tmp: {command}"
        );
    }
}

#[test]
fn test_ar_00025_ac4_deny_advisories_ignore_empty() {
    let cfg = deny_toml();
    assert!(
        toml_array_is_empty(table(&cfg, "advisories"), "ignore"),
        "advisories.ignore is not empty"
    );
}

#[test]
fn test_ar_00025_ac4_deny_bans_deny_multiple_versions_and_wildcards() {
    let cfg = deny_toml();
    let bans = table(&cfg, "bans");
    assert_eq!(toml_str(bans, "multiple-versions"), Some("deny"));
    assert_eq!(toml_str(bans, "wildcards"), Some("deny"));
    assert!(toml_array_is_empty(bans, "skip"), "bans.skip is not empty");
    assert!(
        toml_array_is_empty(bans, "skip-tree"),
        "bans.skip-tree is not empty"
    );
}

#[test]
fn test_ar_00025_ac4_deny_sources_crates_io_only() {
    let cfg = deny_toml();
    let sources = table(&cfg, "sources");
    assert_eq!(toml_str(sources, "unknown-registry"), Some("deny"));
    assert_eq!(toml_str(sources, "unknown-git"), Some("deny"));
    assert!(
        toml_array_is_empty(sources, "allow-git"),
        "sources.allow-git is not empty"
    );
    let registries = toml_string_array(sources, "allow-registry");
    assert!(
        registries.iter().all(|r| r == CRATES_IO_INDEX),
        "registry other than crates.io: {registries:?}"
    );
}

/// Licenses the product owner approved for licenses.allow in both policy files, sorted.
/// T-42 added Unicode-3.0 (unicode-ident) and Zlib (foldhash) to Apache-2.0 and MIT.
/// T-02 added ISC (libloading).
/// Any other license needs a new decision.
const APPROVED_LICENSES: [&str; 5] = ["Apache-2.0", "ISC", "MIT", "Unicode-3.0", "Zlib"];

/// The one license exception the product owner approved for T-02:
/// the font files that epaint_default_fonts embeds, in deny-gui_editor.toml only.
const APPROVED_EXCEPTION_CRATE: &str = "epaint_default_fonts";
const APPROVED_EXCEPTION_LICENSES: [&str; 2] = ["OFL-1.1", "Ubuntu-font-1.0"];

/// Strips the quotes of a raw basic string value, or returns `None` for any other value.
fn unquote(raw: &str) -> Option<&str> {
    raw.strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .filter(|v| !v.contains('"'))
}

/// Checks licenses.exceptions against the approved entry.
/// With `expect_fonts`, the list must hold exactly the approved entry;
/// without it, the list must be empty.
/// Returns the first deviation, so a test can also check that deviations are rejected.
fn check_license_exceptions(licenses: &TomlTable, expect_fonts: bool) -> Result<(), String> {
    let entries = toml_inline_table_array(licenses, "exceptions");
    if !expect_fonts {
        if entries.is_empty() {
            return Ok(());
        }
        return Err(format!("unexpected licenses.exceptions: {entries:?}"));
    }
    let [fields] = entries.as_slice() else {
        return Err(format!("licenses.exceptions is not one entry: {entries:?}"));
    };
    let keys: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
    if keys != ["crate", "allow"] {
        return Err(format!("exception keys are not crate and allow: {keys:?}"));
    }
    if unquote(&fields[0].1) != Some(APPROVED_EXCEPTION_CRATE) {
        return Err(format!("exception crate {}", fields[0].1));
    }
    if parse_string_array("allow", &fields[1].1) != APPROVED_EXCEPTION_LICENSES {
        return Err(format!("exception licenses {}", fields[1].1));
    }
    Ok(())
}

// GEDT-AR-00025, T-02: both policy files allow only approved licenses,
// and only deny-gui_editor.toml holds the single approved font exception.
#[test]
fn test_ar_00025_ac4_deny_licenses_approved_only() {
    for (name, cfg, expect_fonts) in [
        ("deny.toml", deny_toml(), false),
        ("deny-gui_editor.toml", gui_deny_toml(), true),
    ] {
        let licenses = table(&cfg, "licenses");
        for license in toml_string_array(licenses, "allow") {
            assert!(
                APPROVED_LICENSES.contains(&license.as_str()),
                "{name} licenses.allow has {license}"
            );
        }
        if let Err(problem) = check_license_exceptions(licenses, expect_fonts) {
            panic!("{name}: {problem}");
        }
    }
}

// GEDT-AR-00025, T-02: the exception check rejects every entry but the approved one.
#[test]
fn test_ar_00025_t02_license_exception_check_rejects_others() {
    let approved = "{ crate = \"epaint_default_fonts\", allow = [\"OFL-1.1\", \"Ubuntu-font-1.0\"] }";
    let licenses = |exceptions: &str| {
        parse_toml(&format!("[licenses]\nexceptions = [\n{exceptions}\n]\n"))
            .pop()
            .expect("one table")
    };
    assert_eq!(check_license_exceptions(&licenses(approved), true), Ok(()));
    let rejected = [
        String::new(),
        format!("{approved},\n{approved}"),
        approved.replace("epaint_default_fonts", "ring"),
        approved.replace("\"Ubuntu-font-1.0\"", "\"Ubuntu-font-1.0\", \"GPL-3.0\""),
        approved.replace("\"OFL-1.1\", ", ""),
        approved.replace(" }", ", version = \"*\" }"),
    ];
    for exceptions in &rejected {
        assert!(
            check_license_exceptions(&licenses(exceptions), true).is_err(),
            "accepted {exceptions}"
        );
    }
    assert!(check_license_exceptions(&licenses(approved), false).is_err());
    assert_eq!(check_license_exceptions(&licenses(""), false), Ok(()));
}

/// Returns the raw values of every `license` key in the `[package]` table of a manifest.
/// A `license` key in another table, such as a dependency table, does not count.
fn package_license_values(manifest: &str) -> Vec<String> {
    package_values(manifest, "license")
}

/// Returns the raw values of every `wanted` key in the `[package]` table of a manifest.
fn package_values(manifest: &str, wanted: &str) -> Vec<String> {
    let mut in_package = false;
    let mut values = Vec::new();
    for raw in manifest.lines() {
        let line = strip_toml_comment(raw).trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == wanted {
                values.push(value.trim().to_string());
            }
        }
    }
    values
}

// GEDT-STORY-00019, T-42: every crate the CI gate checks declares the repository license,
// so `cargo deny check licenses` never reports a workspace crate as unlicensed.
#[test]
fn test_ar_00025_ac4_crates_declare_apache_license() {
    for krate in CI_CRATES {
        let path = repo_dir().join(krate).join("Cargo.toml");
        let values = package_license_values(&read(&path));
        assert_eq!(
            values,
            ["\"Apache-2.0\""],
            "{} [package] license is not exactly \"Apache-2.0\"",
            path.display()
        );
    }
}

// GEDT-STORY-00019, T-42: Zlib and Unicode-3.0 must be present, not only permitted,
// and the list holds no duplicate, so a lost entry fails here before `cargo deny` runs.
#[test]
fn test_ar_00025_ac4_deny_licenses_allow_is_approved_set() {
    let cfg = deny_toml();
    let mut allow = toml_string_array(table(&cfg, "licenses"), "allow");
    let listed = allow.len();
    allow.sort();
    allow.dedup();
    assert_eq!(listed, allow.len(), "licenses.allow has duplicates");
    assert_eq!(allow, APPROVED_LICENSES, "licenses.allow");
}

// GEDT-AR-00025, T-42: a lower confidence threshold weakens the license check,
// because cargo-deny then accepts a license text it only partly recognises.
#[test]
fn test_ar_00025_ac4_deny_licenses_confidence_threshold_kept() {
    let cfg = deny_toml();
    let raw = toml_raw(table(&cfg, "licenses"), "confidence-threshold")
        .expect("licenses.confidence-threshold is set");
    let threshold: f64 = raw
        .parse()
        .unwrap_or_else(|err| panic!("confidence-threshold {raw}: {err}"));
    assert!(
        threshold >= 0.93,
        "licenses.confidence-threshold {threshold} is below 0.93"
    );
}

// GEDT-STORY-00019, T-42: checks the manifest reader on synthetic manifests,
// so the crate license test cannot pass on a missing, doubled, or misplaced key.
#[test]
fn test_ar_00025_ac4_package_license_values_reads_package_table_only() {
    let one = "[package]\nname = \"a\"\nlicense = \"Apache-2.0\" # repo\n";
    assert_eq!(package_license_values(one), ["\"Apache-2.0\""]);

    let missing = "[package]\nname = \"a\"\n[dependencies]\nx = \"1\"\n";
    assert!(
        package_license_values(missing).is_empty(),
        "missing license"
    );

    let twice = "[package]\nlicense = \"Apache-2.0\"\nlicense = \"MIT\"\n";
    assert_eq!(package_license_values(twice).len(), 2, "second license");

    let other_table = "[package]\nname = \"a\"\n[package.metadata]\nlicense = \"Apache-2.0\"\n\
                       [dependencies.x]\nlicense = \"Apache-2.0\"\n";
    assert!(
        package_license_values(other_table).is_empty(),
        "license outside [package]"
    );

    let before_package = "license = \"Apache-2.0\"\n[package]\nname = \"a\"\n";
    assert!(
        package_license_values(before_package).is_empty(),
        "license before [package]"
    );

    let workspace = "[package]\nlicense.workspace = true\n";
    assert!(
        package_license_values(workspace).is_empty(),
        "inherited license is not a literal Apache-2.0"
    );

    let commented = "[package]\n# license = \"Apache-2.0\"\n";
    assert!(
        package_license_values(commented).is_empty(),
        "commented-out license"
    );

    let dual = "[package]\nlicense = \"Apache-2.0 OR GPL-3.0\"\n";
    assert_ne!(
        package_license_values(dual),
        ["\"Apache-2.0\""],
        "dual license"
    );
}

#[test]
fn test_ar_00025_ac4_deny_graph_keeps_dev_dependencies() {
    let cfg = deny_toml();
    let excluded = cfg
        .iter()
        .find(|t| t.name == "graph")
        .and_then(|graph| toml_raw(graph, "exclude-dev"));
    assert!(
        excluded.is_none_or(|v| v == "false"),
        "graph.exclude-dev hides dev-dependencies from cargo deny: {excluded:?}"
    );
}

// Product owner decision for T-01: the license policy covers dev-dependencies too.
#[test]
fn test_ar_00025_ac4_deny_licenses_include_dev() {
    let cfg = deny_toml();
    assert_eq!(
        toml_bool(table(&cfg, "licenses"), "include-dev"),
        Some(true),
        "licenses.include-dev is not true"
    );
}

/// Tables and keys that `deny.toml` may hold, taken from the committed file.
/// Any other key can weaken a check without a RUSTSEC ID or a level change in sight,
/// for example `graph.exclude`, `licenses.private`, or `bans.allow`.
/// Adding a key here is a policy change that needs review
/// (product owner decision for T-01).
/// T-02 added `graph.targets` and `licenses.exceptions` by product owner decision.
/// T-54 added `advisories.db-path` by product owner decision for R-T54-01;
/// `test_ar_00025_t54_deny_advisory_db_in_container_tmp` pins its value.
const DENY_ALLOWED_KEYS: [(&str, &[&str]); 5] = [
    (
        "advisories",
        &["db-path", "ignore", "unmaintained", "version", "yanked"],
    ),
    ("bans", &["multiple-versions", "skip", "wildcards"]),
    ("graph", &["all-features", "targets"]),
    (
        "licenses",
        &[
            "allow",
            "confidence-threshold",
            "exceptions",
            "include-dev",
            "unused-allowed-license",
            "version",
        ],
    ),
    (
        "sources",
        &[
            "allow-git",
            "allow-registry",
            "unknown-git",
            "unknown-registry",
        ],
    ),
];

#[test]
fn test_ar_00025_ac4_deny_keys_allow_listed() {
    for (file, cfg) in [
        ("deny.toml", deny_toml()),
        ("deny-gui_editor.toml", gui_deny_toml()),
    ] {
        for section in &cfg {
            let allowed = DENY_ALLOWED_KEYS
                .iter()
                .find(|(name, _)| *name == section.name)
                .map(|(_, keys)| *keys)
                .unwrap_or_else(|| panic!("{file} table [{}] is not allow-listed", section.name));
            for (key, _) in &section.entries {
                assert!(
                    allowed.contains(&key.as_str()),
                    "{file} key {}.{key} is not allow-listed",
                    section.name
                );
            }
        }
    }
}

// GEDT-AR-00004, T-02: the policies check the Fedora x86_64 graph only.
// Another target list could hide or add platform-specific crates without a review.
#[test]
fn test_ar_00025_t02_deny_graph_targets_linux_only() {
    for (file, cfg) in [
        ("deny.toml", deny_toml()),
        ("deny-gui_editor.toml", gui_deny_toml()),
    ] {
        assert_eq!(
            toml_string_array(table(&cfg, "graph"), "targets"),
            ["x86_64-unknown-linux-gnu"],
            "{file} graph.targets"
        );
    }
}

// --- GEDT-AR-00025, T-02: deny-gui_editor.toml and deny.toml do not drift ---

/// Replaces the array value of `key` in `[table]` with `[]` and keeps every other line,
/// including comments, so only that one value can differ after the replacement.
fn empty_array_value(text: &str, table: &str, key: &str) -> String {
    let header = format!("[{table}]");
    let prefix = format!("{key} = [");
    let mut in_table = false;
    let mut out = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.starts_with('[') {
            in_table = line == header;
        }
        if in_table && line.starts_with(&prefix) {
            let mut depth = bracket_depth(strip_toml_comment(line));
            while depth > 0 {
                let next = lines
                    .next()
                    .unwrap_or_else(|| panic!("{table}.{key} is not closed"));
                depth += bracket_depth(strip_toml_comment(next));
            }
            out.push(format!("{key} = []"));
            continue;
        }
        out.push(line.to_string());
    }
    out.join("\n") + "\n"
}

/// deny-gui_editor.toml with its gui_editor-only values emptied.
fn gui_deny_text_without_own_entries() -> String {
    let gui = read(&gui_dir().join("deny-gui_editor.toml"));
    let gui = empty_array_value(&gui, "licenses", "exceptions");
    empty_array_value(&gui, "bans", "skip")
}

// GEDT-AR-00025, T-02 security constraint:
// the policy files differ only in licenses.exceptions and bans.skip,
// which cargo-deny would report as unused in the other crates;
// every other setting and comment is identical.
#[test]
fn test_ar_00025_t02_deny_policies_differ_only_in_gui_entries() {
    let shared = read(&gui_dir().join("deny.toml"));
    let gui = gui_deny_text_without_own_entries();
    if gui != shared {
        let first = gui
            .lines()
            .zip(shared.lines())
            .position(|(a, b)| a != b)
            .map_or("length".to_string(), |i| format!("line {}", i + 1));
        panic!("deny-gui_editor.toml differs from deny.toml outside its own entries at {first}");
    }
}

// GEDT-AR-00025, T-02: the drift normaliser empties only the named value.
#[test]
fn test_ar_00025_t02_empty_array_value_keeps_other_lines() {
    let text = "[bans]\nskip = [\n    { crate = \"a@1\", reason = \"[x]\" },\n]\n\
                wildcards = \"allow\"\n[other]\nskip = [\"b\"]\n";
    assert_eq!(
        empty_array_value(text, "bans", "skip"),
        "[bans]\nskip = []\nwildcards = \"allow\"\n[other]\nskip = [\"b\"]\n"
    );
}

/// Duplicates in deny-gui_editor.toml bans.skip, approved by the product owner for T-02.
/// Measured with cargo-deny 0.22.0 on the x86_64-unknown-linux-gnu graph of eframe 0.37.
/// Each entry goes in T-43.
const APPROVED_GUI_SKIPS: [&str; 5] = [
    "linux-raw-sys@0.4.15",
    "rustix@0.38.44",
    "syn@2.0.119",
    "thiserror-impl@1.0.69",
    "thiserror@1.0.69",
];

// GEDT-AR-00025, T-02: every gui_editor skip entry is approved,
// has a reason naming the eframe cause and its removal task, and nothing else is skipped.
#[test]
fn test_ar_00025_t02_gui_bans_skip_is_approved_set_with_reasons() {
    let cfg = gui_deny_toml();
    let bans = table(&cfg, "bans");
    assert!(
        toml_array_is_empty(bans, "skip-tree"),
        "bans.skip-tree is not empty"
    );
    let mut crates = Vec::new();
    for fields in toml_inline_table_array(bans, "skip") {
        let keys: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["crate", "reason"], "skip entry keys: {fields:?}");
        let name = unquote(&fields[0].1).unwrap_or_else(|| panic!("crate {}", fields[0].1));
        let reason = unquote(&fields[1].1).unwrap_or_else(|| panic!("reason {}", fields[1].1));
        for needle in ["eframe 0.37", "remove in T-43"] {
            assert!(reason.contains(needle), "{name} reason lacks {needle}");
        }
        crates.push(name.to_string());
    }
    crates.sort();
    assert_eq!(crates, APPROVED_GUI_SKIPS, "deny-gui_editor.toml bans.skip");
}

// --- GEDT-AR-00025, T-52: path dependencies carry a version ---

fn is_dependency_table(header: &str) -> bool {
    let last = header.rsplit('.').next().unwrap_or(header);
    matches!(
        last,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    )
}

/// Returns the top-level keys and raw values of a one-line inline table.
/// Example: `{ path = "..", x = [1, 2] }`.
/// Panics on literal strings and escapes, which the reader cannot see through.
fn inline_table_fields(value: &str) -> Vec<(String, String)> {
    let inner = value
        .trim()
        .strip_prefix('{')
        .and_then(|v| v.strip_suffix('}'))
        .unwrap_or_else(|| panic!("not a one-line inline table: {value}"));
    assert!(
        !inner.contains(['\'', '\\']),
        "inline table literal strings and escapes are not supported: {value}"
    );
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_string = false;
    let mut depth = 0;
    for c in inner.chars().chain(std::iter::once(',')) {
        match c {
            '"' => in_string = !in_string,
            '[' | '{' if !in_string => depth += 1,
            ']' | '}' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                if let Some((key, raw)) = field.split_once('=') {
                    fields.push((key.trim().to_string(), raw.trim().to_string()));
                }
                field.clear();
                continue;
            }
            _ => {}
        }
        field.push(c);
    }
    fields
}

fn inline_table_keys(value: &str) -> Vec<String> {
    inline_table_fields(value)
        .into_iter()
        .map(|(k, _)| k)
        .collect()
}

/// Returns the names of path dependencies without a `version` key,
/// in every dependency table.
/// `cargo deny` reports such a dependency as a wildcard,
/// which `bans.wildcards = "deny"` fails.
/// Panics on `[dependencies.name]` tables and multi-line values,
/// so a form the reader cannot see through fails the test instead of hiding a dependency.
fn path_dependencies_without_version(manifest: &str) -> Vec<String> {
    let mut in_dependencies = false;
    let mut missing = Vec::new();
    for raw in manifest.lines() {
        let line = strip_toml_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(header) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            let header = header.trim();
            assert!(
                !header.split('.').rev().skip(1).any(is_dependency_table),
                "dependency tables of the form [{header}] are not supported"
            );
            in_dependencies = is_dependency_table(header);
            continue;
        }
        if !in_dependencies {
            continue;
        }
        let (name, value) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("dependency line is not name = value: {raw}"));
        let value = value.trim();
        assert_eq!(bracket_depth(value), 0, "multi-line dependency: {raw}");
        if !value.starts_with('{') {
            continue;
        }
        let keys = inline_table_keys(value);
        if keys.iter().any(|k| k == "path") && !keys.iter().any(|k| k == "version") {
            missing.push(name.trim().to_string());
        }
    }
    missing
}

// GEDT-STORY-00019, T-52: a `path` dependency without `version` is a cargo-deny wildcard.
// `cargo deny check bans` fails on `wildcards = "deny"` for every crate that has one.
// The product owner extended the guard to aws2xml, which CI does not gate.
#[test]
fn test_ar_00025_t52_ci_crates_path_dependencies_have_version() {
    for krate in CI_CRATES.iter().chain(&VERSION_GUARDED_NON_CI_CRATES) {
        let path = repo_dir().join(krate).join("Cargo.toml");
        let missing = path_dependencies_without_version(&read(&path));
        assert!(
            missing.is_empty(),
            "{} path dependencies without version: {missing:?}",
            path.display()
        );
    }
}

// GEDT-STORY-00019, T-52: sad2md and tui_editor pin `sad_xml_sql` by path and version.
// The T-52 definition of done requires the versioned path form.
// aws2xml joins by product owner decision.
// The version equals the sad_xml_sql package version,
// so the requirement matches the path crate.
#[test]
fn test_ar_00025_t52_sad_xml_sql_dependency_is_versioned_path() {
    let package = package_values(&read(&repo_dir().join("sad_xml_sql/Cargo.toml")), "version");
    assert_eq!(
        package.len(),
        1,
        "sad_xml_sql [package] version: {package:?}"
    );
    for krate in ["aws2xml", "sad2md", "tui_editor"] {
        let manifest = read(&repo_dir().join(krate).join("Cargo.toml"));
        let line = manifest
            .lines()
            .map(|l| strip_toml_comment(l).trim())
            .find(|l| {
                l.split_once('=')
                    .is_some_and(|(k, _)| k.trim() == "sad_xml_sql")
            })
            .unwrap_or_else(|| panic!("{krate} has no sad_xml_sql dependency"));
        let fields = inline_table_fields(line.split_once('=').expect("name = value").1);
        let value = |key: &str| {
            fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
        };
        assert!(value("path").is_some(), "{krate}: {line}");
        assert_eq!(
            value("version"),
            Some(package[0].as_str()),
            "{krate}: {line}"
        );
    }
}

// GEDT-STORY-00019, T-52: checks the path dependency reader on synthetic manifests,
// so the version test cannot pass on a missed table, key, or comment.
#[test]
fn test_ar_00025_t52_path_dependencies_without_version_reader() {
    let versioned = "[dependencies]\na = { path = \"../a\", version = \"0.1.0\" }\n";
    assert!(path_dependencies_without_version(versioned).is_empty());

    let bare = "[dependencies]\na = { path = \"../a\" }\nb = \"1\"\n";
    assert_eq!(path_dependencies_without_version(bare), ["a"]);

    let dev = "[package]\nname = \"x\"\n[dev-dependencies]\na = { path = \"../a\" }\n";
    assert_eq!(path_dependencies_without_version(dev), ["a"], "dev");

    let target = "[target.'cfg(unix)'.dependencies]\na = { path = \"../a\" }\n";
    assert_eq!(path_dependencies_without_version(target), ["a"], "target");

    let commented = "[dependencies]\na = { path = \"../a\" } # version = \"0.1.0\"\n";
    assert_eq!(
        path_dependencies_without_version(commented),
        ["a"],
        "comment"
    );

    let in_string = "[dependencies]\na = { path = \"../version = 1\" }\n";
    assert_eq!(
        path_dependencies_without_version(in_string),
        ["a"],
        "string"
    );

    let nested = "[dependencies]\na = { path = \"../a\", features = [\"version\"] }\n";
    assert_eq!(path_dependencies_without_version(nested), ["a"], "array");

    let outside = "[package]\nname = \"x\"\n[features]\nversion = []\n";
    assert!(path_dependencies_without_version(outside).is_empty());

    let dotted = "[dependencies.a]\npath = \"../a\"\n";
    let result = std::panic::catch_unwind(|| path_dependencies_without_version(dotted));
    assert!(result.is_err(), "dotted dependency table is not rejected");

    for unsupported in [
        "[dependencies]\na = { path = '../a' }\n",
        "[dependencies]\na = { path = \"../a\\\" version = 1\" }\n",
    ] {
        let result = std::panic::catch_unwind(|| path_dependencies_without_version(unsupported));
        assert!(
            result.is_err(),
            "unsupported string not rejected: {unsupported}"
        );
    }

    let fields = inline_table_fields("{ path = \"../a\", version = \"0.1.0\" }");
    assert_eq!(
        fields,
        [
            ("path".to_string(), "\"../a\"".to_string()),
            ("version".to_string(), "\"0.1.0\"".to_string()),
        ]
    );
}

// --- CI workflow structure: GEDT-AR-00024, GEDT-AR-00025, GEDT-AR-00030, AC5, AC6 ---

#[test]
fn test_ar_00024_ar_00025_ar_00030_ci_jobs_present() {
    let wf = workflow();
    for name in ["secret-scan", "gui-editor-build", "test"] {
        assert!(
            jobs(&wf).iter().any(|(k, _)| k == name),
            "missing job {name}"
        );
    }
}

#[test]
fn test_ar_00024_ci_permissions_read_only() {
    let wf = workflow();
    assert!(
        read_only_permissions(wf.at("permissions")),
        "top-level permissions are not contents: read"
    );
    for (name, job) in jobs(&wf) {
        if let Some(permissions) = job.get("permissions") {
            assert!(
                read_only_permissions(permissions),
                "{name} widens permissions"
            );
        }
    }
}

#[test]
fn test_ar_00025_ci_actions_pinned_by_sha() {
    let wf = workflow();
    for (name, job) in jobs(&wf) {
        for step in steps(job) {
            if let Some(uses) = step.get("uses") {
                let uses = uses.as_str().unwrap_or_default();
                assert!(
                    is_sha_pinned_action(uses),
                    "{name} action not SHA-pinned: {uses}"
                );
            }
        }
        if let Some(uses) = job.get("uses") {
            panic!("{name} calls a reusable workflow: {uses:?}");
        }
    }
}

#[test]
fn test_ar_00024_ci_checkout_does_not_persist_credentials() {
    let wf = workflow();
    for (name, job) in jobs(&wf) {
        for step in steps(job) {
            if step
                .at("uses")
                .as_str()
                .is_some_and(|u| u.starts_with("actions/checkout@"))
            {
                assert_eq!(
                    step.at("with").at("persist-credentials").as_bool(),
                    Some(false),
                    "{name} checkout persists credentials"
                );
            }
        }
    }
}

#[test]
fn test_ar_00024_ci_run_steps_do_not_interpolate_expressions() {
    let wf = workflow();
    for (name, job) in jobs(&wf) {
        for step in steps(job) {
            assert!(
                !run_text(step).contains("${{"),
                "{name} interpolates expressions into run"
            );
        }
    }
}

/// Engine options of `docker run` that take no value, as used in `ci.yml`.
/// Any other option is assumed to take one value unless written as `--option=value`.
/// An unknown flag therefore shifts the parse onto a non-image token and fails the test.
const ENGINE_FLAGS_WITHOUT_VALUE: [&str; 6] = ["--rm", "-i", "-t", "-it", "--init", "--read-only"];

/// Returns the image argument of every `docker run`, `docker create`, `podman run`,
/// and `podman create` in a run text.
/// A request without an image yields an empty string, which no pinning check accepts.
fn engine_run_images(run: &str) -> Vec<String> {
    let tokens: Vec<&str> = run.split_whitespace().collect();
    let mut images = Vec::new();
    for (i, pair) in tokens.windows(2).enumerate() {
        if !(matches!(pair[0], "docker" | "podman") && matches!(pair[1], "run" | "create")) {
            continue;
        }
        let mut pos = i + 2;
        while let Some(token) = tokens.get(pos) {
            if *token == "--" {
                pos += 1;
                break;
            }
            if !token.starts_with('-') {
                break;
            }
            let takes_value = !ENGINE_FLAGS_WITHOUT_VALUE.contains(token) && !token.contains('=');
            pos += if takes_value { 2 } else { 1 };
        }
        images.push(tokens.get(pos).copied().unwrap_or("").to_string());
    }
    images
}

/// Matches `<name>:<tag>@sha256:<64 hex>`, a digest pin with a visible version tag.
fn is_tag_and_digest_pinned(image: &str) -> bool {
    image
        .split_once("@sha256:")
        .and_then(|(name_tag, _)| name_tag.rsplit_once(':'))
        .is_some_and(|(name, tag)| !name.is_empty() && !tag.is_empty() && !tag.contains('/'))
        && is_sha256_pinned(image)
}

// GEDT-AR-00025 AC2: every image in ci.yml is pinned, wherever the workflow names one.
// GEDT-AR-00034, T-54: no job declares a job container;
// the cargo steps run in the tool image, whose base the Containerfile tests pin.
#[test]
fn test_ar_00025_ac2_ci_images_pinned_by_digest() {
    let wf = workflow();
    for (name, job) in jobs(&wf) {
        assert!(
            job.get("services").is_none(),
            "{name} declares services, whose images this test does not pin"
        );
        assert!(
            job.get("container").is_none(),
            "{name} declares a container instead of using the tool image"
        );
        for step in steps(job) {
            let run = run_text(step);
            for image in run
                .split_whitespace()
                .filter(|t| t.starts_with("docker.io/"))
            {
                assert!(is_sha256_pinned(image), "{name} unpinned image {image}");
            }
            for image in engine_run_images(run) {
                assert!(
                    is_tag_and_digest_pinned(&image),
                    "{name} runs an unpinned image: {image:?}"
                );
            }
        }
    }
}

// GEDT-AR-00025 AC2: the run-text image finder sees images outside docker.io/ too.
#[test]
fn test_ar_00025_ac2_engine_run_images_finds_every_image() {
    let pinned = format!("docker.io/x/gitleaks:v8.30.1@sha256:{}", "a".repeat(64));
    let cases = [
        ("docker run --rm alpine:latest sh", "alpine:latest"),
        ("docker run -v a:/b -e K=V --user 1:1 alpine sh", "alpine"),
        ("podman create --name=x ghcr.io/o/i:1 sh", "ghcr.io/o/i:1"),
        ("docker run --rm -- alpine", "alpine"),
        ("docker run --rm", ""),
    ];
    for (run, want) in cases {
        let images = engine_run_images(run);
        assert_eq!(images, [want], "{run}");
        assert!(!is_tag_and_digest_pinned(want), "{want} counted as pinned");
    }
    let run =
        format!("docker run --rm --user \"$(id -u):$(id -g)\" -v \"$PWD\":/repo:ro {pinned} git");
    assert_eq!(engine_run_images(&run), [pinned.as_str()]);
    assert!(is_tag_and_digest_pinned(&pinned));
    assert!(!is_tag_and_digest_pinned(&format!(
        "docker.io/x/gitleaks@sha256:{}",
        "a".repeat(64)
    )));
}

#[test]
fn test_ar_00024_ac5_secret_scan_fails_on_findings_with_redaction() {
    let wf = workflow();
    let scan_steps = steps(wf.at("jobs").at("secret-scan"));
    let scan: Vec<&str> = scan_steps.iter().map(run_text).collect();
    let scan = scan.join(" ");
    for needle in ["gitleaks", "--exit-code 1", "--redact"] {
        assert!(scan.contains(needle), "secret-scan lacks {needle}");
    }
    assert!(
        scan_steps
            .iter()
            .any(|s| s.at("with").at("fetch-depth").as_u64() == Some(0)),
        "secret-scan lacks full history"
    );
}

#[test]
fn test_ar_00025_ac4_ci_runs_pipeline_tests_before_build_sh() {
    let wf = workflow();
    let runs: Vec<&str> = steps(wf.at("jobs").at("gui-editor-build"))
        .iter()
        .map(run_text)
        .collect();
    let pipeline_at = runs
        .iter()
        .position(|r| r.contains("tests/pipeline/test_build_pipeline.sh"));
    let build_at = runs.iter().position(|r| r.contains("gui_editor/build.sh"));
    assert!(build_at.is_some(), "gui-editor-build does not run build.sh");
    assert!(
        pipeline_at.is_some() && pipeline_at < build_at,
        "gui-editor-build does not run the pipeline tests before build.sh"
    );
}

#[test]
fn test_ar_00030_ac6_ci_tests_four_crates_without_fail_fast() {
    let wf = workflow();
    let test = wf.at("jobs").at("test");
    let mut crates: Vec<&str> = test
        .at("strategy")
        .at("matrix")
        .at("crate")
        .as_seq()
        .unwrap_or(&[])
        .iter()
        .filter_map(Yaml::as_str)
        .collect();
    crates.sort_unstable();
    assert_eq!(crates, CI_CRATES, "test matrix crates");
    assert_eq!(
        test.at("strategy").at("fail-fast").as_bool(),
        Some(false),
        "test fail-fast"
    );
    let last_run = steps(test)
        .iter()
        .rev()
        .find(|s| s.get("run").is_some())
        .expect("test job has a run step");
    assert_eq!(
        run_text(last_run).trim(),
        "bash gui_editor/container/tool_image.sh run \"$CRATE\" cargo test --locked"
    );
    assert!(
        last_run.get("working-directory").is_none(),
        "the test step sets a working directory; tool_image.sh takes the crate directory"
    );
}

/// Reads `NAME="value"` from `container/tool_image.sh`,
/// which pins the base image and the tool versions for build.sh and CI.
fn tool_image_sh_assignment(name: &str) -> String {
    let text = read(&gui_dir().join("container/tool_image.sh"));
    let prefix = format!("{name}=\"");
    text.lines()
        .find_map(|l| l.strip_prefix(&prefix)?.strip_suffix('"'))
        .unwrap_or_else(|| panic!("tool_image.sh has no {name}=\"...\" line"))
        .to_string()
}

/// The `test` job commands in order.
/// The first command builds and checks the tool image (GEDT-AR-00034, T-54).
/// Every crate then passes the ignore-list check, the `$CARGO_HOME/audit.toml` guard,
/// `cargo audit`, and `cargo deny` in the tool image, before any of its code compiles.
/// Both checks run the tools by absolute path, so no cargo `[alias]` can replace them.
const TEST_JOB_COMMANDS: [&str; 8] = [
    "bash gui_editor/container/tool_image.sh build",
    "bash gui_editor/check_advisory_ignores.sh",
    "bash gui_editor/container/tool_image.sh run . sh -c 'test ! -e \"${CARGO_HOME:?}/audit.toml\"'",
    "bash gui_editor/container/tool_image.sh run gui_editor /usr/local/cargo/bin/cargo-audit audit --db /tmp/advisory-dbs/cargo-audit --deny warnings --file \"../$CRATE/Cargo.lock\"",
    "deny_config=../gui_editor/deny.toml",
    "if [ \"$CRATE\" = gui_editor ]; then deny_config=../gui_editor/deny-gui_editor.toml; fi",
    "bash gui_editor/container/tool_image.sh run \"$CRATE\" env ADVISORY_DB_DIR=/tmp/advisory-dbs /usr/local/cargo/bin/cargo-deny --locked --config \"$deny_config\" check --deny warnings",
    "bash gui_editor/container/tool_image.sh run \"$CRATE\" cargo test --locked",
];

// GEDT-AR-00025 and GEDT-AR-00030:
// the CI supply-chain gate runs, in order, before the tests.
// GEDT-AR-00034, T-54: every cargo step runs in the checked tool image.
#[test]
fn test_ar_00025_ar_00030_ci_test_job_gates_each_crate_in_order() {
    let wf = workflow();
    let test = wf.at("jobs").at("test");
    assert_eq!(
        test.at("needs").as_str(),
        Some("gui-editor-build"),
        "test does not wait for gui-editor-build"
    );
    assert_eq!(
        test.at("env").at("CRATE").as_str(),
        Some("${{ matrix.crate }}"),
        "CRATE is not taken from the matrix through env"
    );
    assert_eq!(
        test.at("env").at("CONTAINER_ENGINE").as_str(),
        Some("docker"),
        "the test job does not select docker for the tool image"
    );
    let mut commands: Vec<&str> = Vec::new();
    for step in steps(test) {
        assert!(
            step.get("env").is_none(),
            "test step overrides env: {step:?}"
        );
        assert!(
            step.get("working-directory").is_none(),
            "test step sets a working directory; tool_image.sh takes the directory: {step:?}"
        );
        let run = run_text(step);
        assert!(
            !run.contains("${{"),
            "test interpolates expressions into run"
        );
        commands.extend(run.lines().map(str::trim).filter(|l| !l.is_empty()));
    }
    assert_eq!(commands, TEST_JOB_COMMANDS, "test job commands and order");
}

// --- GEDT-AR-00034, T-54: the tool image ---

fn containerfile_text() -> String {
    let text = read(&gui_dir().join("container/Containerfile"));
    assert_plain_text("Containerfile", &text);
    text
}

/// The Containerfile instructions without comments and blank lines.
/// Panics on a line continuation or a heredoc, which would hide commands from the tests.
fn containerfile_instructions() -> Vec<String> {
    containerfile_text()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            assert!(
                !l.ends_with('\\') && !l.contains("<<"),
                "Containerfile line continuation or heredoc: {l}"
            );
            l.to_string()
        })
        .collect()
}

/// The whole tool image definition.
/// The exact list keeps the image to the pinned base plus the four pinned tools
/// plus the unpinned Debian trixie shellcheck package (T-60):
/// no other package, no other source, and no other instruction.
const CONTAINERFILE_INSTRUCTIONS: [&str; 9] = [
    "ARG BASE_IMAGE=docker.io/library/rust:1.100.0-trixie@sha256:223cc32ddca8f3ee90c436dbc94b6b5f052ea7f9653c5201e9612f08ad7d21b8",
    "FROM ${BASE_IMAGE}",
    "ARG BASE_IMAGE",
    "LABEL org.opencontainers.image.base.name=\"${BASE_IMAGE}\"",
    "RUN cargo install --locked --version 0.23.0 cargo-audit && rm -rf \"$CARGO_HOME/registry\"",
    "RUN cargo install --locked --version 0.22.0 cargo-deny && rm -rf \"$CARGO_HOME/registry\"",
    "RUN rustup component add clippy",
    "RUN rustup component add rustfmt",
    "RUN apt-get update && apt-get install --yes --no-install-recommends shellcheck && rm -rf /var/lib/apt/lists/*",
];

// GEDT-AR-00034, T-54 and T-60:
// the tool image adds to the pinned base only the four pinned tools
// plus the unpinned Debian trixie shellcheck package.
#[test]
fn test_ar_00034_t54_containerfile_adds_only_pinned_tools() {
    assert_eq!(
        containerfile_instructions(),
        CONTAINERFILE_INSTRUCTIONS,
        "Containerfile instructions"
    );
}

/// Options and settings that turn off or weaken the apt archive signature check,
/// or add a package source next to the signature-checked Debian archive of the base.
/// The test compares them in lowercase.
const APT_UNSIGNED_SOURCE_MARKERS: [&str; 10] = [
    "allow-unauthenticated",
    "allowunauthenticated",
    "allow-insecure-repositories",
    "allowinsecurerepositories",
    "allowdowngradetoinsecurerepositories",
    "trusted=yes",
    "check-valid-until",
    "force-yes",
    "allow-downgrades",
    "sources.list",
];

// GEDT-AR-00034, T-60: one instruction installs the Debian trixie shellcheck package,
// with no version pin and with no option that turns off the archive signature check.
#[test]
fn test_ar_00034_t60_shellcheck_package_unpinned_and_signature_checked() {
    let instructions = containerfile_instructions();
    let shellcheck_instructions: Vec<&String> = instructions
        .iter()
        .filter(|l| l.contains("shellcheck"))
        .collect();
    assert_eq!(
        shellcheck_instructions.len(),
        1,
        "Containerfile instructions naming shellcheck: {shellcheck_instructions:?}"
    );
    let install = shellcheck_instructions[0];
    assert!(
        install.contains("apt-get install") && !install.contains("shellcheck="),
        "shellcheck is not installed through apt without a version pin: {install}"
    );
    let text = containerfile_text().to_lowercase();
    for marker in APT_UNSIGNED_SOURCE_MARKERS {
        assert!(!text.contains(marker), "Containerfile contains {marker}");
    }
}

/// A directory under the system temp directory, removed on drop.
struct ScratchDir(PathBuf);

impl ScratchDir {
    fn new(test: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after 1970")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{test}-{}-{nanos}", std::process::id()));
        // create_dir fails on an existing path,
        // so no planted directory or symlink is reused.
        fs::create_dir(&dir).unwrap_or_else(|err| panic!("cannot create {}: {err}", dir.display()));
        ScratchDir(dir)
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The shellcheck command of run_shellcheck_gate in check_shellcheck.sh.
const GATE_SHELLCHECK_BIN: &str = "/usr/bin/shellcheck";
const GATE_SHELLCHECK_OPTIONS: [&str; 2] = ["--norc", "--"];

// GEDT-AR-00055, T-60:
// a planted script with one finding fails the shellcheck command of the gate,
// and a clean script passes it.
// The test runs in the tool image,
// where check_tool_image proves that shellcheck starts,
// so a missing /usr/bin/shellcheck fails the test instead of skipping it.
#[test]
fn test_ar_00055_t60_image_shellcheck_fails_planted_finding() {
    assert!(
        Path::new(GATE_SHELLCHECK_BIN).is_file(),
        "{GATE_SHELLCHECK_BIN} is missing; run the tests in the tool image"
    );
    let gate_script = read(&gui_dir().join("check_shellcheck.sh"));
    assert!(
        gate_script.contains("/usr/bin/shellcheck --norc -- \"${CHECKED_SCRIPTS[@]}\""),
        "the gate command in check_shellcheck.sh differs from the command this test runs"
    );
    let scratch = ScratchDir::new("ar_00055_t60_planted_finding");
    let run_gate_command = |name: &str, content: &str| {
        let script = scratch.0.join(name);
        fs::write(&script, content).expect("write planted script");
        Command::new(GATE_SHELLCHECK_BIN)
            .args(GATE_SHELLCHECK_OPTIONS)
            .arg(&script)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap_or_else(|err| panic!("cannot run {GATE_SHELLCHECK_BIN}: {err}"))
    };
    // SC2086: the unquoted expansion is the single finding.
    let finding = run_gate_command(
        "one_finding.sh",
        "#!/usr/bin/env bash\nname=\"$1\"\necho $name\n",
    );
    assert!(
        !finding.success(),
        "shellcheck passes a script with one SC2086 finding"
    );
    let clean = run_gate_command(
        "clean.sh",
        "#!/usr/bin/env bash\nname=\"$1\"\necho \"$name\"\n",
    );
    assert!(clean.success(), "shellcheck fails a clean script: {clean}");
}

// GEDT-AR-00034: the base is the official Rust image, pinned by version tag and digest,
// and the Containerfile, tool_image.sh, and the checked versions agree.
#[test]
fn test_ar_00034_t54_base_and_tool_pins_agree() {
    let pinned = tool_image_sh_assignment("PINNED_RUST_IMAGE");
    assert!(
        pinned.starts_with("docker.io/library/rust:"),
        "base is not the official Rust image: {pinned}"
    );
    assert!(
        is_version_and_digest_pinned(&pinned),
        "base not pinned by version and digest: {pinned}"
    );
    assert_eq!(
        CONTAINERFILE_INSTRUCTIONS[0],
        format!("ARG BASE_IMAGE={pinned}"),
        "Containerfile base differs from tool_image.sh"
    );
    assert_eq!(tool_image_sh_assignment("CARGO_AUDIT_VERSION"), "0.23.0");
    assert_eq!(tool_image_sh_assignment("CARGO_DENY_VERSION"), "0.22.0");
    let toolchain = pinned
        .split_once(':')
        .and_then(|(_, tag)| tag.split_once('-'))
        .map(|(version, _)| version)
        .unwrap_or_default();
    let clippy_minor = toolchain.split('.').nth(1).unwrap_or_default();
    assert!(
        tool_image_sh_assignment("CLIPPY_VERSION_LINE")
            .starts_with(&format!("clippy 0.1.{clippy_minor} (")),
        "clippy version does not follow toolchain {toolchain}"
    );
    assert!(
        tool_image_sh_assignment("RUSTFMT_VERSION_LINE").starts_with("rustfmt "),
        "RUSTFMT_VERSION_LINE is not a rustfmt version"
    );
}

// GEDT-AR-00034, T-54 and T-60:
// build.sh, the shellcheck gate, and CI use the tool image and install no tool per run;
// CI pushes no image and logs in to no registry.
#[test]
fn test_ar_00034_t54_no_tool_install_and_no_image_push() {
    let files = [
        ("build.sh", read(&gui_dir().join("build.sh"))),
        ("tool_image.sh", read(&gui_dir().join("container/tool_image.sh"))),
        ("check_shellcheck.sh", read(&gui_dir().join("check_shellcheck.sh"))),
        ("ci.yml", workflow_text()),
    ];
    for (file, text) in &files {
        assert!(!text.contains("cargo install"), "{file} runs cargo install");
        assert!(!text.contains("apt-get"), "{file} runs apt-get");
    }
    let ci = workflow_text();
    for needle in ["docker push", "podman push", "docker login", "podman login", "packages:"] {
        assert!(!ci.contains(needle), "ci.yml contains {needle}");
    }
}

// GEDT-AR-00032 and GEDT-AR-00033, T-54:
// the container directory holds only the Containerfile,
// a configuration format, and Bash scripts.
#[test]
fn test_ar_00032_ar_00033_t54_container_files_are_config_or_bash() {
    let dir = gui_dir().join("container");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", dir.display()))
        .map(|entry| entry.expect("directory entry").file_name().to_string_lossy().into_owned())
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["Containerfile", "tool_image.sh"], "container files");
    let script = read(&dir.join("tool_image.sh"));
    assert!(
        script.starts_with("#!/usr/bin/env bash\n"),
        "tool_image.sh is not a Bash script"
    );
}

// GEDT-AR-00025: a hung step cannot hold a runner indefinitely.
#[test]
fn test_ar_00025_ci_every_job_has_timeout() {
    let wf = workflow();
    for (name, job) in jobs(&wf) {
        assert!(
            job.at("timeout-minutes").as_u64().is_some_and(|m| m > 0),
            "{name} has no positive timeout-minutes"
        );
    }
}

// Product owner decision for T-01: no Python in CI or in the pipeline tests.
// T-54 extends the rule to build.sh and the tool image files.
#[test]
fn test_ar_00025_ac6_ci_and_pipeline_tests_use_no_python() {
    for (file, text) in [
        ("ci.yml", workflow_text()),
        (
            "test_build_pipeline.sh",
            read(&gui_dir().join("tests/pipeline/test_build_pipeline.sh")),
        ),
        ("build.sh", read(&gui_dir().join("build.sh"))),
        ("Containerfile", containerfile_text()),
        ("tool_image.sh", read(&gui_dir().join("container/tool_image.sh"))),
    ] {
        let text = text.to_lowercase();
        for needle in ["python", "pyyaml"] {
            assert!(!text.contains(needle), "{file} mentions {needle}");
        }
    }
}

// GEDT-AR-00034, T-54, decision "CI cache, T-54 follow-up":
// CI uses no action other than the pinned checkout,
// and no job imports or exports a layer cache.
#[test]
fn test_ar_00034_t54_ci_adds_no_action_and_no_layer_cache() {
    let wf = workflow();
    for (name, job) in jobs(&wf) {
        for step in steps(job) {
            if let Some(uses) = step.get("uses") {
                let uses = uses.as_str().unwrap_or_default();
                assert!(
                    uses.starts_with("actions/checkout@"),
                    "{name} uses an action other than actions/checkout: {uses}"
                );
            }
        }
    }
    let tool_image_sh = read(&gui_dir().join("container/tool_image.sh"));
    for (file, text) in [("ci.yml", workflow_text()), ("tool_image.sh", tool_image_sh)] {
        for needle in ["--cache-from", "--cache-to", "buildx", "actions/cache", "docker load"] {
            assert!(!text.contains(needle), "{file} contains {needle}");
        }
    }
}

// GEDT-AR-00025 and GEDT-AR-00034, T-54:
// the gui-editor-build job runs the pipeline tests, then build.sh with docker,
// and nothing else.
#[test]
fn test_ar_00025_ar_00034_t54_gui_editor_build_job_commands() {
    let wf = workflow();
    let job = wf.at("jobs").at("gui-editor-build");
    let commands: Vec<&str> = steps(job)
        .iter()
        .flat_map(|step| run_text(step).lines())
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(
        commands,
        [
            "./gui_editor/tests/pipeline/test_build_pipeline.sh",
            "CONTAINER_ENGINE=docker ./gui_editor/build.sh",
        ],
        "gui-editor-build commands"
    );
}

// --- Committed-language check, GEDT-AR-00032 and GEDT-AR-00033, T-55 ---
//
// Every file that git tracks in the whole repository must be Rust, Bash,
// or a data or configuration format, as the rationale of both requirements states.
// The check reads the index, not the working tree,
// so it sees the content and mode that a commit records,
// and ignored caches such as `.cargo-home/registry` stay out of scope.
// The lists below are allowlists, so an unknown format fails the check (fail-closed);
// a maintainer adds a new data or configuration format to the lists in review.

/// Extensions of data and configuration formats, which are not program code or scripts.
/// `lock` is `Cargo.lock`, generated by cargo.
/// `uml` is a PlantUML diagram, a diagram format like Mermaid.
/// No Rust toolchain output such as `wasm-bindgen` JavaScript glue is committed,
/// so the list has no entry for it
/// (GEDT-AR-00053, PLAN row "Generated code, T-55 follow-up").
/// `doxygen` stays out of the list, because a Doxygen configuration runs shell commands
/// through `INPUT_FILTER` and `FILTER_PATTERNS`.
const DATA_EXTENSIONS: [&str; 12] = [
    "dtd", "lock", "lyx", "md", "mmd", "pdf", "toml", "txt", "uml", "xml", "yaml", "yml",
];
/// `.mm` is also the Objective-C++ extension, so only a FreeMind XML map counts as data.
const FREEMIND_EXTENSION: &str = "mm";
const FREEMIND_PREFIX: &[u8] = b"<map ";
/// File names without an extension that hold data or configuration.
/// A `Containerfile` is a configuration format, per requirements open question 15;
/// `Dockerfile` and a lowercase `containerfile` are the same format.
/// The git and editor files hold settings, not commands.
/// `.gitmodules` passes, but a submodule gitlink still fails on its git mode.
/// `Makefile` stays out of the list, because its recipes are shell scripts.
const DATA_FILE_NAMES: [&str; 10] = [
    ".editorconfig",
    ".gitattributes",
    ".gitignore",
    ".gitmodules",
    "ChangeLog",
    "Containerfile",
    "Dockerfile",
    "LICENSE",
    "TODO",
    "containerfile",
];
const BASH_EXTENSIONS: [&str; 2] = ["bash", "sh"];
const RUST_EXTENSION: &str = "rs";
const GIT_MODE_EXECUTABLE: &str = "100755";
const GIT_MODE_FILE: &str = "100644";
const GIT_MODE_SYMLINK: &str = "120000";

/// One entry of the git index.
struct TrackedFile {
    path: String,
    mode: String,
    content: Vec<u8>,
}

/// Builds a git command for `repo` that sees no caller environment (D-T54-06),
/// as `container/tool_image.sh` does:
/// a GIT_* variable could make git answer for another checkout or index,
/// and system or user configuration could start a monitor command.
fn git(repo: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args(["-c", "core.fsmonitor=false", "-C"])
        .arg(repo)
        .stdin(Stdio::null());
    command
}

/// Runs git and returns its standard output; panics when git is missing or fails,
/// so the check never passes without a file list (fail-closed).
fn git_output(mut command: Command) -> Vec<u8> {
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("cannot run git: {err}"));
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

/// Reads every index entry of `repo`,
/// with the blob content of each regular file and symlink.
/// Panics unless `repo` is the top level of a git checkout.
fn tracked_files(repo: &Path) -> Vec<TrackedFile> {
    let mut top_level = git(repo);
    top_level.args(["rev-parse", "--show-toplevel"]);
    let top_level = String::from_utf8(git_output(top_level)).expect("git prints a UTF-8 path");
    let repo_real = fs::canonicalize(repo).expect("repository path resolves");
    assert_eq!(
        Path::new(top_level.trim_end_matches('\n')),
        repo_real,
        "{} is not the top level of a git checkout",
        repo.display()
    );

    let mut list = git(repo);
    list.args(["ls-files", "--stage", "-z"]);
    let listing = git_output(list);
    // Each entry reads "<mode> <object> <stage>\t<path>\0".
    let mut entries: Vec<(String, String, String)> = Vec::new();
    for entry in listing.split(|&b| b == 0).filter(|e| !e.is_empty()) {
        let entry = String::from_utf8_lossy(entry);
        let (meta, path) = entry.split_once('\t').expect("ls-files entry has a path");
        let mut fields = meta.split(' ');
        let mode = fields.next().expect("mode").to_string();
        let object = fields.next().expect("object").to_string();
        entries.push((path.to_string(), mode, object));
    }

    // Gitlinks name a commit of another repository, which has no blob to read here.
    let has_blob =
        |mode: &str| [GIT_MODE_EXECUTABLE, GIT_MODE_FILE, GIT_MODE_SYMLINK].contains(&mode);
    let blob_entries: Vec<&(String, String, String)> = entries
        .iter()
        .filter(|(_, mode, _)| has_blob(mode))
        .collect();
    let mut cat = git(repo);
    cat.args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cat
        .spawn()
        .unwrap_or_else(|err| panic!("cannot run git: {err}"));
    let request: String = blob_entries
        .iter()
        .map(|(_, _, object)| format!("{object}\n"))
        .collect();
    let mut stdin = child.stdin.take().expect("git stdin");
    // A thread writes the request, so a full output pipe cannot block both processes.
    let writer = std::thread::spawn(move || stdin.write_all(request.as_bytes()));
    let output = child.wait_with_output().expect("git cat-file finishes");
    writer
        .join()
        .expect("writer thread")
        .expect("git cat-file reads the request");
    assert!(
        output.status.success(),
        "git cat-file failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Each blob reads "<object> blob <size>\n<content>\n".
    let mut blobs = output.stdout.as_slice();
    let mut files = Vec::new();
    for (path, mode, object) in &entries {
        // The same predicate selected the requested blobs, so the output stays in step.
        if !has_blob(mode) {
            files.push(TrackedFile {
                path: path.clone(),
                mode: mode.clone(),
                content: Vec::new(),
            });
            continue;
        }
        let header_end = blobs.iter().position(|&b| b == b'\n').expect("blob header");
        let header = String::from_utf8_lossy(&blobs[..header_end]).into_owned();
        let mut fields = header.split(' ');
        assert_eq!(
            fields.next(),
            Some(object.as_str()),
            "blob header of {path}: {header}"
        );
        assert_eq!(
            fields.next(),
            Some("blob"),
            "blob header of {path}: {header}"
        );
        let size: usize = fields
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| panic!("blob header of {path}: {header}"));
        let content_start = header_end + 1;
        let content_end = content_start + size;
        assert_eq!(
            blobs.get(content_end),
            Some(&b'\n'),
            "blob of {path} ends early"
        );
        files.push(TrackedFile {
            path: path.clone(),
            mode: mode.clone(),
            content: blobs[content_start..content_end].to_vec(),
        });
        blobs = &blobs[content_end + 1..];
    }
    assert!(
        blobs.is_empty(),
        "git cat-file printed more blobs than requested"
    );
    files
}

/// The only interpreter lines that a committed script may hold, each spelled exactly,
/// with no further character before the line end (GEDT-AR-00054, amended).
/// The product owner chose "a: three exact lines", so flags, `env` options, `-S`,
/// assignments, other paths, extra whitespace, and a CRLF line end all fail.
/// Scripts set bash options in the body instead.
const BASH_SHEBANG_LINES: [&str; 3] = ["#!/bin/bash", "#!/usr/bin/bash", "#!/usr/bin/env bash"];
/// What `shebang_interpreter` returns for an accepted interpreter line.
const BASH_COMMAND: &str = "bash";

/// Returns `bash` when the first line equals one of `BASH_SHEBANG_LINES` (GEDT-AR-00054),
/// and otherwise the reason, which is never `bash`.
/// Returns `None` without a shebang,
/// and for a Rust inner attribute such as `#![allow(...)]`,
/// which rustc never reads as a shebang.
fn shebang_interpreter(content: &[u8]) -> Option<String> {
    let rest = content.strip_prefix(b"#!")?;
    let line_end = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
    let line = &content[..2 + line_end];
    if String::from_utf8_lossy(&rest[..line_end])
        .trim_start()
        .starts_with('[')
    {
        return None;
    }
    if BASH_SHEBANG_LINES
        .iter()
        .any(|accepted| accepted.as_bytes() == line)
    {
        return Some(BASH_COMMAND.to_string());
    }
    Some(format!(
        "interpreter line {:?} is not exactly #!/bin/bash, #!/usr/bin/bash, or #!/usr/bin/env bash",
        String::from_utf8_lossy(line)
    ))
}

/// Returns why the file is not Rust, Bash, or a data or configuration format,
/// or `None` when the file passes.
fn language_violation(file: &TrackedFile) -> Option<String> {
    let name = file.path.rsplit('/').next().unwrap_or(&file.path);
    let extension = Path::new(name)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase());
    let extension = extension.as_deref();
    if file.mode != GIT_MODE_FILE
        && file.mode != GIT_MODE_EXECUTABLE
        && file.mode != GIT_MODE_SYMLINK
    {
        return Some(format!(
            "git mode {} is not a file, so its language is unknown",
            file.mode
        ));
    }
    // A symlink blob holds the target path; the target is checked as its own entry.
    let is_symlink = file.mode == GIT_MODE_SYMLINK;
    let shebang = if is_symlink {
        None
    } else {
        shebang_interpreter(&file.content)
    };
    match shebang.as_deref() {
        Some("bash") => {}
        Some(interpreter) => {
            return Some(format!(
                "shebang is not an accepted bash shebang: {interpreter}"
            ));
        }
        // Without a shebang, a shell runs an executable file with /bin/sh,
        // so the file is a script in a language other than Bash.
        None if file.mode == GIT_MODE_EXECUTABLE => {
            return Some("executable without a shebang line runs as a /bin/sh script".to_string());
        }
        None => {}
    }
    if DATA_FILE_NAMES.contains(&name) {
        return None;
    }
    let passes = match extension {
        Some(ext) if BASH_EXTENSIONS.contains(&ext) => shebang.is_some() || is_symlink,
        Some(RUST_EXTENSION) => true,
        Some(FREEMIND_EXTENSION) => is_symlink || file.content.starts_with(FREEMIND_PREFIX),
        Some(ext) => DATA_EXTENSIONS.contains(&ext),
        // A Bash shebang makes an extensionless file a Bash script.
        None => shebang.is_some(),
    };
    if passes {
        return None;
    }
    Some(match extension {
        Some(ext) if BASH_EXTENSIONS.contains(&ext) => {
            "shell script without a bash shebang line".to_string()
        }
        Some(ext) => format!("extension .{ext} is not Rust, Bash, or a listed data format"),
        None => "name is not a listed data file and has no bash shebang line".to_string(),
    })
}

/// Lists each tracked file of `repo` that breaks GEDT-AR-00032 or GEDT-AR-00033,
/// as "<path>: <reason>".
fn committed_language_violations(repo: &Path) -> Vec<String> {
    tracked_files(repo)
        .iter()
        .filter_map(|file| {
            language_violation(file).map(|reason| format!("{}: {reason}", file.path))
        })
        .collect()
}

// GEDT-AR-00032 and GEDT-AR-00033, T-55:
// every file that git tracks in the repository is Rust, Bash, or data or configuration.
#[test]
fn test_ar_00032_ar_00033_t55_committed_files_are_rust_bash_or_data() {
    let violations = committed_language_violations(&repo_dir());
    assert!(
        violations.is_empty(),
        "{} tracked files are not Rust, Bash, or data or configuration:\n{}",
        violations.len(),
        violations.join("\n")
    );
}

fn file(path: &str, mode: &str, content: &str) -> TrackedFile {
    TrackedFile {
        path: path.to_string(),
        mode: mode.to_string(),
        content: content.as_bytes().to_vec(),
    }
}

// GEDT-AR-00032 and GEDT-AR-00033, T-55: the classification of single files.
#[test]
fn test_ar_00032_ar_00033_t55_language_classification() {
    let passing = [
        file("src/main.rs", GIT_MODE_FILE, "fn main() {}\n"),
        file("src/lib.rs", GIT_MODE_FILE, "#![forbid(unsafe_code)]\n"),
        file(
            "build.sh",
            GIT_MODE_EXECUTABLE,
            "#!/usr/bin/env bash\nset -eu\n",
        ),
        file("lib.sh", GIT_MODE_FILE, "#!/bin/bash\n"),
        file("container/Containerfile", GIT_MODE_FILE, "FROM x\n"),
        file("Cargo.lock", GIT_MODE_FILE, "version = 4\n"),
        file(".github/workflows/ci.yml", GIT_MODE_FILE, "on: push\n"),
        file("docs/README.MD", GIT_MODE_FILE, "# Title\n"),
        file("docs/map.mm", GIT_MODE_FILE, "<map version=\"0.9.0\">\n"),
        file("link.sh", GIT_MODE_SYMLINK, "build.sh"),
    ];
    for f in &passing {
        assert_eq!(language_violation(f), None, "{} fails", f.path);
    }
    let failing = [
        file("tool.py", GIT_MODE_FILE, "print(1)\n"),
        file("tool.js", GIT_MODE_FILE, "x()\n"),
        file("src/main.cpp", GIT_MODE_FILE, "int main() {}\n"),
        file("Makefile", GIT_MODE_FILE, "all:\n"),
        file("run", GIT_MODE_EXECUTABLE, "#!/bin/sh\n"),
        file("run.sh", GIT_MODE_EXECUTABLE, "#!/usr/bin/env sh\n"),
        file("lib.sh", GIT_MODE_FILE, "echo sourced\n"),
        file("notes.txt", GIT_MODE_FILE, "#!/usr/bin/perl -w\n"),
        file("main.rs", GIT_MODE_FILE, "#!/usr/bin/env python3\n"),
        // A flag after the interpreter fails (GEDT-AR-00054).
        file(
            "tools/run",
            GIT_MODE_EXECUTABLE,
            "#!/usr/bin/env -S bash -eu\n",
        ),
        file("data.toml", GIT_MODE_EXECUTABLE, "a = 1\n"),
        file(
            "view.mm",
            GIT_MODE_FILE,
            "#import <Foundation/Foundation.h>\n",
        ),
        file("unknown", GIT_MODE_FILE, "text\n"),
        file("vendor", "160000", ""),
    ];
    for f in &failing {
        assert!(language_violation(f).is_some(), "{} passes", f.path);
    }
}

// GEDT-AR-00033 and GEDT-AR-00054, T-55, D-T55-01:
// every `env` option fails the check, including options that still run bash.
#[test]
fn test_ar_00033_t55_env_shebang_options() {
    // These lines run bash, but each holds an `env` option or `-S` (GEDT-AR-00054).
    let failing_env_options_running_bash = [
        "#!/usr/bin/env -u PYTHONPATH bash\n",
        "#!/usr/bin/env -uPYTHONPATH bash\n",
        "#!/usr/bin/env --unset PYTHONPATH bash\n",
        "#!/usr/bin/env --unset=PYTHONPATH bash\n",
        "#!/usr/bin/env -S -a sh bash\n",
        "#!/usr/bin/env -iS LC_ALL=C bash\n",
        "#!/usr/bin/env -Sbash -e\n",
        "#!/usr/bin/env --split-string=bash -e\n",
        "#!/usr/bin/env -S -- bash\n",
        "#!/usr/bin/env -S --default-signal=PIPE bash\n",
    ];
    let passing_as_bash = shebangs_passing_the_check(&failing_env_options_running_bash);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
    let failing = [
        "#!/usr/bin/env -S -C bash python3\n",
        "#!/usr/bin/env -S --chdir bash python3\n",
        "#!/usr/bin/env -S -a bash python3\n",
        "#!/usr/bin/env -S --argv0 bash python3\n",
        "#!/usr/bin/env -S -iu bash python3\n",
        "#!/usr/bin/env -S -u PYTHONPATH -u bash python3\n",
        "#!/usr/bin/env -S-u bash python3\n",
        "#!/usr/bin/env --split-string=-u bash python3\n",
        "#!/usr/bin/env -S --uns bash python3\n",
        "#!/usr/bin/env -S -x bash\n",
        "#!/usr/bin/env -S --help bash\n",
        "#!/usr/bin/env -S -u\n",
        "#!/usr/bin/env -S -- -u bash\n",
        // A change of directory makes a relative script path name another file
        // (review R-02).
        "#!/usr/bin/env -S -C /tmp bash -e\n",
        "#!/usr/bin/env -S --chdir=/tmp bash\n",
    ];
    for content in failing {
        let f = file("tools/run", GIT_MODE_EXECUTABLE, content);
        assert!(language_violation(&f).is_some(), "{content:?} passes");
    }
}

// GEDT-AR-00033 and GEDT-AR-00054, T-55, D-T55-03 and D-T55-04:
// only the three exact interpreter lines pass, so env quoting and expansion,
// bash options, environment assignments, and a CRLF line end all fail the check.
#[test]
fn test_ar_00033_t55_shebang_grammar_allowlist() {
    // These lines run bash, but each holds more than an exact line (GEDT-AR-00054).
    let failing_flags_and_assignments = [
        "#!/usr/bin/env -S bash -eu -x\n",
        "#!/usr/bin/env -S bash --norc --noprofile -x --\n",
        "#!/usr/bin/env -S LANG=C.UTF-8 LC_CTYPE=C TZ=UTC bash\n",
        "#!/bin/bash -e\r\n",
    ];
    let failing = [
        // Characters that env -S interprets, and whitespace that is not a space or a tab.
        "#!/usr/bin/env -S bash -e #python3\n",
        "#!/usr/bin/env -S bash `python3`\n",
        "#!/usr/bin/env -S -u\tNAME\x0bbash python3\n",
        "#!/usr/bin/env -S -u NAME\u{2003}bash python3\n",
        "#!/usr/bin/env -S -u NAME\rbash python3\n",
        // Bash options that run a command, read commands from elsewhere, or take a value.
        "#!/usr/bin/env -S bash -c exit\n",
        "#!/bin/bash -xc\n",
        "#!/usr/bin/env -S bash -s\n",
        "#!/usr/bin/env -S bash -i\n",
        "#!/usr/bin/env -S bash -l\n",
        "#!/usr/bin/env -S bash --login\n",
        "#!/usr/bin/env -S bash --rcfile /tmp/rc\n",
        "#!/usr/bin/env -S bash --init-file=/tmp/rc\n",
        "#!/usr/bin/env -S bash -O extglob\n",
        "#!/usr/bin/env -S bash +O extglob\n",
        "#!/usr/bin/env -S bash -o posix\n",
        "#!/usr/bin/env -S bash -\n",
        "#!/usr/bin/env -S bash tool.py\n",
        "#!/usr/bin/env -S bash -- tool.py\n",
        // Assignments outside the locale and time zone names.
        "#!/usr/bin/env -S BASH_ENV=/tmp/rc bash\n",
        "#!/usr/bin/env -S ENV=/tmp/rc bash\n",
        "#!/usr/bin/env -S SHELLOPTS=xtrace PS4=x bash\n",
        "#!/usr/bin/env -S PATH=/tmp bash\n",
        "#!/usr/bin/env -S BASH_FUNC_x%%=x bash\n",
        "#!/usr/bin/env -S lc_all=C bash\n",
    ];
    let passing_as_bash = shebangs_passing_the_check(&failing_flags_and_assignments);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
    let passing_as_bash = shebangs_passing_the_check(&failing);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
}

// GEDT-AR-00033 and GEDT-AR-00054, T-55, product owner answers "a: restrict paths"
// and "a: three exact lines", and round 4 tester observation:
// /usr/bin/env runs only `bash`, so env with a bash path or an argv0 value fails,
// as does every other interpreter path.
#[test]
fn test_ar_00033_t55_shebang_paths_and_argv0() {
    // These lines run bash, but none is one of the three exact lines (GEDT-AR-00054).
    let passing_as_bash = shebangs_passing_the_check(&[
        "#!/usr/bin/env /bin/bash\n",
        "#!/usr/bin/env -S /usr/bin/bash -e\n",
        "#!/usr/bin/env -S -a bash bash\n",
        "#!/usr/bin/env -S --argv0=tool bash\n",
        "#!/usr/bin/env -S -- - LANG=C bash\n",
    ]);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
    let passing_as_bash = shebangs_passing_the_check(&[
        "#!bash\n",
        "#!./bash\n",
        "#!/opt/x/bash\n",
        "#!/usr/local/bin/bash\n",
        "#!/bin/env bash\n",
        "#!/usr/bin/env ./bash\n",
        "#!/usr/bin/env /opt/x/bash\n",
        "#!/usr/bin/env -S /usr/bin/env bash\n",
        "#!/usr/bin/env -S -a -bash bash\n",
        "#!/usr/bin/env -S -a-bash bash\n",
        "#!/usr/bin/env -S -ia-bash bash\n",
        "#!/usr/bin/env -S --argv0=-bash bash\n",
        "#!/usr/bin/env -S --argv0 -bash bash\n",
        "#!/usr/bin/env -S LANG=C - bash\n",
    ]);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
}

/// A fresh git repository in the temporary directory, removed on drop.
struct PlantedRepo(PathBuf);

impl PlantedRepo {
    fn new(test: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after 1970")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{test}-{}-{nanos}", std::process::id()));
        fs::create_dir(&dir).unwrap_or_else(|err| panic!("cannot create {}: {err}", dir.display()));
        let mut init = git(&dir);
        init.args(["init", "--quiet"]);
        git_output(init);
        PlantedRepo(dir)
    }

    /// Writes the file and stages it, with the executable bit when `executable` holds.
    fn add(&self, path: &str, content: &str, executable: bool) {
        let target = self.0.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).expect("create planted directory");
        }
        fs::write(&target, content).expect("write planted file");
        let mut add = git(&self.0);
        let chmod = if executable {
            "--chmod=+x"
        } else {
            "--chmod=-x"
        };
        add.args(["add", chmod, "--", path]);
        git_output(add);
    }
}

impl Drop for PlantedRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// GEDT-AR-00032 and GEDT-AR-00033, T-55:
// the check over a git repository flags only two planted files:
// a Python file and a script with a non-Bash shebang.
#[test]
fn test_ar_00032_ar_00033_t55_planted_files_fail_the_check() {
    let repo = PlantedRepo::new("t55-planted");
    repo.add("src/main.rs", "fn main() {}\n", false);
    repo.add("build.sh", "#!/usr/bin/env bash\n", true);
    repo.add("Cargo.toml", "[package]\n", false);
    repo.add("tools/report.py", "print('report')\n", false);
    repo.add("tools/deploy", "#!/bin/sh\necho deploy\n", true);
    fs::write(repo.0.join("ignored.py"), "print('untracked')\n").expect("write untracked file");
    let violations = committed_language_violations(&repo.0);
    let flagged: Vec<&str> = violations
        .iter()
        .map(|v| v.split(':').next().unwrap_or_default())
        .collect();
    assert_eq!(
        flagged,
        ["tools/deploy", "tools/report.py"],
        "violations: {violations:?}"
    );
}

// GEDT-AR-00054, T-55, product owner answer "a: three exact lines":
// the planted cases of the GEDT-AR-00054 verification.
// Planted tracked scripts with the three exact interpreter lines pass,
// and every other line fails, as an executable and as a `.sh` file.
#[test]
fn test_ar_00054_t55_planted_interpreter_lines() {
    let accepted = ["#!/bin/bash", "#!/usr/bin/bash", "#!/usr/bin/env bash"];
    let rejected = [
        "#!./bash",
        "#!/opt/x/bash",
        "#!/usr/bin/env /opt/x/bash",
        "#!/bin/bash -e",
        "#!/usr/bin/env bash -e",
        "#!/usr/bin/env -S bash",
        "#!/usr/bin/env /bin/bash",
        "#! /bin/bash",
        "#!/bin/bash ",
        "#!/bin/bash\r",
    ];
    let repo = PlantedRepo::new("t55-ar-00054");
    let mut expected = Vec::new();
    for (index, line) in accepted.iter().chain(&rejected).enumerate() {
        let content = format!("{line}\necho run\n");
        let executable = format!("tools/run{index:02}");
        let sourced = format!("tools/lib{index:02}.sh");
        repo.add(&executable, &content, true);
        repo.add(&sourced, &content, false);
        if index >= accepted.len() {
            expected.push(sourced);
            expected.push(executable);
        }
    }
    expected.sort();
    let violations = committed_language_violations(&repo.0);
    let flagged: Vec<&str> = violations
        .iter()
        .map(|v| v.split(':').next().unwrap_or_default())
        .collect();
    assert_eq!(flagged, expected, "violations: {violations:?}");
}

// GEDT-AR-00053, T-55, tester, review R-01:
// the GEDT-AR-00032 and GEDT-AR-00033 check, with no exception for generated code,
// fails planted tracked files that the Rust toolchain generates for WebAssembly,
// including `.js` glue with a generator marker and glue under `pkg/` and `target/`.
#[test]
fn test_ar_00053_t55_tester_planted_generated_js_fails() {
    let glue =
        "// @generated by wasm-bindgen\nlet wasm;\nexport function start() { wasm.start(); }\n";
    let repo = PlantedRepo::new("t55-tester-ar-00053");
    repo.add("src/lib.rs", "pub fn start() {}\n", false);
    repo.add("Cargo.toml", "[package]\n", false);
    repo.add("pkg/gui_editor.js", glue, false);
    repo.add(
        "pkg/gui_editor.d.ts",
        "export function start(): void;\n",
        false,
    );
    repo.add("pkg/gui_editor_bg.wasm", "\0asm\u{1}\0\0\0", false);
    repo.add("pkg/package.json", "{\"name\": \"gui_editor\"}\n", false);
    repo.add(
        "target/wasm32-unknown-unknown/release/gui_editor.js",
        glue,
        false,
    );
    repo.add("web/snippets/inline0.js", "export function f() {}\n", true);
    repo.add("web/boot.js", "#!/usr/bin/env bash\nnode glue.js\n", true);
    repo.add(
        "web/attr.js",
        "#![generated]\nexport function g() {}\n",
        false,
    );
    let violations = committed_language_violations(&repo.0);
    let flagged: Vec<&str> = violations
        .iter()
        .map(|v| v.split(':').next().unwrap_or_default())
        .collect();
    assert_eq!(
        flagged,
        [
            "pkg/gui_editor.d.ts",
            "pkg/gui_editor.js",
            "pkg/gui_editor_bg.wasm",
            "pkg/package.json",
            "target/wasm32-unknown-unknown/release/gui_editor.js",
            "web/attr.js",
            "web/boot.js",
            "web/snippets/inline0.js",
        ],
        "violations: {violations:?}"
    );
}

// GEDT-AR-00054, T-55, tester:
// line end, end of file, and prefix cases of the three exact interpreter lines,
// planted in a git index as an executable and as a `.sh` file.
// The end of the file ends the first line, as for the kernel.
#[test]
fn test_ar_00054_t55_tester_planted_line_end_and_prefix_cases() {
    let accepted = [
        "#!/bin/bash",
        "#!/usr/bin/bash",
        "#!/usr/bin/env bash",
        "#!/bin/bash\n",
        "#!/usr/bin/env bash\n\r\n",
    ];
    let rejected = [
        "#!/bin/bash\r\necho run\r\n",
        "#!/bin/bash\r",
        "#!/usr/bin/env bash\r\n",
        "\u{feff}#!/bin/bash\necho run\n",
        " #!/bin/bash\necho run\n",
        "\t#!/bin/bash\necho run\n",
        "\n#!/bin/bash\necho run\n",
        "#!/usr/bin/env bash\t\necho run\n",
        "#!/usr/bin/env bash\t",
        "#!/usr/bin/env\tbash\n",
        "#!/usr/bin/env bash \n",
        "#!/usr/bin/bash\u{b}\n",
        "#!/usr/bin/bash\u{c}\n",
        "#!/bin/bash\0\n",
        "#!/bin/bash\u{a0}\n",
        "#!/bin/bash\u{2028}\n",
        "#!/bin/bash#\n",
        "#!/bin/bashx\n",
        "#!/bin/bas\n",
        "#!/usr/bin/env bash;python3\n",
        "#!/usr/bin/env bash\u{feff}\n",
    ];
    let repo = PlantedRepo::new("t55-tester-ar-00054-ends");
    let mut expected = Vec::new();
    for (index, content) in accepted.iter().chain(&rejected).enumerate() {
        let executable = format!("tools/run{index:02}");
        let sourced = format!("tools/lib{index:02}.sh");
        repo.add(&executable, content, true);
        repo.add(&sourced, content, false);
        if index >= accepted.len() {
            expected.push(sourced);
            expected.push(executable);
        }
    }
    expected.sort();
    let violations = committed_language_violations(&repo.0);
    let flagged: Vec<&str> = violations
        .iter()
        .map(|v| v.split(':').next().unwrap_or_default())
        .collect();
    assert_eq!(flagged, expected, "violations: {violations:?}");
}

// GEDT-AR-00032, GEDT-AR-00033, and GEDT-AR-00054, T-55, tester, security:
// the Rust inner attribute exemption `#![` cannot make a runnable non-Rust file pass.
// The kernel reads `#![...]` of an executable file as the interpreter path `[...]`,
// so every executable with such a first line fails, whatever its extension.
#[test]
fn test_ar_00054_t55_tester_rust_attribute_exemption_cannot_smuggle() {
    let passing = [file(
        "src/lib.rs",
        GIT_MODE_FILE,
        "#![forbid(unsafe_code)]\n",
    )];
    for f in &passing {
        assert_eq!(language_violation(f), None, "{} fails", f.path);
    }
    let failing = [
        file("tools/run", GIT_MODE_EXECUTABLE, "#![allow(x)]\nprint(1)\n"),
        file("tools/run", GIT_MODE_EXECUTABLE, "#! [x]\nprint(1)\n"),
        file("tools/run", GIT_MODE_EXECUTABLE, "#!\t[x]\nprint(1)\n"),
        file("tools/run", GIT_MODE_EXECUTABLE, "#![\n"),
        file("tools/run.sh", GIT_MODE_EXECUTABLE, "#![x]\necho run\n"),
        file("tools/run.sh", GIT_MODE_FILE, "#![x]\necho run\n"),
        file("tools/run.py", GIT_MODE_FILE, "#![x]\nprint(1)\n"),
        file("tools/run.py", GIT_MODE_EXECUTABLE, "#![x]\nprint(1)\n"),
        file(
            "tools/run.rs",
            GIT_MODE_EXECUTABLE,
            "#![allow(dead_code)]\nfn main() {}\n",
        ),
        file("tools/run.toml", GIT_MODE_EXECUTABLE, "#![x]\na = 1\n"),
        file("tools/notes", GIT_MODE_FILE, "#![x]\nprint(1)\n"),
        file("Makefile", GIT_MODE_FILE, "#![x]\nall:\n"),
    ];
    for f in &failing {
        assert!(
            language_violation(f).is_some(),
            "{} passes: {:?}",
            f.path,
            String::from_utf8_lossy(&f.content)
        );
    }
}

// GEDT-AR-00032, GEDT-AR-00033, and GEDT-AR-00054, T-55, tester, security:
// an accepted interpreter line does not make a file of another language pass
// when its extension names that language.
#[test]
fn test_ar_00054_t55_tester_exact_line_does_not_pass_other_extensions() {
    for path in [
        "tool.py",
        "tool.js",
        "tool.pl",
        "tool.ps1",
        "rules.mk",
        "tool.tmpl",
    ] {
        for line in BASH_SHEBANG_LINES {
            let content = format!("{line}\necho run\n");
            for mode in [GIT_MODE_EXECUTABLE, GIT_MODE_FILE] {
                let f = file(path, mode, &content);
                assert!(
                    language_violation(&f).is_some(),
                    "{path} {mode} {line:?} passes"
                );
            }
        }
    }
}

// GEDT-AR-00032 and GEDT-AR-00033, T-55, tester:
// a listed container configuration name passes in a subdirectory
// and with an accepted interpreter line on the first line.
#[test]
fn test_ar_00032_ar_00033_t55_tester_container_file_names_pass() {
    let passing = [
        file("sub/Containerfile", GIT_MODE_FILE, "#!/bin/bash\nFROM x\n"),
        file("Dockerfile", GIT_MODE_FILE, "FROM x\n"),
    ];
    for f in &passing {
        assert_eq!(language_violation(f), None, "{} fails", f.path);
    }
}

// GEDT-AR-00032 and GEDT-AR-00033, T-55, product owner answer to tester question 1:
// git, editor, and container configuration names pass,
// and build, template, and manifest files with embedded code or recipes still fail.
#[test]
fn test_ar_00032_ar_00033_t55_configuration_file_names() {
    let passing = [
        file(".gitattributes", GIT_MODE_FILE, "*.sh text eol=lf\n"),
        file(".gitmodules", GIT_MODE_FILE, "[submodule \"x\"]\n"),
        file("a/.editorconfig", GIT_MODE_FILE, "root = true\n"),
        file("container/Dockerfile", GIT_MODE_FILE, "FROM x\n"),
        file("container/containerfile", GIT_MODE_FILE, "FROM x\n"),
    ];
    for f in &passing {
        assert_eq!(language_violation(f), None, "{} fails", f.path);
    }
    let failing = [
        file("Makefile", GIT_MODE_FILE, "all:\n\techo x\n"),
        file("GNUmakefile", GIT_MODE_FILE, "all:\n"),
        file("makefile", GIT_MODE_FILE, "all:\n"),
        file("rules.mk", GIT_MODE_FILE, "all:\n"),
        file("view.tmpl", GIT_MODE_FILE, "{ $OUT .= 1 }\n"),
        file("site.pp", GIT_MODE_FILE, "class x {}\n"),
        file("lib.sh", GIT_MODE_FILE, "echo sourced\n"),
        file("view.mm", GIT_MODE_FILE, "<?xml version=\"1.0\"?>\n"),
        file("DOCKERFILE", GIT_MODE_FILE, "FROM x\n"),
        file("Dockerfile.py", GIT_MODE_FILE, "print(1)\n"),
        file(".gitattributes", GIT_MODE_EXECUTABLE, "*.sh text\n"),
        file("Dockerfile", GIT_MODE_FILE, "#!/usr/bin/env python3\n"),
    ];
    for f in &failing {
        assert!(language_violation(f).is_some(), "{} passes", f.path);
    }
}

// GEDT-AR-00032, T-55: with `.gitmodules` allowed, a submodule gitlink still fails,
// because its code is another repository whose language the check cannot read.
#[test]
fn test_ar_00032_t55_gitmodules_passes_and_submodule_gitlink_fails() {
    let repo = PlantedRepo::new("t55-gitlink");
    repo.add(
        ".gitmodules",
        "[submodule \"vendor\"]\n\tpath = vendor\n\turl = https://example.invalid/v.git\n",
        false,
    );
    let mut gitlink = git(&repo.0);
    gitlink.args([
        "update-index",
        "--add",
        "--cacheinfo",
        "160000,1111111111111111111111111111111111111111,vendor",
    ]);
    git_output(gitlink);
    let violations = committed_language_violations(&repo.0);
    let flagged: Vec<&str> = violations
        .iter()
        .map(|v| v.split(':').next().unwrap_or_default())
        .collect();
    assert_eq!(flagged, ["vendor"], "violations: {violations:?}");
}

// --- Tester cases for T-55, GEDT-AR-00032 and GEDT-AR-00033 ---

// GEDT-AR-00032 and GEDT-AR-00033, T-55, tester:
// edge cases of extension and shebang detection.
#[test]
fn test_ar_00032_ar_00033_t55_tester_detection_edge_cases() {
    let passing = [
        file("run.sh", GIT_MODE_EXECUTABLE, "#!/usr/bin/bash\n"),
        file("run.Sh", GIT_MODE_EXECUTABLE, "#!/usr/bin/env bash\n"),
        file("lib.BASH", GIT_MODE_FILE, "#!/usr/bin/env bash\n"),
        file("gui_editor/build.rs", GIT_MODE_FILE, "fn main() {}\n"),
        file("src/Main.RS", GIT_MODE_FILE, "fn main() {}\n"),
        file("a/b/.gitignore", GIT_MODE_FILE, "target\n"),
        file("LICENSE", GIT_MODE_FILE, "text\n"),
        file("docs/flow.mmd", GIT_MODE_FILE, "graph LR\n"),
    ];
    for f in &passing {
        assert_eq!(language_violation(f), None, "{} fails", f.path);
    }
    let failing = [
        // Bash lines that are not exactly one of the three lines (GEDT-AR-00054).
        file("run.sh", GIT_MODE_EXECUTABLE, "#!/bin/bash\r\necho x\r\n"),
        file("run.sh", GIT_MODE_EXECUTABLE, "#! /bin/bash\n"),
        file("run.Sh", GIT_MODE_EXECUTABLE, "#!/usr/bin/env -S bash -e\n"),
        file("run.sh", GIT_MODE_FILE, "#!/bin/sh\n"),
        file("run.sh", GIT_MODE_EXECUTABLE, "#!/bin/sh\r\n"),
        file("run.sh", GIT_MODE_EXECUTABLE, "#!/bin/dash\n"),
        file("run.sh", GIT_MODE_EXECUTABLE, "#!/usr/bin/env zsh\n"),
        file("a/b/c/tool.py", GIT_MODE_FILE, "print(1)\n"),
        file("TOOL.PY", GIT_MODE_FILE, "print(1)\n"),
        file("tool.Py", GIT_MODE_FILE, "print(1)\n"),
        file("tool.py", GIT_MODE_EXECUTABLE, "#!/usr/bin/env bash\n"),
        file("app.js", GIT_MODE_FILE, "x()\n"),
        file("app.ts", GIT_MODE_FILE, "x()\n"),
        file("main.c", GIT_MODE_FILE, "int main(){}\n"),
        file("main.go", GIT_MODE_FILE, "package main\n"),
        file("gen.pl", GIT_MODE_FILE, "print 1;\n"),
        file("gen.rb", GIT_MODE_FILE, "puts 1\n"),
        file("rules.mk", GIT_MODE_FILE, "all:\n"),
        file("view.tmpl", GIT_MODE_FILE, "{ $OUT }\n"),
        file("site.pp", GIT_MODE_FILE, "class x {}\n"),
        file("x.ps1", GIT_MODE_FILE, "Write-Host 1\n"),
        file("tool", GIT_MODE_FILE, "#!/usr/bin/perl\n"),
        file("tool", GIT_MODE_FILE, "#!/usr/bin/env -S python3 -u\n"),
        file(
            "tool",
            GIT_MODE_FILE,
            "#!/usr/bin/env -i PATH=/usr/bin perl\n",
        ),
        file("tool", GIT_MODE_FILE, "#!/usr/bin/env\n"),
        file("tool", GIT_MODE_FILE, "#!\n"),
        file("notes", GIT_MODE_FILE, "text\n"),
        file("data.xml", GIT_MODE_FILE, "#!/usr/bin/env node\n"),
    ];
    for f in &failing {
        assert!(
            language_violation(f).is_some(),
            "{} passes: {:?}",
            f.path,
            f.content
        );
    }
}

// GEDT-AR-00033, T-55, tester:
// an `env` option that takes a value must not hide the real interpreter.
// With `-S`, env splits the line, unsets the variable `bash`, and runs python3.
#[test]
fn test_ar_00033_t55_tester_env_option_value_does_not_hide_interpreter() {
    for content in [
        "#!/usr/bin/env -S -u bash python3\n",
        "#!/usr/bin/env -S --unset bash python3\n",
    ] {
        let f = file("tools/report", GIT_MODE_EXECUTABLE, content);
        assert!(
            language_violation(&f).is_some(),
            "{content:?} passes as bash"
        );
    }
}

// GEDT-AR-00033 and GEDT-AR-00054, T-55, tester, round 2:
// shebang spellings that the kernel and GNU env run with bash,
// but that are not one of the three exact lines, fail the check.
// The test name predates the amended GEDT-AR-00054 and is kept for traceability.
#[test]
fn test_ar_00033_t55_tester_bash_shebang_spellings_pass() {
    let passing_as_bash = shebangs_passing_the_check(&[
        "#!/usr/bin/env  bash\n",
        "#!/usr/bin/env\tbash\n",
        "#!/bin/bash -x\n",
        "#!/bin/bash -eu\n",
        "#!/usr/bin/env -S bash -e\n",
        "#!/usr/bin/env -S\tbash -e\n",
    ]);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
}

// GEDT-AR-00033, T-55, tester, round 2 (D-T55-03):
// GNU `env -S` removes quotes, expands `${NAME}`, splits at `\_`,
// and splits only at ASCII whitespace.
// Each line below makes env run `python3`, with `bash` only as an option value,
// so the check must flag every line.
// Verified with GNU coreutils env 9.10: `env --debug -S "-u'' bash echo"` runs echo.
#[test]
fn test_ar_00033_t55_tester_env_split_string_syntax_does_not_hide_interpreter() {
    let passing_as_bash = shebangs_passing_the_check(&[
        "#!/usr/bin/env -S -u'' bash python3\n",
        "#!/usr/bin/env -S -u\"\" bash python3\n",
        "#!/usr/bin/env -S -u${T55_UNSET} bash python3\n",
        "#!/usr/bin/env -S A=1\\_python3 bash\n",
        "#!/usr/bin/env -S -u NAME\u{a0}bash python3\n",
    ]);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
}

/// Returns the shebang lines that `language_violation` lets pass,
/// so a failing tester case lists every bypass, not only the first.
fn shebangs_passing_the_check<'a>(contents: &[&'a str]) -> Vec<&'a str> {
    contents
        .iter()
        .copied()
        .filter(|content| {
            let f = file("tools/report", GIT_MODE_EXECUTABLE, content);
            language_violation(&f).is_none()
        })
        .collect()
}

// GEDT-AR-00033, T-55, tester, round 2 (D-T55-04):
// `bash -c` runs its argument as the program, so a script body in another language
// runs through bash; the shebang line shows the real interpreter.
#[test]
fn test_ar_00033_t55_tester_bash_command_option_does_not_hide_interpreter() {
    let passing_as_bash = shebangs_passing_the_check(&[
        "#!/usr/bin/env -S bash -c 'exec python3 \"$0\" \"$@\"'\nprint(1)\n",
        "#!/usr/bin/env -S bash -ec 'exec python3 \"$0\"'\nprint(1)\n",
    ]);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
}

// GEDT-AR-00032, T-55, tester:
// the scope is what git tracks, so an ignored file that is force-added is checked,
// and an ignored untracked cache is not.
#[test]
fn test_ar_00032_t55_tester_tracked_scope_includes_force_added_ignored_files() {
    let repo = PlantedRepo::new("t55-tester-scope");
    repo.add(".gitignore", "*.py\n.cargo-home/\n", false);
    fs::create_dir_all(repo.0.join(".cargo-home/registry")).expect("create cache");
    fs::write(repo.0.join(".cargo-home/registry/dep.c"), "int x;\n").expect("write cache");
    fs::write(repo.0.join("cache.py"), "print(1)\n").expect("write ignored file");
    fs::write(repo.0.join("forced.py"), "print(1)\n").expect("write forced file");
    let mut add = git(&repo.0);
    add.args(["add", "-f", "--", "forced.py"]);
    git_output(add);
    let violations = committed_language_violations(&repo.0);
    let flagged: Vec<&str> = violations
        .iter()
        .map(|v| v.split(':').next().unwrap_or_default())
        .collect();
    assert_eq!(flagged, ["forced.py"], "violations: {violations:?}");
}

// GEDT-AR-00032 and GEDT-AR-00033, T-55, tester:
// the check fails closed outside the top level of a git checkout.
#[test]
#[should_panic(expected = "git")]
fn test_ar_00032_ar_00033_t55_tester_non_repository_fails_closed() {
    let repo = PlantedRepo::new("t55-tester-norepo");
    fs::remove_dir_all(repo.0.join(".git")).expect("remove .git");
    committed_language_violations(&repo.0);
}

#[test]
#[should_panic(expected = "is not the top level of a git checkout")]
fn test_ar_00032_ar_00033_t55_tester_subdirectory_fails_closed() {
    let repo = PlantedRepo::new("t55-tester-subdir");
    repo.add("sub/x.rs", "fn f() {}\n", false);
    committed_language_violations(&repo.0.join("sub"));
}

// GEDT-AR-00032 and GEDT-AR-00033, T-55, tester, security:
// a command in the repository configuration of the checked checkout never runs.
#[test]
fn test_ar_00032_ar_00033_t55_tester_repo_config_fsmonitor_does_not_run() {
    let repo = PlantedRepo::new("t55-tester-fsmonitor");
    repo.add("src/main.rs", "fn main() {}\n", false);
    let marker = repo.0.join("FSMONITOR_RAN");
    let hook = repo.0.join("hook.sh");
    fs::write(&hook, format!("#!/bin/sh\ntouch '{}'\n", marker.display())).expect("hook");
    let mut chmod = Command::new("chmod");
    chmod.arg("755").arg(&hook);
    assert!(chmod.status().expect("chmod runs").success());
    let mut config = git(&repo.0);
    config.args(["config", "core.fsmonitor"]).arg(&hook);
    git_output(config);
    let violations = committed_language_violations(&repo.0);
    assert!(violations.is_empty(), "violations: {violations:?}");
    assert!(!marker.exists(), "core.fsmonitor of the repository ran");
}

// GEDT-AR-00033, T-55, tester:
// the CI definitions are in the rationale scope, and their run steps are scripts,
// so every run step must run in Bash: no `shell` other than bash,
// and a Linux runner, whose default shell is bash.
#[test]
fn test_ar_00033_t55_tester_ci_run_steps_run_in_bash() {
    let wf = workflow();
    let allowed = |shell: &Yaml| match shell.as_str() {
        None => true,
        Some(s) => s == "bash" || s.starts_with("bash "),
    };
    assert!(
        allowed(wf.at("defaults").at("run").at("shell")),
        "workflow default shell"
    );
    for (name, job) in jobs(&wf) {
        let runner = job.at("runs-on").as_str().unwrap_or_default();
        assert!(runner.starts_with("ubuntu-"), "{name} runs on {runner}");
        assert!(
            allowed(job.at("defaults").at("run").at("shell")),
            "{name} default shell"
        );
        for step in steps(job) {
            assert!(allowed(step.at("shell")), "{name} step shell");
        }
    }
}

// GEDT-AR-00033 and GEDT-AR-00054, T-55, tester, round 3:
// attempts to break the shebang check with env option forms, flag clusters,
// separators, and first-line rules.
// Only the first line counts, so a second shebang line is body text.
#[test]
fn test_ar_00033_t55_tester_shebang_allowlist_edge_cases() {
    let f = file(
        "tools/run",
        GIT_MODE_EXECUTABLE,
        "#!/bin/bash\n#!/usr/bin/env python3\n",
    );
    assert_eq!(language_violation(&f), None, "second shebang line fails");
    // These lines run bash under GNU coreutils env 9.7 and bash 5.2 in the tool image,
    // but none is one of the three exact lines (GEDT-AR-00054).
    let passing_as_bash = shebangs_passing_the_check(&[
        "#!/usr/bin/env\t-S\tbash\t-x\n",
        "#!/usr/bin/env -S -i -- bash -nvT\n",
        "#!/usr/bin/env -S -vS bash\n",
        "#!/usr/bin/env -S -uS bash\n",
        "#!/usr/bin/env -S -i bash --noprofile --norc --\n",
        "#!/bin/bash -BCEHPTabefhnpuvx\n",
    ]);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
    let passing_as_bash = shebangs_passing_the_check(&[
        "#!/usr/bin/env -S -u=bash python3\n",
        "#!/usr/bin/env -S -Su bash\n",
        "#!/usr/bin/env -S -C bash -- python3\n",
        "#!/usr/bin/env --split-string=-ubash python3\n",
        "#!/usr/bin/env -S --split-string=-a bash python3\n",
        "#!/usr/bin/env -S LC_=C bash\n",
        "#!/usr/bin/env -S LC_ALL1=C bash\n",
        "#!/usr/bin/env -S lang=C bash\n",
        "#!/usr/bin/env -i\n",
        "#!/usr/bin/env -S -C\n",
        "#!/bin/bash -nc x\n",
        "#!/bin/bash -vs\n",
        "#!/bin/bash -xi\n",
        "#!/bin/bash +x\n",
        "#!/usr/bin/env -S bash --norc=x\n",
        "#!/usr/bin/env -S bash --noprofil\n",
        "#!/usr/bin/env python3\n#!/bin/bash\n",
        "\u{feff}#!/bin/bash\n",
        "#!\u{feff}/bin/bash\n",
        "#! [x]\n",
        // A change of directory makes a relative script path name another file
        // (review R-02).
        "#!/usr/bin/env -S -C tools bash\n",
        "#!/usr/bin/env -S --chdir=tools bash\n",
    ]);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
}

// GEDT-AR-00033, T-55, tester, round 3 (D-T55-05):
// a word that ends the options is an operand, not a flag.
// After `--`, bash runs the next word as the script and passes the committed file as $1.
// After an assignment or a lone `-`, GNU env stops reading options
// and runs the next word as the command.
// Verified in the tool image: `#!/usr/bin/env -S bash -- -x` runs a `-x` file from PATH,
// and `#!/usr/bin/env -S LANG=C -a python3 bash` runs a `-a` file from PATH.
#[test]
fn test_ar_00033_t55_tester_operand_after_option_end_does_not_hide_interpreter() {
    let passing_as_bash = shebangs_passing_the_check(&[
        "#!/usr/bin/env -S bash -- -x\nprint(1)\n",
        "#!/usr/bin/env -S bash -e -- --norc\nprint(1)\n",
        "#!/usr/bin/env -S LANG=C -a python3 bash\nprint(1)\n",
        "#!/usr/bin/env -S - -a python3 bash\nprint(1)\n",
        "#!/usr/bin/env -S LANG=C -i bash\nprint(1)\n",
    ]);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
}

// GEDT-AR-00033 and GEDT-AR-00054, T-55, tester, round 4,
// product owner answers "a: restrict paths" and "a: three exact lines":
// only #!/bin/bash, #!/usr/bin/bash, and #!/usr/bin/env bash pass, spelled exactly.
#[test]
fn test_ar_00033_t55_tester_interpreter_paths_are_restricted() {
    let passing = [
        "#!/bin/bash\n",
        "#!/usr/bin/bash\n",
        "#!/usr/bin/env bash\n",
    ];
    for content in passing {
        let f = file("tools/run", GIT_MODE_EXECUTABLE, content);
        assert_eq!(language_violation(&f), None, "{content:?} fails");
    }
    // These lines run bash, not as a login shell,
    // under GNU coreutils env 9.7 and bash 5.2.37 in the tool image,
    // but none is one of the three exact lines (GEDT-AR-00054).
    let passing_as_bash = shebangs_passing_the_check(&[
        "#! /bin/bash\n",
        "#!\t/usr/bin/bash\n",
        "#!/usr/bin/env /bin/bash\n",
        "#!/usr/bin/env /usr/bin/bash\n",
        "#!/usr/bin/env -S /bin/bash\n",
        "#!/usr/bin/env -S /usr/bin/bash -e\n",
        "#!/usr/bin/env -S -i - bash\n",
        "#!/usr/bin/env -S -a x bash\n",
    ]);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
    let passing_as_bash = shebangs_passing_the_check(&[
        "#!/bin/env bash\n",
        "#!/usr/local/bin/bash\n",
        "#!/bin//bash\n",
        "#!/bin/./bash\n",
        "#!//bin/bash\n",
        "#!/usr/bin/../bin/bash\n",
        "#!/bin/bash/\n",
        "#!/BIN/BASH\n",
        "#!/bin/Bash\n",
        "#!/bin/bash5\n",
        "#!/bin/sh\n",
        "#!/usr/bin//env bash\n",
        "#!/usr/bin/./env bash\n",
        "#!/usr/local/bin/env bash\n",
        "#!env bash\n",
        "#!/usr/bin/env -S /bin//bash\n",
        "#!/usr/bin/env -S /bin/./bash\n",
        "#!/usr/bin/env -S /usr/local/bin/bash\n",
        "#!/usr/bin/env -S bin/bash\n",
        "#!/usr/bin/env -S ./bash\n",
        "#!/usr/bin/env -S ../bash\n",
        "#!/usr/bin/env -S BASH\n",
        "#!/usr/bin/env -S bash5\n",
        "#!/usr/bin/env -S /bin/env bash\n",
        "#!/usr/bin/env -S /usr/bin/env -S bash\n",
        "#!/usr/bin/env -S /bin/sh\n",
    ]);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
}

// GEDT-AR-00033, T-55, tester, round 4 (D-T55-05 follow-up):
// an argv0 value starting with `-` makes bash a login shell, which sources startup files,
// and a second `-`, a second `--`, or `--` after an assignment is the env command.
#[test]
fn test_ar_00033_t55_tester_argv0_and_option_end_variants() {
    let passing_as_bash = shebangs_passing_the_check(&[
        "#!/usr/bin/env -S -a -x bash\n",
        "#!/usr/bin/env -S -a -- bash\n",
        "#!/usr/bin/env -S -a - bash\n",
        "#!/usr/bin/env -S -va -bash bash\n",
        "#!/usr/bin/env -S -iva-bash bash\n",
        "#!/usr/bin/env -S --argv0=--login bash\n",
        "#!/usr/bin/env -S --argv0=- bash\n",
        "#!/usr/bin/env -S --argv0 - bash\n",
        "#!/usr/bin/env -S -S -a -bash bash\n",
        "#!/usr/bin/env -S --split-string=-a-bash bash\n",
        "#!/usr/bin/env -S - - bash\n",
        "#!/usr/bin/env -S -- -- bash\n",
        "#!/usr/bin/env -S LANG=C -- bash\n",
        "#!/usr/bin/env -S - LANG=C - bash\n",
        "#!/usr/bin/env -S LANG=C -u X bash\n",
        "#!/bin/bash -- --\n",
        "#!/bin/bash -- -x\n",
        "#!/usr/bin/env -S bash -- --\n",
        "#!/usr/bin/env -S bash -x -- -e\n",
    ]);
    assert!(
        passing_as_bash.is_empty(),
        "pass as bash: {passing_as_bash:#?}"
    );
}
