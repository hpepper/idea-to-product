//! Static checks of the build pipeline configuration for GEDT-STORY-00019: T-01 and T-54.
//! Covers GEDT-AR-00024, GEDT-AR-00025, GEDT-AR-00030, GEDT-AR-00032, GEDT-AR-00033,
//! and GEDT-AR-00034, by inspecting `deny.toml`, `deny-gui_editor.toml`,
//! `.github/workflows/ci.yml`, and the tool image files in `container/`.
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
use std::path::{Path, PathBuf};

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
/// The exact list keeps the image to the pinned base plus the four pinned tools:
/// no other package, no other source, and no other instruction.
const CONTAINERFILE_INSTRUCTIONS: [&str; 8] = [
    "ARG BASE_IMAGE=docker.io/library/rust:1.100.0-trixie@sha256:223cc32ddca8f3ee90c436dbc94b6b5f052ea7f9653c5201e9612f08ad7d21b8",
    "FROM ${BASE_IMAGE}",
    "ARG BASE_IMAGE",
    "LABEL org.opencontainers.image.base.name=\"${BASE_IMAGE}\"",
    "RUN cargo install --locked --version 0.23.0 cargo-audit && rm -rf \"$CARGO_HOME/registry\"",
    "RUN cargo install --locked --version 0.22.0 cargo-deny && rm -rf \"$CARGO_HOME/registry\"",
    "RUN rustup component add clippy",
    "RUN rustup component add rustfmt",
];

// GEDT-AR-00034: the tool image adds only the four pinned tools to the pinned base.
#[test]
fn test_ar_00034_t54_containerfile_adds_only_pinned_tools() {
    assert_eq!(
        containerfile_instructions(),
        CONTAINERFILE_INSTRUCTIONS,
        "Containerfile instructions"
    );
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

// GEDT-AR-00034, T-54: build.sh and CI use the tool image and install no tool per run;
// CI pushes no image and logs in to no registry.
#[test]
fn test_ar_00034_t54_no_tool_install_and_no_image_push() {
    let files = [
        ("build.sh", read(&gui_dir().join("build.sh"))),
        ("tool_image.sh", read(&gui_dir().join("container/tool_image.sh"))),
        ("ci.yml", workflow_text()),
    ];
    for (file, text) in &files {
        assert!(!text.contains("cargo install"), "{file} runs cargo install");
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
