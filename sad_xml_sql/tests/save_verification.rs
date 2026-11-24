//! Tester checks for the save path of GEDT-STORY-00002, task T-40,
//! that complement `save_round_trip.rs`.
//!
//! The fixture has no component with both a `TeamId` and a `Title`,
//! so one test adds a team to a titled component to check the DTD order of the two.
//!
//! An edit stores user text in the database, and `dump_db_to_xml` writes that text to XML.
//! Markup in the text must come back as text after a reload,
//! and must never add elements or attributes to the saved file (GEDT-AR-00001, GEDT-AR-00002).

use rusqlite::Connection;
use sad_xml_sql::db_dump_to_xml::dump_db_to_xml;
use sad_xml_sql::{
    convert_from_address_to_id, db_create_in_mem_db, db_populate_from_xml, get_component_by_name,
    update_component_by_id,
};
use std::fs;
use std::path::{Path, PathBuf};
use xmltree::Element;

/// The file `tests/test_sad.xml` includes, resolved relative to the including file.
const INCLUDED_FIXTURE: &str = "test_infrastructure_sad.xml";

/// A fixture component with a purpose, a summary, and a title.
const EDITED_COMPONENT: &str = "BattleResolver";

/// Text that closes the element, opens a forged element and a comment,
/// and holds an entity reference and an internal-subset fragment.
const MARKUP_TEXT: &str =
    "x</Purpose><Injected Id=\"9.9.9.9\"/><!-- c -->&amp;&lt; ]><!ENTITY e \"v\">";

/// A name that tries to close its attribute value and add an attribute.
const MARKUP_NAME: &str = "BattleResolver\" Injected=\"1\" x=\">";

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn load(xml_file: &Path) -> Connection {
    let db_conn = Connection::open_in_memory().expect("cannot open in-memory database");
    db_create_in_mem_db(&db_conn);
    db_populate_from_xml(&db_conn, &xml_file.to_str().expect("non UTF-8 path").to_string());
    db_conn
}

/// Removes the per-test work directory on drop, also when an assertion fails.
struct WorkDir(PathBuf);

impl WorkDir {
    fn new(test_name: &str) -> Self {
        let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("{test_name}_{}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path)
                .unwrap_or_else(|err| panic!("cannot remove {}: {err}", path.display()));
        }
        fs::create_dir(&path)
            .unwrap_or_else(|err| panic!("cannot create {}: {err}", path.display()));
        fs::copy(fixture_dir().join(INCLUDED_FIXTURE), path.join(INCLUDED_FIXTURE))
            .expect("cannot copy the included fixture");
        Self(path)
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        if let Err(err) = fs::remove_dir_all(&self.0) {
            eprintln!("cannot remove {}: {err}", self.0.display());
        }
    }
}

fn contains_element_named(element: &Element, name: &str) -> bool {
    element.name == name
        || element
            .children
            .iter()
            .filter_map(|node| node.as_element())
            .any(|child| contains_element_named(child, name))
}

fn contains_attribute_named(element: &Element, name: &str) -> bool {
    element.attributes.contains_key(name)
        || element
            .children
            .iter()
            .filter_map(|node| node.as_element())
            .any(|child| contains_attribute_named(child, name))
}

#[test]
fn ar_00001_saved_markup_in_edited_text_stays_text() {
    let work_dir = WorkDir::new("ar_00001_markup_text");
    let db_conn = load(&fixture_dir().join("test_sad.xml"));
    let mut component =
        get_component_by_name(&db_conn, EDITED_COMPONENT.to_string()).expect("fixture component");
    component.name = MARKUP_NAME.to_string();
    component.purpose = MARKUP_TEXT.to_string();
    component.summary = MARKUP_TEXT.to_string();
    component.title = MARKUP_TEXT.to_string();
    update_component_by_id(&db_conn, &component);
    let saved_file = work_dir.0.join("saved_sad.xml");
    dump_db_to_xml(&db_conn, saved_file.to_str().expect("non UTF-8 path"))
        .expect("dump_db_to_xml failed");

    let saved_text = fs::read_to_string(&saved_file).expect("cannot read saved file");
    let root = Element::parse(saved_text.as_bytes()).expect("saved file is not well-formed XML");
    assert!(!contains_element_named(&root, "Injected"), "edited text added an element");
    assert!(!contains_attribute_named(&root, "Injected"), "edited name added an attribute");
    assert!(!saved_text.contains("<!ENTITY"), "edited text reached the saved file as markup");

    let reloaded = get_component_by_name(&load(&saved_file), MARKUP_NAME.to_string())
        .expect("edited component missing after reload");
    assert_eq!(reloaded.id, component.id);
    assert_eq!(reloaded.purpose, MARKUP_TEXT);
    assert_eq!(reloaded.summary, MARKUP_TEXT);
    assert_eq!(reloaded.title, MARKUP_TEXT);
}

/// The saved file must not reveal where the project was loaded from or saved to.
#[test]
fn ar_00001_saved_file_holds_no_machine_paths() {
    let work_dir = WorkDir::new("ar_00001_no_paths");
    let fixture = fixture_dir().join("test_sad.xml");
    let saved_file = work_dir.0.join("saved_sad.xml");
    dump_db_to_xml(&load(&fixture), saved_file.to_str().expect("non UTF-8 path"))
        .expect("dump_db_to_xml failed");

    let saved_text = fs::read_to_string(&saved_file).expect("cannot read saved file");
    for machine_path in [
        env!("CARGO_MANIFEST_DIR"),
        env!("CARGO_TARGET_TMPDIR"),
        fixture.to_str().expect("non UTF-8 path"),
        "tests/test_sad.xml",
    ] {
        assert!(
            !saved_text.contains(machine_path),
            "saved file contains the path {machine_path}"
        );
    }
}

/// The DTD sequence for `Component` puts `TeamId` before `Title` (GEDT-AR-00001).
#[test]
fn ar_00001_saved_component_team_id_precedes_title() {
    let work_dir = WorkDir::new("ar_00001_team_id_order");
    let db_conn = load(&fixture_dir().join("test_sad.xml"));
    let team_id = convert_from_address_to_id("1.0.3.1".to_string(), "test");
    let updated = db_conn
        .execute(
            "UPDATE component SET team_id = ?1 WHERE name = ?2",
            (team_id, EDITED_COMPONENT),
        )
        .expect("cannot set the component team");
    assert_eq!(updated, 1, "fixture lacks component {EDITED_COMPONENT}");
    let saved_file = work_dir.0.join("saved_sad.xml");
    dump_db_to_xml(&db_conn, saved_file.to_str().expect("non UTF-8 path"))
        .expect("dump_db_to_xml failed");

    let saved_text = fs::read_to_string(&saved_file).expect("cannot read saved file");
    let root = Element::parse(saved_text.as_bytes()).expect("saved file is not well-formed XML");
    let component = root
        .children
        .iter()
        .filter_map(|node| node.as_element())
        .find(|element| {
            element.name == "Component"
                && element.attributes.get("Name").map(String::as_str) == Some(EDITED_COMPONENT)
        })
        .expect("edited component missing in saved file");
    let child_names: Vec<&str> = component
        .children
        .iter()
        .filter_map(|node| node.as_element())
        .map(|child| child.name.as_str())
        .collect();
    let position = |name: &str| {
        child_names
            .iter()
            .position(|child| *child == name)
            .unwrap_or_else(|| panic!("<{name}> missing in {child_names:?}"))
    };
    assert!(position("TeamId") < position("Title"), "TeamId after Title: {child_names:?}");

    let reloaded = get_component_by_name(&load(&saved_file), EDITED_COMPONENT.to_string())
        .expect("component missing after reload");
    assert_eq!(reloaded.team_id, team_id);
}
