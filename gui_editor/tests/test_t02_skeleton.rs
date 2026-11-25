//! Tester checks for T-02, the egui walking skeleton of GEDT-STORY-00019.
//! Covers GEDT-AR-00003, GEDT-AR-00025, GEDT-AR-00036,
//! and the T-02 follow-up decisions in docs/PLAN.md.
//! The window itself needs a display, so the Wayland and X11 start
//! (GEDT-AR-00004) stays a manual product owner check.
//! Uses only the Rust standard library, by product owner rule.

// The binary crate has no library target,
// so the test compiles the picker module from its source file.
// The include also compiles the inline test module of path_picker.rs,
// so its picker tests run twice, here and in the binary; accepted (T02-N-02),
// because a library target only to avoid the repeat is not worth the extra crate layout.
#[path = "../src/path_picker.rs"]
mod path_picker;

use path_picker::PathPicker;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

fn gui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    let path = gui_dir().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

/// Polls until the dialog closes; the deadline only guards against a hang.
fn poll_until_closed(picker: &mut PathPicker) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while picker.is_open() {
        assert!(Instant::now() < deadline, "dialog worker did not finish");
        picker.poll();
        thread::yield_now();
    }
}

fn choose(picker: &mut PathPicker, result: Option<PathBuf>) {
    assert!(picker.start(move || result), "dialog did not start");
    poll_until_closed(picker);
}

// --- GEDT-STORY-00019, T-02: open-file dialog state ---

#[test]
fn test_story_00019_t02_cancel_without_earlier_path_keeps_none() {
    let mut picker = PathPicker::default();
    choose(&mut picker, None);
    assert_eq!(picker.chosen(), None);
    assert!(!picker.is_open());
}

#[test]
fn test_story_00019_t02_new_choice_replaces_earlier_path() {
    let mut picker = PathPicker::default();
    choose(&mut picker, Some(PathBuf::from("/first.xml")));
    choose(&mut picker, Some(PathBuf::from("/second.xml")));
    assert_eq!(picker.chosen(), Some(Path::new("/second.xml")));
}

#[test]
fn test_story_00019_t02_cancel_after_two_choices_keeps_latest() {
    let mut picker = PathPicker::default();
    choose(&mut picker, Some(PathBuf::from("/first.xml")));
    choose(&mut picker, Some(PathBuf::from("/second.xml")));
    choose(&mut picker, None);
    assert_eq!(picker.chosen(), Some(Path::new("/second.xml")));
}

#[test]
fn test_story_00019_t02_poll_without_dialog_changes_nothing() {
    let mut picker = PathPicker::default();
    picker.poll();
    assert!(!picker.is_open());
    assert_eq!(picker.chosen(), None);
}

// One click opens one dialog:
// a start while a dialog is open must not run its picker at all.
#[test]
fn test_story_00019_t02_second_start_never_runs_second_dialog() {
    let mut picker = PathPicker::default();
    let release = Arc::new(Barrier::new(2));
    let worker_release = Arc::clone(&release);
    assert!(picker.start(move || {
        worker_release.wait();
        Some(PathBuf::from("/first.xml"))
    }));
    let second_ran = Arc::new(AtomicBool::new(false));
    for _ in 0..3 {
        let flag = Arc::clone(&second_ran);
        assert!(!picker.start(move || {
            flag.store(true, Ordering::SeqCst);
            Some(PathBuf::from("/second.xml"))
        }));
        picker.poll();
        assert!(picker.is_open(), "dialog closed before the user answered");
    }
    release.wait();
    poll_until_closed(&mut picker);
    assert!(!second_ran.load(Ordering::SeqCst), "a second dialog ran");
    assert_eq!(picker.chosen(), Some(Path::new("/first.xml")));
    assert!(
        picker.start(|| None),
        "picker accepts a new dialog after the first closes"
    );
    poll_until_closed(&mut picker);
}

// The UI thread polls while the dialog blocks, so poll must return without the result.
#[test]
fn test_story_00019_t02_poll_does_not_block_while_dialog_open() {
    let mut picker = PathPicker::default();
    let (answer, wait_for_answer) = mpsc::channel::<()>();
    assert!(picker.start(move || {
        wait_for_answer.recv().ok();
        Some(PathBuf::from("/answered.xml"))
    }));
    for _ in 0..100 {
        picker.poll();
    }
    assert!(picker.is_open());
    assert_eq!(picker.chosen(), None, "path shown before the user answered");
    answer.send(()).expect("worker waits for the answer");
    poll_until_closed(&mut picker);
    assert_eq!(picker.chosen(), Some(Path::new("/answered.xml")));
}

#[test]
fn test_story_00019_t02_panicking_dialog_without_earlier_path_keeps_none() {
    let mut picker = PathPicker::default();
    assert!(picker.start(|| panic!("synthetic portal failure")));
    poll_until_closed(&mut picker);
    assert_eq!(picker.chosen(), None);
    choose(&mut picker, Some(PathBuf::from("/after-failure.xml")));
    assert_eq!(picker.chosen(), Some(Path::new("/after-failure.xml")));
}

// Closing the window with an open dialog drops the picker; the worker still finishes.
#[test]
fn test_story_00019_t02_dropping_picker_with_open_dialog_does_not_block() {
    let release = Arc::new(Barrier::new(2));
    let worker_release = Arc::clone(&release);
    let (done, worker_done) = mpsc::channel::<()>();
    let mut picker = PathPicker::default();
    assert!(picker.start(move || {
        worker_release.wait();
        done.send(()).ok();
        Some(PathBuf::from("/late.xml"))
    }));
    drop(picker);
    release.wait();
    worker_done
        .recv_timeout(Duration::from_secs(10))
        .expect("worker finished its dialog");
}

// T-02 decision: the dialog only returns a path; the editor never reads the file.
// A missing file, a directory, and hostile names are kept verbatim, never opened or run.
#[test]
fn test_story_00019_t02_chosen_path_is_kept_verbatim_without_access() {
    let missing = std::env::temp_dir()
        .join("gui_editor_t02_missing")
        .join("absent.xml");
    assert!(!missing.exists(), "synthetic path must not exist");
    let hostile = [
        missing.clone(),
        std::env::temp_dir(),
        PathBuf::from("/tmp/$(touch pwned); rm -rf ~ <script>alert(1)</script>.xml"),
        PathBuf::from("../../../etc/shadow"),
        PathBuf::from("/tmp/line\nbreak.xml"),
    ];
    for path in hostile {
        let mut picker = PathPicker::default();
        choose(&mut picker, Some(path.clone()));
        assert_eq!(picker.chosen(), Some(path.as_path()));
    }
    assert!(!missing.exists(), "picker created the chosen file");
}

#[test]
fn test_story_00019_t02_non_utf8_path_is_kept() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let raw = PathBuf::from(OsString::from_vec(b"/tmp/sad-\xff\xfe.xml".to_vec()));
    let mut picker = PathPicker::default();
    choose(&mut picker, Some(raw.clone()));
    assert_eq!(picker.chosen(), Some(raw.as_path()));
}

/// Tokens that would read files, start processes, or open sockets.
/// The skeleton only displays the chosen path.
/// Later tasks add file access on purpose.
const FORBIDDEN_SOURCE_TOKENS: [&str; 12] = [
    "BufReader",
    "File::",
    "OpenOptions",
    "Command::",
    "TcpStream",
    "UdpSocket",
    "fs::",
    "include_bytes",
    "include_str",
    "process::",
    "read_to_string",
    "std::net",
];

/// Source text without comments and without the inline test module,
/// whose helpers may legitimately need the standard library tools above.
fn production_source(relative: &str) -> String {
    let text = read(relative);
    let code = text.split("#[cfg(test)]").next().unwrap_or_default();
    code.lines()
        .map(|line| line.split("//").next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_story_00019_t02_sources_do_not_read_files_or_run_processes() {
    for file in ["src/main.rs", "src/path_picker.rs"] {
        let code = production_source(file);
        for token in FORBIDDEN_SOURCE_TOKENS {
            assert!(!code.contains(token), "{file} uses {token}");
        }
    }
}

#[test]
fn test_story_00019_t02_forbidden_token_scan_detects_file_read() {
    let sample = "fn f() { let _ = std::fs::read_to_string(\"x\"); }";
    assert!(FORBIDDEN_SOURCE_TOKENS.iter().any(|t| sample.contains(t)));
}

// --- GEDT-STORY-00019, GEDT-AR-00004: window contents for the manual check ---

// The manual X11 check greps `xwininfo` for the window title.
#[test]
fn test_story_00019_t02_window_title_and_skeleton_widgets() {
    let code = production_source("src/main.rs");
    assert!(code.contains("const APP_TITLE: &str = \"gui_editor\";"));
    assert!(code.contains("eframe::run_native(\n        APP_TITLE,"));
    for label in [
        "\"Open\"",
        "\"No file chosen\"",
        "\"Components\"",
        "\"No components\"",
    ] {
        assert!(code.contains(label), "main.rs lacks {label}");
    }
    assert!(code.contains("rfd::FileDialog::new().pick_file()"));
}

// --- GEDT-AR-00003, GEDT-AR-00036, T-02 follow-up: manifest and lock file ---

fn table_lines(manifest: &str, header: &str) -> Vec<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == header;
            continue;
        }
        if inside && !line.is_empty() && !line.starts_with('#') {
            lines.push(line.to_string());
        }
    }
    lines
}

#[test]
fn test_ar_00036_t02_gui_editor_version_is_0_2_0() {
    let manifest = read("Cargo.toml");
    let package = table_lines(&manifest, "[package]");
    assert!(
        package.contains(&"version = \"0.2.0\"".to_string()),
        "{package:?}"
    );
}

// eframe 0.37.0 without default features, with glow, wayland, x11, and default_fonts;
// rfd 0.18 with xdg-portal only; no other dependency.
#[test]
fn test_ar_00003_t02_dependencies_are_approved_set() {
    let manifest = read("Cargo.toml");
    let expected = [
        "eframe = { version = \"0.37.0\", default-features = false, \
         features = [\"glow\", \"wayland\", \"x11\", \"default_fonts\"] }",
        "rfd = { version = \"0.18.0\", default-features = false, features = [\"xdg-portal\"] }",
    ];
    assert_eq!(table_lines(&manifest, "[dependencies]"), expected);
    for other in [
        "[dev-dependencies]",
        "[build-dependencies]",
        "[features]",
        "[patch.crates-io]",
    ] {
        assert!(!manifest.contains(other), "Cargo.toml has {other}");
    }
}

/// Versions of every locked package with this name.
fn locked_versions(lock: &str, name: &str) -> Vec<String> {
    let wanted = format!("name = \"{name}\"");
    let lines: Vec<&str> = lock.lines().collect();
    lines
        .windows(2)
        .filter(|pair| pair[0] == wanted)
        .filter_map(|pair| pair[1].strip_prefix("version = \""))
        .map(|v| v.trim_end_matches('"').to_string())
        .collect()
}

#[test]
fn test_ar_00025_t02_lock_file_pins_approved_versions() {
    let lock = read("Cargo.lock");
    for (name, version) in [("eframe", "0.37.0"), ("flate2", "1.1.9"), ("rfd", "0.18.0")] {
        assert_eq!(
            locked_versions(&lock, name),
            [version],
            "{name} in Cargo.lock"
        );
    }
    // The gtk3 and gtk4 features of rfd would link GTK;
    // only the portal backend is approved.
    for absent in ["gtk-sys", "gtk4"] {
        assert!(
            locked_versions(&lock, absent).is_empty(),
            "Cargo.lock has {absent}"
        );
    }
    let gui = locked_versions(&lock, "gui_editor");
    assert_eq!(gui, ["0.2.0"], "gui_editor in Cargo.lock");
}

#[test]
fn test_ar_00025_t02_locked_versions_reader() {
    let lock = "[[package]]\nname = \"a\"\nversion = \"1.0.0\"\n\n\
                [[package]]\nname = \"a\"\nversion = \"2.0.0\"\n\
                [[package]]\nname = \"ab\"\nversion = \"3.0.0\"\n";
    assert_eq!(locked_versions(lock, "a"), ["1.0.0", "2.0.0"]);
    assert!(locked_versions(lock, "b").is_empty());
}

// --- T-02 follow-up: the zenity fallback is documented ---

#[test]
fn test_story_00019_t02_readme_documents_zenity_fallback_from_path() {
    let readme = read("README.md");
    let line = readme
        .lines()
        .find(|l| l.contains("zenity"))
        .expect("README.md mentions the zenity fallback");
    assert!(
        line.contains("PATH"),
        "README zenity line does not say PATH: {line}"
    );
}
