use rusqlite::Connection;
use std::fs;
use sad_xml_sql::db_create_in_mem_db;
use sad_xml_sql::db_populate_from_xml;
use sad_xml_sql::db_dump_to_xml::dump_db_to_xml;
use xmltree::{Element};

fn print_db_info(db_conn: &Connection) {
    // TODO get tables
    let table_names: Vec<String> = db_conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|result| result.unwrap())
        .collect();

    println!("Tables in the database:");
    for table_name in table_names {
        println!(" - {}", table_name);
    }
}

#[test]
fn test_xml_load_and_dump() {
    let filename = "tests/test_sad.xml".to_string();

    let db_conn = Connection::open_in_memory().expect("Connecting to the SQLite database failed.");

    db_create_in_mem_db(&db_conn);
    db_populate_from_xml(&db_conn, &filename);
    print_db_info(&db_conn);
    let output_file = "dump_test_output.xml";
    let _ = dump_db_to_xml(&db_conn, output_file);
    // TODO: Add assertions to verify the contents of the output XML file
    let xml_content = fs::read_to_string(output_file).unwrap();
    let root = Element::parse(xml_content.as_bytes()).unwrap();
    assert_eq!(root.name, "SoftwareArchitectureDocumentation");

    // A component with a Title keeps it, a component without one gets no Title element.
    let find_component = |name: &str| {
        root.children
            .iter()
            .filter_map(|node| node.as_element())
            .find(|elem| elem.name == "Component" && elem.attributes.get("Name").map(String::as_str) == Some(name))
            .unwrap_or_else(|| panic!("Component {} missing in dump", name))
    };
    let with_title = find_component("BattleResolver");
    assert_eq!(
        with_title.get_child("Title").and_then(|t| t.get_text()).as_deref(),
        Some("Battle Resolver")
    );
    let without_title = find_component("CardDeckStorage");
    assert!(without_title.get_child("Title").is_none());
}