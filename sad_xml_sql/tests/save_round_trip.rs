//! Save round-trip tests for `dump_db_to_xml` (GEDT-STORY-00002, task T-40).
//!
//! Each test loads the fixture `tests/test_sad.xml`, saves it with `dump_db_to_xml`,
//! and checks the saved file against GEDT-AR-00001, GEDT-AR-00002, and GEDT-AR-00030.
//! No approved crate validates against a DTD,
//! so the tests check the structural DTD rules the dump must follow directly.
//! The saved file goes to a per-test work directory,
//! next to a copy of the included file, so a reload resolves the `Include` the same way.

use rusqlite::Connection;
use sad_xml_sql::db_dump_to_xml::dump_db_to_xml;
use sad_xml_sql::{db_create_in_mem_db, db_populate_from_xml};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use xmltree::Element;

/// The file `tests/test_sad.xml` includes, resolved relative to the including file.
const INCLUDED_FIXTURE: &str = "test_infrastructure_sad.xml";

/// The `FileId` of `tests/test_sad.xml`.
const MAIN_FILE_ID: u64 = 1;

/// The DOCTYPE line the SAD fixtures use,
/// see `tests/software_architecture_documentation.dtd`.
const SAD_DOCTYPE: &str =
    r#"<!DOCTYPE SoftwareArchitectureDocumentation SYSTEM "software_architecture_documentation.dtd">"#;

/// Child elements whose text is an `a.b.c.d` address, as the XML reader expects.
const ADDRESS_ELEMENTS: &[&str] = &[
    "ComponentAId",
    "ComponentBId",
    "ComponentId",
    "TeamId",
    "ViewPacketId",
];

/// Root children in DTD order.
/// The order is the DTD sequence for `SoftwareArchitectureDocumentation`.
const ROOT_CHILD_ORDER: &[&str] = &[
    "Document",
    "Include",
    "ViewPacket",
    "Component",
    "ComponentRelation",
    "Behavior",
    "Requirement",
    "Team",
];

/// Children of each element in the order its DTD sequence allows.
/// Only the elements the dump writes are listed.
const ELEMENT_CHILD_ORDER: &[(&str, &[&str])] = &[
    ("Behavior", &["ViewPacketId", "Description", "DiagramKey"]),
    (
        "Component",
        &[
            "ArtifactRepository",
            "ArchitectureDecisionId",
            "BehaviorDescription",
            "BuildCycle",
            "Capacity",
            "Contact",
            "File",
            "Functionality",
            "ImplementationConstraints",
            "Interface",
            "License",
            "Performance",
            "Reliability",
            "ResourceRequirement",
            "Purpose",
            "SchedulerPriority",
            "Security",
            "SourceRepository",
            "Summary",
            "TeamId",
            "TestInformation",
            "Title",
            "VisibilityOfInterfaces",
        ],
    ),
    (
        "ComponentRelation",
        &[
            "ComponentAId",
            "ComponentBId",
            "Key",
            "PropertyOfRelation",
            "ConnectionType",
            "RelationText",
            "RelationDescription",
            "Style",
        ],
    ),
    ("Document", &["FileId", "Title", "Issue", "Summary"]),
    ("Include", &[]),
    ("Team", &["Description", "Member"]),
    (
        "ViewPacket",
        &[
            "Title",
            "ComponentRecurseLevel",
            "Introduction",
            "ComponentId",
            "PrimaryDisplayKey",
            "ContextModelKey",
            "DiagramKey",
            "ParentViewPacketId",
            "ChildViewPacketId",
            "SiblingViewPacketId",
            "TeamId",
        ],
    ),
];

/// IDs that identify the saved content,
/// each list sorted so two databases compare directly.
#[derive(Debug, PartialEq)]
struct SavedIds {
    behaviors: Vec<u64>,
    component_relations: Vec<(u64, u64, u64, String)>,
    components: Vec<u64>,
    teams: Vec<u64>,
    viewpackets: Vec<u64>,
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn fixture_path() -> PathBuf {
    fixture_dir().join("test_sad.xml")
}

/// Returns an empty directory for one test, so parallel tests never share files.
/// The directory sits in the Cargo target directory,
/// so a leftover from an earlier run is removed.
fn fresh_work_dir(test_name: &str) -> PathBuf {
    let work_dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("{test_name}_{}", std::process::id()));
    if work_dir.exists() {
        fs::remove_dir_all(&work_dir)
            .unwrap_or_else(|err| panic!("cannot remove {}: {err}", work_dir.display()));
    }
    fs::create_dir(&work_dir)
        .unwrap_or_else(|err| panic!("cannot create {}: {err}", work_dir.display()));
    work_dir
}

fn load(xml_file: &Path) -> Connection {
    let db_conn = Connection::open_in_memory().expect("cannot open in-memory database");
    db_create_in_mem_db(&db_conn);
    db_populate_from_xml(&db_conn, &xml_file.to_str().expect("non UTF-8 path").to_string());
    db_conn
}

fn save(db_conn: &Connection, saved_file: &Path) {
    dump_db_to_xml(db_conn, saved_file.to_str().expect("non UTF-8 path"))
        .expect("dump_db_to_xml failed");
}

/// Loads the fixture and saves it unedited into `work_dir`, returning the saved file.
/// The included file is copied next to the saved file, so reloading the saved file works.
fn save_fixture(work_dir: &Path) -> PathBuf {
    fs::copy(fixture_dir().join(INCLUDED_FIXTURE), work_dir.join(INCLUDED_FIXTURE))
        .expect("cannot copy the included fixture");
    let saved_file = work_dir.join("saved_sad.xml");
    save(&load(&fixture_path()), &saved_file);
    saved_file
}

fn parse_file(xml_file: &Path) -> Element {
    let xml_text = fs::read_to_string(xml_file)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", xml_file.display()));
    Element::parse(xml_text.as_bytes())
        .unwrap_or_else(|err| panic!("{} is not well-formed XML: {err}", xml_file.display()))
}

fn root_children_named<'root>(root: &'root Element, name: &'root str) -> Vec<&'root Element> {
    child_elements(root).filter(|element| element.name == name).collect()
}

fn child_text(element: &Element, child_name: &str) -> Option<String> {
    element.get_child(child_name).and_then(|child| child.get_text()).map(|text| text.into_owned())
}

/// Maps each `Id` attribute of the root children named `name` to `value_of` that element.
fn values_by_id<T>(
    root: &Element,
    name: &str,
    value_of: impl Fn(&Element) -> T,
) -> BTreeMap<String, T> {
    root_children_named(root, name)
        .into_iter()
        .map(|element| (element.attributes["Id"].clone(), value_of(element)))
        .collect()
}

/// Lists each fixture file with its content and modification time,
/// so a comparison detects a file the dump created, changed, or rewrote unchanged.
fn fixture_dir_snapshot() -> BTreeMap<PathBuf, (Vec<u8>, SystemTime)> {
    fs::read_dir(fixture_dir())
        .expect("cannot list the fixture directory")
        .map(|entry| {
            let path = entry.expect("cannot read a fixture directory entry").path();
            let content = fs::read(&path).unwrap_or_default();
            let modified = fs::metadata(&path)
                .and_then(|metadata| metadata.modified())
                .expect("cannot read a fixture modification time");
            (path, (content, modified))
        })
        .collect()
}

fn sorted_ids(db_conn: &Connection, query: &str) -> Vec<u64> {
    let mut statement = db_conn.prepare(query).expect("invalid query");
    let mut ids = statement
        .query_map([], |row| row.get(0))
        .expect("query failed")
        .collect::<Result<Vec<u64>, _>>()
        .expect("cannot read id");
    ids.sort_unstable();
    ids
}

fn saved_ids(db_conn: &Connection) -> SavedIds {
    let mut statement = db_conn
        .prepare("SELECT id, component_a_id, component_b_id, key FROM component_relation")
        .expect("invalid query");
    let mut component_relations = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
        .expect("query failed")
        .collect::<Result<Vec<(u64, u64, u64, String)>, _>>()
        .expect("cannot read component relation");
    component_relations.sort();
    SavedIds {
        behaviors: sorted_ids(db_conn, "SELECT id FROM behavior"),
        component_relations,
        components: sorted_ids(db_conn, "SELECT id FROM component"),
        teams: sorted_ids(db_conn, "SELECT id FROM team"),
        viewpackets: sorted_ids(db_conn, "SELECT viewpacket_id FROM view_packet"),
    }
}

fn is_address(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() == 4 && parts.iter().all(|part| part.parse::<u8>().is_ok())
}

fn child_elements(element: &Element) -> impl Iterator<Item = &Element> {
    element.children.iter().filter_map(|node| node.as_element())
}

/// Panics when a child is unknown to the DTD sequence,
/// or when a child appears after a later sibling type.
fn assert_children_in_dtd_order(element: &Element, allowed_order: &[&str]) {
    let mut previous_position = 0;
    for child in child_elements(element) {
        let position = allowed_order
            .iter()
            .position(|name| *name == child.name)
            .unwrap_or_else(|| panic!("<{}> is not allowed in <{}>", child.name, element.name));
        assert!(
            position >= previous_position,
            "<{}> comes after <{}> in <{}> {:?}, against the DTD order",
            child.name,
            allowed_order[previous_position],
            element.name,
            element.attributes.get("Id")
        );
        previous_position = position;
    }
}

#[test]
fn ar_00001_saved_file_has_doctype_and_version() {
    let work_dir = fresh_work_dir("ar_00001_doctype");
    let saved_text = fs::read_to_string(save_fixture(&work_dir)).expect("cannot read saved file");
    fs::remove_dir_all(&work_dir).expect("cannot remove work directory");

    let mut lines = saved_text.lines();
    assert!(lines.next().is_some_and(|line| line.starts_with("<?xml ")), "missing XML declaration");
    assert_eq!(lines.next(), Some(SAD_DOCTYPE));
    let root = Element::parse(saved_text.as_bytes()).expect("saved file is not well-formed XML");
    assert_eq!(root.name, "SoftwareArchitectureDocumentation");
    assert_eq!(root.attributes.get("Version").map(String::as_str), Some("0.1.1"));
    assert!(!root.attributes.contains_key("version"), "the DTD defines no lowercase 'version'");
}

#[test]
fn ar_00001_saved_file_follows_dtd_element_order_and_address_format() {
    let work_dir = fresh_work_dir("ar_00001_structure");
    let saved_text = fs::read_to_string(save_fixture(&work_dir)).expect("cannot read saved file");
    fs::remove_dir_all(&work_dir).expect("cannot remove work directory");
    let root = Element::parse(saved_text.as_bytes()).expect("saved file is not well-formed XML");

    assert_children_in_dtd_order(&root, ROOT_CHILD_ORDER);
    for element in child_elements(&root) {
        let (_, allowed_order) = ELEMENT_CHILD_ORDER
            .iter()
            .find(|(name, _)| *name == element.name)
            .unwrap_or_else(|| panic!("no DTD order listed for <{}>", element.name));
        assert_children_in_dtd_order(element, allowed_order);

        if element.name != "Document" && element.name != "Include" {
            let id = element.attributes.get("Id").map(String::as_str).unwrap_or_default();
            assert!(is_address(id), "<{}> Id '{id}' is not an a.b.c.d address", element.name);
        }
        let address_children = child_elements(element)
            .filter(|child| ADDRESS_ELEMENTS.contains(&child.name.as_str()));
        for child in address_children {
            let text = child.get_text().unwrap_or_default();
            assert!(
                is_address(&text),
                "<{}> '{text}' in <{}> {:?} is not an a.b.c.d address",
                child.name,
                element.name,
                element.attributes.get("Id")
            );
        }
    }
}

/// The saved main file holds only its own items,
/// so a reload gets the items of the included file from the copy next to it.
#[test]
fn ar_00002_saved_file_reloads_with_same_ids() {
    let original_ids = saved_ids(&load(&fixture_path()));
    let work_dir = fresh_work_dir("ar_00002_reload");
    let saved_file = save_fixture(&work_dir);
    let reloaded_ids = saved_ids(&load(&saved_file));
    fs::remove_dir_all(&work_dir).expect("cannot remove work directory");

    assert!(!original_ids.components.is_empty(), "fixture loaded no components");
    assert!(!original_ids.viewpackets.is_empty(), "fixture loaded no viewpackets");
    assert!(!original_ids.component_relations.is_empty(), "fixture loaded no relations");
    assert_eq!(reloaded_ids, original_ids);
}

/// Runs the `sad2md` crate of this repository on the saved file.
/// A separate target directory keeps the nested build off the build lock of the test.
#[test]
fn ar_00002_sad2md_processes_saved_file() {
    let work_dir = fresh_work_dir("ar_00002_sad2md");
    let saved_file = save_fixture(&work_dir);
    let sad2md_manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sad2md/Cargo.toml");

    let output = Command::new(env!("CARGO"))
        .args(["run", "--quiet", "--locked", "--manifest-path"])
        .arg(&sad2md_manifest)
        .arg("--")
        .arg(&saved_file)
        .env("CARGO_TARGET_DIR", Path::new(env!("CARGO_TARGET_TMPDIR")).join("sad2md_target"))
        .current_dir(&work_dir)
        .output()
        .expect("cannot start cargo for sad2md");
    let markdown = fs::read_to_string(work_dir.join("draft_architecture.md"));
    fs::remove_dir_all(&work_dir).expect("cannot remove work directory");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "sad2md failed on the saved file: {}\nstderr:\n{stderr}", output.status);
    assert!(!markdown.expect("sad2md wrote no draft_architecture.md").is_empty());
    // The fixture gives viewpacket 1.3.3.1 a team,
    // so only a dropped TeamId causes this warning.
    assert!(
        !stderr.contains("ViewPacket id= 1.3.3.1 has viewStyle 'Assignment' but no TeamId"),
        "the saved file lost the TeamId of viewpacket 1.3.3.1:\n{stderr}"
    );
}

/// Saving a reload of a saved file must not change it,
/// or each save drifts further from the source.
#[test]
fn ar_00002_second_save_is_byte_identical_to_first() {
    let work_dir = fresh_work_dir("ar_00002_drift");
    let first_save = save_fixture(&work_dir);
    let second_save = work_dir.join("second_save_sad.xml");
    save(&load(&first_save), &second_save);
    let first_bytes = fs::read(&first_save).expect("cannot read the first save");
    let second_bytes = fs::read(&second_save).expect("cannot read the second save");
    fs::remove_dir_all(&work_dir).expect("cannot remove work directory");

    assert!(
        first_bytes == second_bytes,
        "the second save differs from the first:\n--- first\n{}\n--- second\n{}",
        String::from_utf8_lossy(&first_bytes),
        String::from_utf8_lossy(&second_bytes)
    );
}

#[test]
fn ar_00002_saved_diagram_keys_equal_source() {
    let work_dir = fresh_work_dir("ar_00002_diagram_key");
    let saved_root = parse_file(&save_fixture(&work_dir));
    fs::remove_dir_all(&work_dir).expect("cannot remove work directory");
    let source_root = parse_file(&fixture_path());

    let diagram_key = |behavior: &Element| child_text(behavior, "DiagramKey");
    let source_keys = values_by_id(&source_root, "Behavior", diagram_key);
    assert!(!source_keys.is_empty(), "fixture has no behaviors");
    assert_eq!(values_by_id(&saved_root, "Behavior", diagram_key), source_keys);
}

#[test]
fn ar_00002_viewpacket_team_id_survives_round_trip() {
    let original_db = load(&fixture_path());
    let work_dir = fresh_work_dir("ar_00002_viewpacket_team");
    let saved_file = save_fixture(&work_dir);
    let saved_root = parse_file(&saved_file);
    let reloaded_db = load(&saved_file);
    fs::remove_dir_all(&work_dir).expect("cannot remove work directory");

    let team_ids = |viewpacket: &Element| child_text(viewpacket, "TeamId");
    let saved_team_ids = values_by_id(&saved_root, "ViewPacket", team_ids);
    assert_eq!(saved_team_ids["1.3.3.1"].as_deref(), Some("1.0.3.1"));
    let team_ids_query = "SELECT viewpacket_id, team_id FROM view_packet ORDER BY viewpacket_id";
    let read_team_ids = |db_conn: &Connection| -> Vec<(u64, u64)> {
        let mut statement = db_conn.prepare(team_ids_query).expect("invalid query");
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("query failed")
            .collect::<Result<_, _>>()
            .expect("cannot read team_id")
    };
    assert_eq!(read_team_ids(&reloaded_db), read_team_ids(&original_db));
}

/// The dump writes no included file (GEDT-AR-00014, T-07),
/// so the saved main file keeps the `Include`,
/// and leaves out the items of the included file.
#[test]
fn ar_00002_saved_main_file_keeps_include_and_only_own_items() {
    let original_db = load(&fixture_path());
    let work_dir = fresh_work_dir("ar_00002_include");
    let saved_root = parse_file(&save_fixture(&work_dir));
    fs::remove_dir_all(&work_dir).expect("cannot remove work directory");
    let source_root = parse_file(&fixture_path());

    let include_values = |root: &Element| -> Vec<(BTreeMap<String, String>, Option<String>)> {
        root_children_named(root, "Include")
            .into_iter()
            .map(|include| {
                let attributes = include.attributes.clone().into_iter().collect();
                (attributes, include.get_text().map(|text| text.into_owned()))
            })
            .collect()
    };
    let source_includes = include_values(&source_root);
    assert!(!source_includes.is_empty(), "fixture has no Include");
    assert_eq!(include_values(&saved_root), source_includes);

    let main_file_ids = |query: &str| -> Vec<String> {
        let mut statement = original_db.prepare(query).expect("invalid query");
        let mut addresses: Vec<String> = statement
            .query_map([MAIN_FILE_ID], |row| row.get::<_, u64>(0))
            .expect("query failed")
            .map(|id| sad_xml_sql::convert_from_id_to_address(id.expect("cannot read id")))
            .collect();
        addresses.sort();
        addresses
    };
    let saved_ids = |name: &str| -> Vec<String> {
        values_by_id(&saved_root, name, |_| ()).into_keys().collect()
    };
    let main_components = main_file_ids("SELECT id FROM component WHERE file_id = ?1");
    let main_viewpackets = main_file_ids("SELECT viewpacket_id FROM view_packet WHERE file_id = ?1");
    let all_components = sorted_ids(&original_db, "SELECT id FROM component");
    assert!(main_components.len() < all_components.len(), "fixture includes no components");
    assert_eq!(saved_ids("Component"), main_components);
    assert_eq!(saved_ids("ViewPacket"), main_viewpackets);
}

#[test]
fn ar_00002_dump_leaves_fixture_directory_untouched() {
    let snapshot_before = fixture_dir_snapshot();
    let work_dir = fresh_work_dir("ar_00002_fixture_untouched");
    let saved_file = save_fixture(&work_dir);
    save(&load(&saved_file), &work_dir.join("second_save_sad.xml"));
    fs::remove_dir_all(&work_dir).expect("cannot remove work directory");

    assert!(fixture_dir_snapshot() == snapshot_before, "the dump changed the fixture directory");
}
