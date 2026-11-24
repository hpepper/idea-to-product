//! Golden-output regression tests for the `sad2md` binary (GEDT-AR-00030).
//!
//! Each test runs `sad2md` on a shared SAD XML fixture in `sad_xml_sql/tests/`,
//! and compares the generated `draft_architecture.md` with `tests/golden/<fixture>.md`.
//! The match must be byte for byte.
//! The golden files are the output of `sad2md` at commit dd719d1, the T-39 baseline.
//!
//! To accept an intended output change, regenerate the golden file:
//! run `cargo run -- ../sad_xml_sql/tests/<fixture>.xml` in `sad2md/`,
//! then copy `draft_architecture.md` to `tests/golden/<fixture>.md` and review the diff.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The file name `sad2md` always writes to, in its working directory.
const MARKDOWN_OUTPUT_FILE: &str = "draft_architecture.md";

fn fixture_path(fixture: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../sad_xml_sql/tests")
        .join(format!("{fixture}.xml"))
}

fn golden_path(fixture: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!("{fixture}.md"))
}

/// Runs `sad2md` on the fixture in a fresh working directory,
/// and returns the generated markdown.
///
/// `sad2md` writes its output to a fixed name in the working directory,
/// so each run gets its own directory to keep parallel tests apart.
/// `create_dir` fails on an existing path,
/// so a leftover or planted directory is never reused.
fn run_sad2md(fixture: &str) -> String {
    let work_dir =
        std::env::temp_dir().join(format!("sad2md_golden_{fixture}_{}", std::process::id()));
    fs::create_dir(&work_dir)
        .unwrap_or_else(|err| panic!("cannot create {}: {err}", work_dir.display()));

    let output = Command::new(env!("CARGO_BIN_EXE_sad2md"))
        .arg(fixture_path(fixture))
        .current_dir(&work_dir)
        .output()
        .expect("cannot start sad2md");
    let markdown = fs::read_to_string(work_dir.join(MARKDOWN_OUTPUT_FILE));
    fs::remove_dir_all(&work_dir)
        .unwrap_or_else(|err| panic!("cannot remove {}: {err}", work_dir.display()));

    assert!(
        output.status.success(),
        "sad2md failed on {fixture}: {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    markdown.unwrap_or_else(|err| panic!("sad2md wrote no {MARKDOWN_OUTPUT_FILE}: {err}"))
}

/// Drops the differences a CRLF checkout or an edited trailing newline introduce.
/// The result only explains a mismatch; the pass condition stays byte for byte.
fn without_line_ending_differences(text: &str) -> String {
    text.replace("\r\n", "\n")
        .trim_end_matches('\n')
        .to_string()
}

fn panic_line_endings_only(fixture: &str, golden_file: &Path) -> ! {
    panic!(
        "sad2md output for {fixture} differs from {} \
         only in line endings or the trailing newline.\n\
         Check that the golden file has LF line endings (see .gitattributes) \
         and ends with exactly the newlines sad2md writes.",
        golden_file.display()
    )
}

/// Fails with the first differing line,
/// so a regression is visible without an external diff tool.
fn assert_matches_golden(fixture: &str) {
    let actual = run_sad2md(fixture);
    let golden_file = golden_path(fixture);
    let expected = fs::read_to_string(&golden_file)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", golden_file.display()));
    if actual == expected {
        return;
    }
    if without_line_ending_differences(&actual) == without_line_ending_differences(&expected) {
        panic_line_endings_only(fixture, &golden_file);
    }

    let mut actual_lines = actual.lines();
    let mut expected_lines = expected.lines();
    let mut line_number = 1;
    loop {
        match (expected_lines.next(), actual_lines.next()) {
            (Some(expected_line), Some(actual_line)) if expected_line == actual_line => {
                line_number += 1;
            }
            // `str::lines` drops `\r` and the final newline,
            // so equal lines can still hide a line ending difference.
            (None, None) => panic_line_endings_only(fixture, &golden_file),
            (expected_line, actual_line) => panic!(
                "sad2md output for {fixture} differs from {} at line {line_number}\n\
                 expected: {expected_line:?}\n\
                 actual:   {actual_line:?}",
                golden_file.display()
            ),
        }
    }
}

#[test]
fn ar_00030_sad2md_golden_test_sad() {
    assert_matches_golden("test_sad");
}

#[test]
fn ar_00030_sad2md_golden_test_infrastructure_sad() {
    assert_matches_golden("test_infrastructure_sad");
}
