//! Load, edit, and save regression tests for the editor (GEDT-AR-00030).
//!
//! The tests drive the editor as `main` does, one render and one key event at a time,
//! but render to a ratatui `TestBackend`, so no terminal is needed.
//! The project is the shared fixture `sad_xml_sql/tests/test_sad.xml`,
//! which includes `test_infrastructure_sad.xml`.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, layout::Rect, Terminal};
use rusqlite::Connection;
use sad_xml_sql::get_component_by_name;

use crate::app_state::AppState;
use crate::work::{handle_work_input, render_work, WorkState, MODIFIED_MARK};
use crate::{load_project, render_status};

/// The file name the F7 handler in `work.rs` saves the project to by default.
const SAVE_FILE_NAME: &str = "tui_out.xml";

/// The height of the status pane, as `main` lays it out.
const STATUS_PANE_HEIGHT: u16 = 3;

/// Returns the save path the F7 tests set on `WorkState`,
/// `test-output/tui_out.xml` under the crate root, and creates its folder.
/// The tool image mounts the crate root read-only and only `test-output` writable (T-54).
fn save_file() -> String {
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test-output");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("cannot create {}: {err}", output_dir.display()));
    output_dir
        .join(SAVE_FILE_NAME)
        .to_str()
        .expect("non UTF-8 crate path")
        .to_string()
}

/// Serializes the tests that press F7, because every save writes the same `save_file()`.
static SAVE_FILE_LOCK: Mutex<()> = Mutex::new(());

/// A component of the fixture with a purpose, a summary, and a title.
const EDITED_COMPONENT: &str = "BattleResolver";

fn fixture_path() -> String {
    format!(
        "{}/../sad_xml_sql/tests/test_sad.xml",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// The editor UI state and a headless terminal, wired like the main loop in `main.rs`.
struct Editor {
    db_conn: Connection,
    terminal: Terminal<TestBackend>,
    work_state: WorkState,
    app_state: AppState,
}

impl Editor {
    fn open(filename: &String) -> Self {
        let mut editor = Self {
            db_conn: load_project(filename),
            terminal: Terminal::new(TestBackend::new(200, 40)).expect("test backend"),
            work_state: WorkState::new(),
            app_state: AppState::new(),
        };
        editor.render();
        editor
    }

    /// Rendering fills the selection list and the tab counts that key handling reads.
    /// The status pane at the bottom shows the status message as `main` does.
    fn render(&mut self) {
        let Self {
            db_conn,
            terminal,
            work_state,
            app_state,
        } = self;
        terminal
            .draw(|frame| {
                let area = frame.area();
                let tab_pane = Rect { height: 1, ..area };
                let work_pane = Rect {
                    y: area.y + 1,
                    height: area.height - 1 - STATUS_PANE_HEIGHT,
                    ..area
                };
                let status_pane = Rect {
                    y: area.y + area.height - STATUS_PANE_HEIGHT,
                    height: STATUS_PANE_HEIGHT,
                    ..area
                };
                render_work(db_conn, frame, tab_pane, work_pane, work_state, app_state);
                render_status(frame, status_pane, app_state);
            })
            .expect("render to test backend");
    }

    fn press(&mut self, modifiers: KeyModifiers, code: KeyCode) {
        let handled = handle_work_input(
            &mut self.work_state,
            KeyEvent::new(code, modifiers),
            &self.db_conn,
        );
        assert!(handled, "work pane ignored {modifiers:?} {code:?}");
        self.render();
    }

    fn type_text(&mut self, text: &str) {
        for character in text.chars() {
            self.press(KeyModifiers::NONE, KeyCode::Char(character));
        }
    }

    fn select_component(&mut self, name: &str) {
        let index = self
            .work_state
            .current_selection_list
            .iter()
            .position(|listed| listed == name)
            .unwrap_or_else(|| panic!("component {name} not listed"));
        for _ in 0..index {
            self.press(KeyModifiers::NONE, KeyCode::Down);
        }
    }

    /// Opens the selected component with F4, moves to the purpose field,
    /// appends `suffix`, and saves with F7.
    fn append_to_purpose_and_save(&mut self, suffix: &str) {
        self.press(KeyModifiers::NONE, KeyCode::F(4));
        self.press(KeyModifiers::NONE, KeyCode::Tab);
        self.type_text(suffix);
        self.press(KeyModifiers::NONE, KeyCode::F(7));
    }
}

/// SQL cannot bind a table name,
/// so each table gets a literal query instead of a formatted one.
fn count_rows(db_conn: &Connection, table: &str) -> i64 {
    let query = match table {
        "component" => "SELECT COUNT(*) FROM component",
        "component_relation" => "SELECT COUNT(*) FROM component_relation",
        "view_packet" => "SELECT COUNT(*) FROM view_packet",
        _ => panic!("no row count query for table {table}"),
    };
    db_conn
        .query_row(query, [], |row| row.get(0))
        .unwrap_or_else(|err| panic!("cannot count {table}: {err}"))
}

/// Keeps a save file an earlier run left behind,
/// and restores it after the test.
struct SaveFileGuard {
    path: String,
    previous_content: Option<Vec<u8>>,
}

impl SaveFileGuard {
    fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            previous_content: fs::read(path).ok(),
        }
    }
}

impl Drop for SaveFileGuard {
    /// A second panic during unwinding aborts the test binary,
    /// which hides the first failure and leaves the save file unrestored.
    /// A restore error therefore fails the test loudly only when no panic is in progress.
    fn drop(&mut self) {
        let restored = match &self.previous_content {
            Some(content) => fs::write(&self.path, content),
            None => match fs::remove_file(&self.path) {
                Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
                removed => removed,
            },
        };
        if let Err(err) = restored {
            let message = format!("cannot restore {}: {err}", self.path);
            if std::thread::panicking() {
                eprintln!("{message}");
            } else {
                panic!("{message}");
            }
        }
    }
}

#[test]
fn ar_00030_tui_editor_load_lists_fixture_components() {
    let editor = Editor::open(&fixture_path());

    let listed = &editor.work_state.current_selection_list;
    let component_count = count_rows(&editor.db_conn, "component");
    assert!(component_count > 0, "fixture loaded no components");
    assert_eq!(listed.len() as i64, component_count);
    assert!(listed.iter().any(|name| name == EDITED_COMPONENT));
    // A component from the included file shows that includes load too.
    assert!(
        listed.iter().any(|name| name == "Kubernetes Cluster"),
        "{listed:?}"
    );
    let mut sorted = listed.clone();
    sorted.sort_by_key(|name| name.to_lowercase());
    assert_eq!(listed, &sorted, "selection list is not sorted by name");
}

#[test]
fn ar_00030_tui_editor_edit_updates_component() {
    let _lock = SAVE_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let save_file = save_file();
    let _save_file_guard = SaveFileGuard::new(&save_file);
    let mut editor = Editor::open(&fixture_path());
    editor.work_state.save_path = save_file.clone();
    let original = get_component_by_name(&editor.db_conn, EDITED_COMPONENT.to_string())
        .expect("fixture component");

    editor.select_component(EDITED_COMPONENT);
    editor.append_to_purpose_and_save("T39-edited");

    let edited = get_component_by_name(&editor.db_conn, EDITED_COMPONENT.to_string())
        .expect("edited component");
    assert_eq!(edited.purpose, format!("{}T39-edited", original.purpose));
    assert_eq!(edited.id, original.id);
    assert_eq!(edited.summary, original.summary);
    assert_eq!(edited.title, original.title);
}

/// The file `test_sad.xml` includes,
/// resolved relative to the directory of the including file.
const INCLUDED_FIXTURE: &str = "test_infrastructure_sad.xml";

/// The DOCTYPE line of the SAD fixtures, see `sad_xml_sql/tests/test_sad.xml`.
const SAD_DOCTYPE: &str =
    r#"<!DOCTYPE SoftwareArchitectureDocumentation SYSTEM "software_architecture_documentation.dtd">"#;

/// Returns the sorted IDs of every row in `table`.
/// SQL cannot bind a table name,
/// so each table gets a literal query instead of a formatted one.
fn sorted_ids(db_conn: &Connection, table: &str) -> Vec<u64> {
    let query = match table {
        "component" => "SELECT id FROM component ORDER BY id",
        "component_relation" => "SELECT id FROM component_relation ORDER BY id",
        "view_packet" => "SELECT viewpacket_id FROM view_packet ORDER BY viewpacket_id",
        _ => panic!("no ID query for table {table}"),
    };
    let mut statement = db_conn
        .prepare(query)
        .unwrap_or_else(|err| panic!("cannot query {table}: {err}"));
    // The binding ends the row iterator borrow of `statement` before `statement` drops.
    let ids = statement
        .query_map([], |row| row.get(0))
        .unwrap_or_else(|err| panic!("cannot query {table}: {err}"))
        .collect::<Result<Vec<u64>, _>>()
        .unwrap_or_else(|err| panic!("cannot read an ID of {table}: {err}"));
    ids
}

/// Returns the saved `Component` element with the given name,
/// from its start tag to its end tag.
fn component_element<'xml>(xml: &'xml str, name: &str) -> &'xml str {
    let name_attribute = format!(" Name=\"{name}\">");
    xml.split("<Component ")
        .skip(1)
        .find(|element| {
            element
                .lines()
                .next()
                .is_some_and(|tag| tag.ends_with(&name_attribute))
        })
        .and_then(|element| element.split("</Component>").next())
        .unwrap_or_else(|| panic!("component {name} missing in saved file"))
}

/// A project directory under the system temporary directory,
/// holding a copy of the save file next to a copy of the included fixture,
/// as the files of a real project sit side by side.
/// The reader resolves an `Include` relative to the directory of the including file,
/// and the F7 tests save to `test-output` under the crate root.
/// Reloading a copy here keeps the included file out of the crate root,
/// where the copy could overwrite a developer's file.
/// The directory is removed on drop.
struct ReloadProjectDir {
    path: PathBuf,
}

impl ReloadProjectDir {
    fn new(save_file: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "tui_editor_reload_{}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path)
                .unwrap_or_else(|err| panic!("cannot remove {}: {err}", path.display()));
        }
        fs::create_dir(&path)
            .unwrap_or_else(|err| panic!("cannot create {}: {err}", path.display()));
        let project_dir = Self { path };
        let included_source = Path::new(&fixture_path()).with_file_name(INCLUDED_FIXTURE);
        fs::copy(&included_source, project_dir.path.join(INCLUDED_FIXTURE))
            .unwrap_or_else(|err| panic!("cannot copy {}: {err}", included_source.display()));
        fs::copy(save_file, project_dir.main_file())
            .unwrap_or_else(|err| panic!("cannot copy {save_file}: {err}"));
        project_dir
    }

    fn main_file(&self) -> PathBuf {
        self.path.join(SAVE_FILE_NAME)
    }
}

impl Drop for ReloadProjectDir {
    /// Same rule as `SaveFileGuard`: no second panic while unwinding.
    fn drop(&mut self) {
        if let Err(err) = fs::remove_dir_all(&self.path) {
            let message = format!("cannot remove {}: {err}", self.path.display());
            if std::thread::panicking() {
                eprintln!("{message}");
            } else {
                panic!("{message}");
            }
        }
    }
}

/// Saves an edit with F7,
/// then reloads the saved file through `sad_xml_sql` with its included file beside it,
/// as a real project would (GEDT-AR-00002, T-40).
/// The saved main file holds only its own items,
/// so the reload, not the saved file alone, must give back every original ID.
#[test]
fn ar_00030_tui_editor_save_writes_edit_to_xml() {
    let _lock = SAVE_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let save_file = save_file();
    let _save_file_guard = SaveFileGuard::new(&save_file);
    let mut editor = Editor::open(&fixture_path());
    editor.work_state.save_path = save_file.clone();
    let original = get_component_by_name(&editor.db_conn, EDITED_COMPONENT.to_string())
        .expect("fixture component");
    let original_ids: Vec<(&str, Vec<u64>)> = ["component", "component_relation", "view_packet"]
        .into_iter()
        .map(|table| (table, sorted_ids(&editor.db_conn, table)))
        .collect();

    // A failed save leaves an earlier file in place,
    // so a stale save file from an earlier run must not stand in for this save.
    // The guard restores the earlier file afterwards.
    match fs::remove_file(&save_file) {
        Err(err) if err.kind() != ErrorKind::NotFound => {
            panic!("cannot remove stale {save_file}: {err}")
        }
        _ => {}
    }
    editor.select_component(EDITED_COMPONENT);
    editor.append_to_purpose_and_save("T39-saved");

    let saved = fs::read_to_string(&save_file).expect("F7 wrote no save file");
    assert_eq!(saved.lines().nth(1), Some(SAD_DOCTYPE), "saved file lacks the SAD DOCTYPE");
    let root_tag = saved
        .split("<SoftwareArchitectureDocumentation ")
        .nth(1)
        .and_then(|rest| rest.split('>').next())
        .expect("no SAD root element");
    assert!(root_tag.contains("Version=\"0.1.1\""), "root tag lacks Version: {root_tag}");
    assert!(!root_tag.contains("version="), "the DTD defines no lowercase 'version': {root_tag}");
    let component = component_element(&saved, EDITED_COMPONENT);
    for expected in [
        format!("<Purpose>{}T39-saved</Purpose>", original.purpose),
        format!("<Summary>{}</Summary>", original.summary),
        format!("<Title>{}</Title>", original.title),
    ] {
        assert!(
            component.contains(&expected),
            "{expected} missing in:\n{component}"
        );
    }

    let reload_dir = ReloadProjectDir::new(&save_file);
    let reloaded_db = load_project(
        &reload_dir
            .main_file()
            .to_str()
            .expect("non UTF-8 temporary path")
            .to_string(),
    );
    let reloaded = get_component_by_name(&reloaded_db, EDITED_COMPONENT.to_string())
        .expect("edited component missing after reload");
    assert_eq!(reloaded.purpose, format!("{}T39-saved", original.purpose));
    assert_eq!(reloaded.id, original.id);
    assert_eq!(reloaded.summary, original.summary);
    assert_eq!(reloaded.title, original.title);
    for (table, ids) in &original_ids {
        assert!(!ids.is_empty(), "fixture loaded no {table} rows");
        assert_eq!(
            &sorted_ids(&reloaded_db, table),
            ids,
            "reloaded save and original load differ in {table} IDs"
        );
    }
}

// Tester checks for GEDT-STORY-00019, task T-41 (GEDT-AR-00030):
// editor key handling after the swap from `tui-textarea` 0.7 to `ratatui-textarea` 0.9,
// and from `ratatui` 0.29 to 0.30.
// The component field type is private to the `work` module,
// so the tests compare the `Debug` name of the active field.

/// The summary of `EDITED_COMPONENT` in the fixture.
const EDITED_SUMMARY: &str = "Execute the battle";

impl Editor {
    /// Like `press`, but returns whether the work pane handled the key.
    /// A cursor move changes no text, so the text area reports it as not handled.
    fn send(&mut self, modifiers: KeyModifiers, code: KeyCode) -> bool {
        let handled = handle_work_input(
            &mut self.work_state,
            KeyEvent::new(code, modifiers),
            &self.db_conn,
        );
        self.render();
        handled
    }

    fn active_field(&self) -> String {
        format!("{:?}", self.work_state.component_text_areas.active_field)
    }

    fn active_pane(&self) -> String {
        format!("{:?}", self.work_state.active_pane)
    }

    /// Opens `EDITED_COMPONENT` in the editor with F4, without saving anything.
    fn open_edited_component() -> Self {
        let mut editor = Self::open(&fixture_path());
        editor.select_component(EDITED_COMPONENT);
        editor.press(KeyModifiers::NONE, KeyCode::F(4));
        editor
    }

    /// The rendered screen, one string per terminal row.
    fn screen_rows(&self) -> Vec<String> {
        let buffer = self.terminal.backend().buffer();
        let width = usize::from(buffer.area.width);
        buffer
            .content
            .chunks(width)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect())
            .collect()
    }
}

#[test]
fn t41_ar_00030_tui_editor_tab_cycles_fields_and_skips_id() {
    let mut editor = Editor::open_edited_component();
    assert_eq!(editor.active_field(), "Name");

    let mut visited = Vec::new();
    for _ in 0..6 {
        editor.press(KeyModifiers::NONE, KeyCode::Tab);
        visited.push(editor.active_field());
    }

    assert_eq!(visited, ["Purpose", "Summary", "Name", "Purpose", "Summary", "Name"]);
}

#[test]
fn t41_ar_00030_tui_editor_backtab_cycles_fields_backwards_and_skips_id() {
    let mut editor = Editor::open_edited_component();

    let mut visited = Vec::new();
    for _ in 0..6 {
        editor.press(KeyModifiers::SHIFT, KeyCode::BackTab);
        visited.push(editor.active_field());
    }

    assert_eq!(visited, ["Summary", "Purpose", "Name", "Summary", "Purpose", "Name"]);
}

/// No key sequence focuses the ID field, so typed text never reaches the ID.
#[test]
fn t41_ar_00030_tui_editor_id_field_stays_read_only() {
    let mut editor = Editor::open_edited_component();
    let original_id = editor.work_state.component_text_areas.id.lines().to_vec();
    assert!(!original_id.concat().is_empty(), "F4 loaded no ID");

    for (modifiers, code) in [
        (KeyModifiers::NONE, KeyCode::Tab),
        (KeyModifiers::SHIFT, KeyCode::BackTab),
        (KeyModifiers::SHIFT, KeyCode::BackTab),
    ] {
        editor.press(modifiers, code);
        assert_ne!(editor.active_field(), "Id");
        editor.type_text("9");
        editor.press(KeyModifiers::NONE, KeyCode::Backspace);
    }

    assert_eq!(editor.work_state.component_text_areas.id.lines(), original_id.as_slice());
    assert!(
        editor.screen_rows().iter().any(|row| row.contains("ID (read-only)")),
        "editor does not show the ID as read-only"
    );
}

/// Each mapped key in `keyevent_to_input` must reach the text area with its meaning.
#[test]
fn t41_ar_00030_tui_editor_editing_keys_reach_text_area() {
    let mut editor = Editor::open_edited_component();
    editor.press(KeyModifiers::NONE, KeyCode::Tab);
    editor.press(KeyModifiers::NONE, KeyCode::Tab);
    assert_eq!(editor.active_field(), "Summary");
    let summary_lines = |editor: &Editor| editor.work_state.component_text_areas.summary.lines().to_vec();
    assert_eq!(summary_lines(&editor), [EDITED_SUMMARY]);

    editor.press(KeyModifiers::NONE, KeyCode::Backspace);
    assert_eq!(summary_lines(&editor), ["Execute the battl"]);

    editor.send(KeyModifiers::NONE, KeyCode::Home);
    editor.type_text("X");
    editor.press(KeyModifiers::NONE, KeyCode::Delete);
    assert_eq!(summary_lines(&editor), ["Xxecute the battl"]);

    editor.send(KeyModifiers::NONE, KeyCode::End);
    editor.type_text("Y");
    editor.send(KeyModifiers::NONE, KeyCode::Left);
    editor.type_text("Z");
    editor.send(KeyModifiers::NONE, KeyCode::Right);
    editor.type_text("W");
    assert_eq!(summary_lines(&editor), ["Xxecute the battlZYW"]);

    // A terminal reports an uppercase letter with the SHIFT modifier.
    editor.press(KeyModifiers::SHIFT, KeyCode::Char('Q'));
    editor.press(KeyModifiers::NONE, KeyCode::Enter);
    editor.type_text("2");
    assert_eq!(summary_lines(&editor), ["Xxecute the battlZYWQ", "2"]);

    editor.send(KeyModifiers::NONE, KeyCode::Up);
    editor.send(KeyModifiers::NONE, KeyCode::Home);
    editor.type_text("U");
    editor.send(KeyModifiers::NONE, KeyCode::Down);
    editor.type_text("D");
    assert_eq!(summary_lines(&editor), ["UXxecute the battlZYWQ", "2D"]);

    let screen = editor.screen_rows();
    assert!(
        screen.iter().any(|row| row.contains("UXxecute the battlZYWQ")),
        "edited summary not rendered:\n{}",
        screen.join("\n")
    );
}

/// Esc leaves the editor without writing the edit to the database.
#[test]
fn t41_ar_00030_tui_editor_esc_discards_unsaved_edit() {
    let mut editor = Editor::open_edited_component();
    editor.press(KeyModifiers::NONE, KeyCode::Tab);
    editor.type_text("T41-unsaved");

    editor.press(KeyModifiers::NONE, KeyCode::Esc);

    let stored = get_component_by_name(&editor.db_conn, EDITED_COMPONENT.to_string())
        .expect("fixture component");
    assert_eq!(stored.purpose, "");
    assert_eq!(stored.summary, EDITED_SUMMARY);
}

/// T-54 keeps production behaviour: F7 saves to `tui_out.xml` in the working directory,
/// and only the tests above point `save_path` into `test-output` (AR-00030).
#[test]
fn t54_ar_00030_tui_editor_default_save_path_is_tui_out_xml() {
    assert_eq!(WorkState::new().save_path, SAVE_FILE_NAME);
    let editor = Editor::open(&fixture_path());
    assert_eq!(editor.work_state.save_path, SAVE_FILE_NAME);
}

// Save failure checks for GEDT-STORY-00002, task T-47:
// GEDT-FR-00046 to GEDT-FR-00048 applied to `tui_editor` through the shared core
// (GEDT-AR-00030).
// The fixtures need no terminal and no root:
// a directory that refuses new files, and the Linux device `/dev/full`,
// which fails every write with "no space left", like a full disk.

/// A Linux character device that fails every write with `ENOSPC`.
const FULL_DISK_FILE: &str = "/dev/full";

/// Returns whether `dir` refuses a new file, and removes the probe file when it does not.
fn refuses_new_files(dir: &Path) -> bool {
    let probe = dir.join(".t47_write_probe");
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
    {
        Ok(_) => {
            fs::remove_file(&probe)
                .unwrap_or_else(|err| panic!("cannot remove {}: {err}", probe.display()));
            false
        }
        Err(err)
            if matches!(
                err.kind(),
                ErrorKind::PermissionDenied | ErrorKind::ReadOnlyFilesystem
            ) =>
        {
            true
        }
        Err(err) => panic!("cannot probe {}: {err}", dir.display()),
    }
}

/// A directory that refuses new files, for the read-only directory fixture.
/// Mode 0o555 stops an unprivileged user.
/// The tool image runs the tests as root of a rootless user namespace,
/// whose `CAP_DAC_OVERRIDE` ignores the mode,
/// so the crate root, which the tool image mounts read-only, stands in there.
/// Drop restores the mode, so the directory can be removed.
struct ReadOnlyDir {
    path: PathBuf,
    chmod_dir: PathBuf,
}

impl ReadOnlyDir {
    fn new() -> Self {
        use std::os::unix::fs::PermissionsExt;

        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let chmod_dir = crate_root
            .join("test-output")
            .join(format!("t47_read_only_{}", std::process::id()));
        if chmod_dir.exists() {
            fs::set_permissions(&chmod_dir, fs::Permissions::from_mode(0o755))
                .unwrap_or_else(|err| panic!("cannot chmod {}: {err}", chmod_dir.display()));
            fs::remove_dir_all(&chmod_dir)
                .unwrap_or_else(|err| panic!("cannot remove {}: {err}", chmod_dir.display()));
        }
        fs::create_dir_all(&chmod_dir)
            .unwrap_or_else(|err| panic!("cannot create {}: {err}", chmod_dir.display()));
        fs::set_permissions(&chmod_dir, fs::Permissions::from_mode(0o555))
            .unwrap_or_else(|err| panic!("cannot chmod {}: {err}", chmod_dir.display()));
        let path = [chmod_dir.clone(), crate_root]
            .into_iter()
            .find(|candidate| refuses_new_files(candidate))
            .unwrap_or_else(|| {
                panic!("no read-only directory: run the tests unprivileged or in the tool image")
            });
        Self { path, chmod_dir }
    }

    /// A name no developer file uses, because the fallback directory is the crate root.
    fn save_file(&self) -> String {
        self.path
            .join("t47_read_only_save.xml")
            .to_str()
            .expect("non UTF-8 crate path")
            .to_string()
    }
}

impl Drop for ReadOnlyDir {
    /// Same rule as `SaveFileGuard`: no second panic while unwinding.
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;

        let cleanup = fs::set_permissions(&self.chmod_dir, fs::Permissions::from_mode(0o755))
            .and_then(|()| fs::remove_dir_all(&self.chmod_dir));
        if let Err(err) = cleanup {
            let message = format!("cannot remove {}: {err}", self.chmod_dir.display());
            if std::thread::panicking() {
                eprintln!("{message}");
            } else {
                panic!("{message}");
            }
        }
    }
}

/// Saves an edit with F7 to `failing_save_file`, which cannot be written,
/// and checks that the screen shows the error, naming the file and `expected_cause`.
/// Then checks that the edit stays in the session,
/// so the next F7 to a writable file saves the edit and clears the error.
fn assert_failed_save_is_shown_and_keeps_edit(failing_save_file: &str, expected_cause: &str) {
    let _lock = SAVE_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let save_file = save_file();
    let _save_file_guard = SaveFileGuard::new(&save_file);
    let mut editor = Editor::open(&fixture_path());
    editor.work_state.save_path = failing_save_file.to_string();
    let original = get_component_by_name(&editor.db_conn, EDITED_COMPONENT.to_string())
        .expect("fixture component");

    editor.select_component(EDITED_COMPONENT);
    editor.append_to_purpose_and_save("T47-kept");

    let expected_error = format!("cannot save {failing_save_file}: {expected_cause}");
    assert!(
        editor.app_state.status_message.contains(&expected_error),
        "status lacks {expected_error:?}: {}",
        editor.app_state.status_message
    );
    let screen = editor.screen_rows();
    assert!(
        screen.iter().any(|row| row.contains(&expected_error)),
        "screen lacks {expected_error:?}:\n{}",
        screen.join("\n")
    );
    let kept = get_component_by_name(&editor.db_conn, EDITED_COMPONENT.to_string())
        .expect("edited component");
    assert_eq!(kept.purpose, format!("{}T47-kept", original.purpose));
    assert!(
        editor.work_state.is_modified(),
        "a failed save cleared the modified flag"
    );
    // The failed F7 keeps the editor pane open, so F7 is pressed again without F4.
    assert_eq!(editor.active_pane(), "Editor");

    match fs::remove_file(&save_file) {
        Err(err) if err.kind() != ErrorKind::NotFound => {
            panic!("cannot remove stale {save_file}: {err}")
        }
        _ => {}
    }
    editor.work_state.save_path = save_file.clone();
    editor.press(KeyModifiers::NONE, KeyCode::F(7));

    let saved = fs::read_to_string(&save_file).expect("F7 wrote no save file");
    let component = component_element(&saved, EDITED_COMPONENT);
    let expected_purpose = format!("<Purpose>{}T47-kept</Purpose>", original.purpose);
    assert!(
        component.contains(&expected_purpose),
        "{expected_purpose} missing in:\n{component}"
    );
    assert!(
        !editor.app_state.status_message.contains("cannot save"),
        "a successful save kept the error: {}",
        editor.app_state.status_message
    );
}

/// Returns the error that creating `failing_save_file` gives,
/// the cause a failed save shows.
/// `create_new` never truncates an existing file,
/// and a file created by an unexpected success is removed before the test fails.
fn expected_create_error(failing_save_file: &str) -> String {
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(failing_save_file)
    {
        Ok(_) => {
            fs::remove_file(failing_save_file)
                .unwrap_or_else(|err| panic!("cannot remove {failing_save_file}: {err}"));
            panic!("{failing_save_file} accepted a new file");
        }
        Err(err) => err.to_string(),
    }
}

#[test]
fn t47_fr_00047_fr_00048_read_only_directory_save_shows_error_and_keeps_edit() {
    let read_only_dir = ReadOnlyDir::new();
    let failing_save_file = read_only_dir.save_file();
    let expected_cause = expected_create_error(&failing_save_file);

    assert_failed_save_is_shown_and_keeps_edit(&failing_save_file, &expected_cause);

    assert!(
        !Path::new(&failing_save_file).exists(),
        "{failing_save_file} was written"
    );
}

#[test]
fn t47_fr_00047_fr_00048_full_disk_save_shows_error_and_keeps_edit() {
    let expected_cause = fs::write(FULL_DISK_FILE, b"probe")
        .expect_err("a write to the full disk succeeded")
        .to_string();

    assert_failed_save_is_shown_and_keeps_edit(FULL_DISK_FILE, &expected_cause);
}

// Tester checks for GEDT-STORY-00002, task T-47:
// the save error stays on screen, once, until a save succeeds (GEDT-FR-00047),
// and every edit made across failed saves stays in the session (GEDT-FR-00048).

/// Counts the screen rows and status messages that hold `text`.
fn count_on_screen(editor: &Editor, text: &str) -> (usize, usize) {
    let rows = editor
        .screen_rows()
        .iter()
        .filter(|row| row.contains(text))
        .count();
    let in_status = editor.app_state.status_message.matches(text).count();
    (rows, in_status)
}

#[test]
fn t47_fr_00047_save_error_stays_once_on_screen_across_renders_and_navigation() {
    let _lock = SAVE_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut editor = Editor::open(&fixture_path());
    editor.work_state.save_path = FULL_DISK_FILE.to_string();

    editor.select_component(EDITED_COMPONENT);
    editor.append_to_purpose_and_save("T47-nav");
    let expected_error = format!("cannot save {FULL_DISK_FILE}: ");
    assert_eq!(count_on_screen(&editor, &expected_error), (1, 1));

    for _ in 0..3 {
        editor.render();
    }
    // The failed F7 keeps the editor pane open,
    // so Esc returns to the list before navigating.
    editor.press(KeyModifiers::NONE, KeyCode::Esc);
    editor.press(KeyModifiers::NONE, KeyCode::Down);
    editor.press(KeyModifiers::NONE, KeyCode::Up);

    assert_eq!(
        count_on_screen(&editor, &expected_error),
        (1, 1),
        "status: {}",
        editor.app_state.status_message
    );
}

#[test]
fn t47_fr_00047_fr_00048_missing_directory_save_shows_error_and_keeps_edit() {
    let missing_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test-output")
        .join(format!("t47_missing_dir_{}", std::process::id()));
    assert!(!missing_dir.exists(), "{} exists", missing_dir.display());
    let failing_save_file = missing_dir
        .join("tui_out.xml")
        .to_str()
        .expect("non UTF-8 crate path")
        .to_string();
    let expected_cause = expected_create_error(&failing_save_file);

    assert_failed_save_is_shown_and_keeps_edit(&failing_save_file, &expected_cause);

    assert!(
        !missing_dir.exists(),
        "{} was created",
        missing_dir.display()
    );
}

#[test]
fn t47_fr_00048_edits_across_two_failed_saves_stay_until_a_save_succeeds() {
    let _lock = SAVE_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let save_file = save_file();
    let _save_file_guard = SaveFileGuard::new(&save_file);
    let mut editor = Editor::open(&fixture_path());
    editor.work_state.save_path = FULL_DISK_FILE.to_string();
    let original = get_component_by_name(&editor.db_conn, EDITED_COMPONENT.to_string())
        .expect("fixture component");

    editor.select_component(EDITED_COMPONENT);
    editor.append_to_purpose_and_save("T47-first");
    // The failed F7 keeps the editor pane open on the purpose field,
    // so F4 and Tab are not pressed.
    assert_eq!(editor.active_pane(), "Editor");
    assert_eq!(editor.active_field(), "Purpose");
    // Without F4, the text area still holds the first edit,
    // so the session database is checked directly.
    let after_first = get_component_by_name(&editor.db_conn, EDITED_COMPONENT.to_string())
        .expect("edited component");
    assert_eq!(
        after_first.purpose,
        format!("{}T47-first", original.purpose)
    );
    editor.type_text("T47-second");
    editor.press(KeyModifiers::NONE, KeyCode::F(7));
    assert!(
        editor.app_state.status_message.contains("cannot save"),
        "second failed save not shown: {}",
        editor.app_state.status_message
    );

    match fs::remove_file(&save_file) {
        Err(err) if err.kind() != ErrorKind::NotFound => {
            panic!("cannot remove stale {save_file}: {err}")
        }
        _ => {}
    }
    editor.work_state.save_path = save_file.clone();
    editor.press(KeyModifiers::NONE, KeyCode::F(7));

    let saved = fs::read_to_string(&save_file).expect("F7 wrote no save file");
    let expected_purpose = format!("<Purpose>{}T47-firstT47-second</Purpose>", original.purpose);
    let component = component_element(&saved, EDITED_COMPONENT);
    assert!(
        component.contains(&expected_purpose),
        "{expected_purpose} missing in:\n{component}"
    );
    assert!(
        !editor.app_state.status_message.contains("cannot save"),
        "a successful save kept the error: {}",
        editor.app_state.status_message
    );
}

// Developer checks for GEDT-STORY-00002, task T-47, round 2:
// the modified flag and its status line mark (GEDT-FR-00048),
// and the pane after F7 (GEDT-FR-00049).

/// Whether the status message and exactly one screen row show the modified mark.
fn shows_modified_mark(editor: &Editor) -> bool {
    match count_on_screen(editor, MODIFIED_MARK) {
        (0, 0) => false,
        (1, 1) => true,
        counts => panic!("modified mark shown {counts:?} times, rows and status"),
    }
}

/// Removes `save_file` left by an earlier test, so a test sees only its own save.
fn remove_stale(save_file: &str) {
    match fs::remove_file(save_file) {
        Err(err) if err.kind() != ErrorKind::NotFound => {
            panic!("cannot remove stale {save_file}: {err}")
        }
        _ => {}
    }
}

#[test]
fn t47_fr_00048_modified_flag_is_off_after_load_and_on_after_edit() {
    let mut editor = Editor::open(&fixture_path());
    assert!(!editor.work_state.is_modified());
    assert!(!shows_modified_mark(&editor));

    editor.select_component(EDITED_COMPONENT);
    editor.press(KeyModifiers::NONE, KeyCode::F(4));
    assert!(
        !editor.work_state.is_modified(),
        "F4 alone marked the project modified"
    );
    assert!(!shows_modified_mark(&editor));

    editor.press(KeyModifiers::NONE, KeyCode::Tab);
    editor.type_text("T47-mark");
    assert!(editor.work_state.is_modified());
    assert!(shows_modified_mark(&editor));
}

#[test]
fn t47_fr_00048_failed_save_keeps_modified_mark_and_successful_save_clears_it() {
    let _lock = SAVE_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let save_file = save_file();
    let _save_file_guard = SaveFileGuard::new(&save_file);
    let mut editor = Editor::open(&fixture_path());
    editor.work_state.save_path = FULL_DISK_FILE.to_string();

    editor.select_component(EDITED_COMPONENT);
    editor.append_to_purpose_and_save("T47-flag");
    assert!(
        editor.work_state.is_modified(),
        "a failed save cleared the modified flag"
    );
    assert!(shows_modified_mark(&editor));

    // The edit is in the session database after the failed save, so Esc keeps the mark.
    editor.press(KeyModifiers::NONE, KeyCode::Esc);
    assert!(
        editor.work_state.is_modified(),
        "Esc after a failed save cleared the flag"
    );
    assert!(shows_modified_mark(&editor));

    remove_stale(&save_file);
    editor.work_state.save_path = save_file.clone();
    editor.press(KeyModifiers::NONE, KeyCode::F(4));
    editor.press(KeyModifiers::NONE, KeyCode::F(7));
    assert!(fs::metadata(&save_file).is_ok(), "F7 wrote no save file");
    assert!(
        !editor.work_state.is_modified(),
        "a successful save kept the flag"
    );
    assert!(!shows_modified_mark(&editor));
}

#[test]
fn t47_fr_00048_esc_discards_unsaved_edit_and_its_modified_mark() {
    let mut editor = Editor::open(&fixture_path());
    editor.select_component(EDITED_COMPONENT);
    editor.press(KeyModifiers::NONE, KeyCode::F(4));
    editor.press(KeyModifiers::NONE, KeyCode::Tab);
    editor.type_text("T47-esc");
    assert!(shows_modified_mark(&editor));

    editor.press(KeyModifiers::NONE, KeyCode::Esc);
    assert!(
        !editor.work_state.is_modified(),
        "Esc kept the flag of a discarded edit"
    );
    assert!(!shows_modified_mark(&editor));
}

/// GEDT-FR-00049: after a failed F7 save, the editor pane stays open.
#[test]
fn t47_fr_00049_failed_save_keeps_editor_pane_open() {
    let _lock = SAVE_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut editor = Editor::open(&fixture_path());
    editor.work_state.save_path = FULL_DISK_FILE.to_string();

    editor.select_component(EDITED_COMPONENT);
    editor.append_to_purpose_and_save("T47-pane");
    assert!(editor.app_state.status_message.contains("cannot save"));
    assert_eq!(editor.active_pane(), "Editor");
    assert_eq!(editor.active_field(), "Purpose");
    let purpose = editor
        .work_state
        .component_text_areas
        .purpose
        .lines()
        .join("\n");
    assert!(
        purpose.ends_with("T47-pane"),
        "editor lost the edit: {purpose:?}"
    );
}

/// Guards existing behaviour noted in REQUIREMENTS.md, not a requirement:
/// after a successful F7 save, `tui_editor` returns to the previous pane.
/// The test keeps the GEDT-FR-00049 change from altering the successful save path.
#[test]
fn t47_existing_successful_save_returns_to_previous_pane() {
    let _lock = SAVE_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let save_file = save_file();
    let _save_file_guard = SaveFileGuard::new(&save_file);
    remove_stale(&save_file);
    let mut editor = Editor::open(&fixture_path());
    editor.work_state.save_path = save_file.clone();

    editor.select_component(EDITED_COMPONENT);
    let pane_before_edit = editor.active_pane();
    editor.append_to_purpose_and_save("T47-back");
    assert!(fs::metadata(&save_file).is_ok(), "F7 wrote no save file");
    assert_eq!(editor.active_pane(), pane_before_edit);
    assert_eq!(pane_before_edit, "Selector");
}

// Tester checks for GEDT-STORY-00002, task T-47, round 2 (GEDT-FR-00047, GEDT-FR-00048):
// which keys set the modified mark, and that only a successful save clears it.

/// A key that changes no text is not an edit, so the mark stays off.
#[test]
fn t47_fr_00048_keys_without_text_change_leave_mark_off() {
    let mut editor = Editor::open_edited_component();
    assert_eq!(editor.active_field(), "Name");

    for (modifiers, code) in [
        (KeyModifiers::NONE, KeyCode::Home),
        (KeyModifiers::NONE, KeyCode::Backspace),
        (KeyModifiers::NONE, KeyCode::End),
        (KeyModifiers::NONE, KeyCode::Delete),
        (KeyModifiers::NONE, KeyCode::Left),
        (KeyModifiers::NONE, KeyCode::Right),
        (KeyModifiers::NONE, KeyCode::Up),
        (KeyModifiers::NONE, KeyCode::Down),
    ] {
        editor.send(modifiers, code);
    }
    editor.press(KeyModifiers::NONE, KeyCode::Tab);
    editor.press(KeyModifiers::SHIFT, KeyCode::BackTab);

    assert_eq!(
        editor.work_state.component_text_areas.name.lines(),
        [EDITED_COMPONENT],
        "a key changed the name"
    );
    assert!(
        !editor.work_state.is_modified(),
        "a key without a text change marked the project modified"
    );
    assert!(!shows_modified_mark(&editor));
}

/// After a failed save, the edit is in the session database,
/// so neither a discarded second edit nor another component clears the mark.
#[test]
fn t47_fr_00048_mark_survives_esc_of_later_edit_and_component_change() {
    let _lock = SAVE_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut editor = Editor::open(&fixture_path());
    editor.work_state.save_path = FULL_DISK_FILE.to_string();

    editor.select_component(EDITED_COMPONENT);
    editor.append_to_purpose_and_save("T47-survive");
    assert!(editor.app_state.status_message.contains("cannot save"));
    editor.press(KeyModifiers::NONE, KeyCode::Esc);

    editor.press(KeyModifiers::NONE, KeyCode::F(4));
    editor.type_text("T47-discarded");
    editor.press(KeyModifiers::NONE, KeyCode::Esc);
    assert!(
        editor.work_state.is_modified(),
        "Esc of a later edit cleared the mark of the failed save"
    );
    assert!(shows_modified_mark(&editor));

    editor.press(KeyModifiers::NONE, KeyCode::Up);
    editor.press(KeyModifiers::NONE, KeyCode::F(4));
    editor.press(KeyModifiers::NONE, KeyCode::Esc);
    assert!(
        editor.work_state.is_modified(),
        "opening another component cleared the mark"
    );
    assert!(shows_modified_mark(&editor));
    let stored = get_component_by_name(&editor.db_conn, EDITED_COMPONENT.to_string())
        .expect("edited component");
    assert!(
        stored.purpose.ends_with("T47-survive"),
        "the failed save lost the edit: {:?}",
        stored.purpose
    );
}

/// A failed F7 without an edit still shows the error (GEDT-FR-00047),
/// and marks nothing modified, because nothing changed.
#[test]
fn t47_fr_00047_failed_save_without_edit_shows_error() {
    let _lock = SAVE_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut editor = Editor::open_edited_component();
    editor.work_state.save_path = FULL_DISK_FILE.to_string();

    editor.press(KeyModifiers::NONE, KeyCode::F(7));

    let expected_error = format!("cannot save {FULL_DISK_FILE}: ");
    assert_eq!(count_on_screen(&editor, &expected_error), (1, 1));
    assert_eq!(editor.active_pane(), "Editor");
    assert!(!shows_modified_mark(&editor));
}

/// A first successful save of an edit clears the mark and leaves no error on screen.
#[test]
fn t47_fr_00048_successful_save_of_edit_clears_mark() {
    let _lock = SAVE_FILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let save_file = save_file();
    let _save_file_guard = SaveFileGuard::new(&save_file);
    remove_stale(&save_file);
    let mut editor = Editor::open(&fixture_path());
    editor.work_state.save_path = save_file.clone();

    editor.select_component(EDITED_COMPONENT);
    editor.append_to_purpose_and_save("T47-clean");

    assert!(fs::metadata(&save_file).is_ok(), "F7 wrote no save file");
    assert!(!editor.work_state.is_modified());
    assert!(!shows_modified_mark(&editor));
    assert!(!editor.app_state.status_message.contains("cannot save"));
}
