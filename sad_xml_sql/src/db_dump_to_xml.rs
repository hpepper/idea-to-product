use crate::convert_from_id_to_address;
use rusqlite::Connection;
use simple_xml_builder::XMLElement;
use std::fs::File;
use std::io::Write;

use crate::db_retrieval::{
    get_document,
    get_vector_of_behaviors_sorted_by_key_and_order,
    get_vector_of_component_relations_sorted_by_key_and_order,
    get_vector_of_components_sorted_by_name,
    get_vector_of_includes_sorted_by_file_id,
    get_vector_of_main_file_teams_sorted_by_name,
    get_viewpacket_vector_by_type_and_style_sorted_by_order,
};

/// The DOCTYPE the SAD fixtures declare,
/// so DTD-aware tools validate a saved file the same way.
const SAD_DOCTYPE: &str =
    "<!DOCTYPE SoftwareArchitectureDocumentation SYSTEM \"software_architecture_documentation.dtd\">\n";

/// Writes the main SAD file of the database to `output_file`.
///
/// The saved file keeps the `Include` elements of the main file,
/// and holds only the items that belong to the main file.
/// Items loaded from included files stay in those files,
/// which this function never writes.
/// Edits to items from included files are therefore not saved yet.
/// Include paths come from file content and are not validated yet (GEDT-AR-00014, T-07),
/// so writing to them could let a crafted file overwrite any file the user can write.
///
/// # Panics
///
/// Panics when the database has no `document` row, or when a table cannot be read.
pub fn dump_db_to_xml(db_conn: &Connection, output_file: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(output_file)?;

    println!("DDD Dumping database to XML file: {}", output_file);

    let mut xml_root = XMLElement::new("SoftwareArchitectureDocumentation");
    // The DTD requires the attribute name `Version`, see GEDT-AR-00001.
    xml_root.add_attribute("Version", "0.1.1");

    let main_file_id =
        dump_document_to_xml(&mut xml_root, db_conn).expect("Failed to dump document");
    dump_table_include_to_xml(&mut xml_root, db_conn).expect("Failed to dump includes");
    // TODO dump the top architecture
    dump_table_viewpacket_to_xml(&mut xml_root, db_conn, main_file_id)
        .expect("Failed to dump view packets");
    println!("DDD Dumped view packets");
    dump_table_component_to_xml(&mut xml_root, db_conn, main_file_id)
        .expect("Failed to dump components");
    println!("DDD Dumped components");
    dump_table_componentrelation_to_xml(&mut xml_root, db_conn).expect("Failed to dump component relations");
    dump_table_behavior_to_xml(&mut xml_root, db_conn, main_file_id)
        .expect("Failed to dump behaviors");
    dump_table_team_to_xml(&mut xml_root, db_conn, main_file_id).expect("Failed to dump teams");
    // TODO dump_table_requirement_to_xml(&mut xml_root, db_conn)?;
    write_with_doctype(&xml_root, &mut file)
}

/// Writes the XML declaration, the SAD DOCTYPE, and then the document element.
/// simple-xml-builder always writes the declaration as its first line,
/// and has no DOCTYPE support, so the DOCTYPE goes in after that first line.
fn write_with_doctype(xml_root: &XMLElement, output: &mut impl Write) -> std::io::Result<()> {
    let mut xml_bytes = Vec::new();
    xml_root.write(&mut xml_bytes)?;
    let declaration_end = xml_bytes
        .iter()
        .position(|&byte| byte == b'\n')
        .map_or(0, |newline| newline + 1);
    output.write_all(&xml_bytes[..declaration_end])?;
    output.write_all(SAD_DOCTYPE.as_bytes())?;
    output.write_all(&xml_bytes[declaration_end..])
}

pub fn convert_id_to_address(id: u64) -> String {
    let a = id / (256 * 256 * 256);
    let mut temp_id = id - (a * 256 * 256 * 256);
    let b = temp_id / (256 * 256);
    temp_id = temp_id - (b * 256 * 256);
    let c = temp_id / 256;
    let d = temp_id - (c * 256);
    format!("{}.{}.{}.{}", a, b, c, d)
}

/// Writes the `Document` element and returns the `FileId` of the main file.
fn dump_document_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
) -> Result<u64, rusqlite::Error> {
    let document = get_document(db_conn)?;

    let mut document_element = XMLElement::new("Document");

    let mut file_id = XMLElement::new("FileId");
    file_id.add_text(document.file_id.to_string());
    document_element.add_child(file_id);

    let mut title = XMLElement::new("Title");
    title.add_text(document.title.clone());
    document_element.add_child(title);

    let mut issue = XMLElement::new("Issue");
    issue.add_text(document.issue.clone());
    document_element.add_child(issue);

    let mut summary = XMLElement::new("Summary");
    summary.add_text(document.summary.clone());
    document_element.add_child(summary);

    xml_root.add_child(document_element);
    Ok(document.file_id)
}

/// Writes each `Include` of the main file with the attribute and text it was loaded with.
fn dump_table_include_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
) -> Result<(), rusqlite::Error> {
    for include in get_vector_of_includes_sorted_by_file_id(db_conn)? {
        let mut include_element = XMLElement::new("Include");
        include_element.add_attribute("url", &include.url);
        include_element.add_text(include.filename);
        xml_root.add_child(include_element);
    }
    Ok(())
}

/// Returns `stored_key` without the `{file_id}-` prefix the XML reader adds to a behavior
/// `DiagramKey`, so a save writes the key as the source file had it.
/// Without this, each save and reload adds one more prefix.
/// A key without the prefix,
/// for example one created in the session, is returned unchanged.
fn diagram_key_as_in_file(stored_key: &str, file_id: u64) -> &str {
    stored_key.strip_prefix(&format!("{file_id}-")).unwrap_or(stored_key)
}

/// The XML reader loads behaviors from the main file only,
/// so every behavior belongs to the main file.
fn dump_table_behavior_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
    main_file_id: u64,
) -> Result<(), rusqlite::Error> {
    let behavior_vector = get_vector_of_behaviors_sorted_by_key_and_order(db_conn)?;

    for behavior in behavior_vector {
        let mut behavior_element = XMLElement::new("Behavior");
        // The XML reader expects both IDs as "a.b.c.d" addresses.
        behavior_element.add_attribute("Id", convert_from_id_to_address(behavior.id));
        behavior_element.add_attribute("SortOrder", &behavior.sort_order.to_string());

        let mut view_packet_id = XMLElement::new("ViewPacketId");
        view_packet_id.add_text(convert_from_id_to_address(behavior.view_packet_id));
        behavior_element.add_child(view_packet_id);

        let mut description = XMLElement::new("Description");
        description.add_text(behavior.description.clone());
        behavior_element.add_child(description);

        let mut diagram_key = XMLElement::new("DiagramKey");
        diagram_key.add_text(diagram_key_as_in_file(&behavior.diagram_key, main_file_id));
        behavior_element.add_child(diagram_key);

        xml_root.add_child(behavior_element);
    }
    Ok(())
}

fn dump_table_component_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
    main_file_id: u64,
) -> Result<(), rusqlite::Error> {
    let component_vector = get_vector_of_components_sorted_by_name(db_conn)?;

    for component in component_vector.into_iter().filter(|component| component.file_id == main_file_id) {
        let mut component_element = XMLElement::new("Component");
        component_element.add_attribute("Id", convert_from_id_to_address(component.id));
        component_element.add_attribute("Name", &component.name);

        let mut purpose_element = XMLElement::new("Purpose");
        purpose_element.add_text(component.purpose.clone());
        component_element.add_child(purpose_element);

        let mut summary_element = XMLElement::new("Summary");
        summary_element.add_text(component.summary.clone());
        component_element.add_child(summary_element);

        // The XML reader expects <TeamId> inside <Component>, as an "a.b.c.d" address.
        // The DTD sequence for Component puts TeamId before Title.
        if component.team_id > 0 {
            let mut team_id_element = XMLElement::new("TeamId");
            team_id_element.add_text(convert_from_id_to_address(component.team_id));
            component_element.add_child(team_id_element);
        }
        if !component.title.is_empty() {
            let mut title_element = XMLElement::new("Title");
            title_element.add_text(component.title.clone());
            component_element.add_child(title_element);
        }
        xml_root.add_child(component_element);
    }
    Ok(())
}

fn dump_table_team_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
    main_file_id: u64,
) -> Result<(), rusqlite::Error> {
    for team in get_vector_of_main_file_teams_sorted_by_name(db_conn, main_file_id)? {
        let mut team_element = XMLElement::new("Team");
        team_element.add_attribute("Id", convert_from_id_to_address(team.id));
        team_element.add_attribute("Name", &team.name);

        // The XML reader requires <Description>, even when it is empty.
        let mut description = XMLElement::new("Description");
        description.add_text(team.description.clone());
        team_element.add_child(description);

        xml_root.add_child(team_element);
    }
    Ok(())
}

fn dump_table_componentrelation_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
) -> Result<(), rusqlite::Error> {
    let component_relation_vector =
        get_vector_of_component_relations_sorted_by_key_and_order(db_conn)?;

    for component_relation in component_relation_vector {
        let mut component_relation_element = XMLElement::new("ComponentRelation");
        component_relation_element.add_attribute("Id", convert_from_id_to_address(component_relation.id));
        component_relation_element
            .add_attribute("SortOrder", &component_relation.sort_order.to_string());

        let mut component_a_id = XMLElement::new("ComponentAId");
        component_a_id.add_text(convert_from_id_to_address(component_relation.component_a_id));
        component_relation_element.add_child(component_a_id);

        let mut component_b_id = XMLElement::new("ComponentBId");
        component_b_id.add_text(convert_from_id_to_address(component_relation.component_b_id));
        component_relation_element.add_child(component_b_id);

        let mut key = XMLElement::new("Key");
        key.add_text(component_relation.key.clone());
        component_relation_element.add_child(key);

        let mut property_of_relation = XMLElement::new("PropertyOfRelation");
        property_of_relation.add_text(component_relation.property_of_relation.clone());
        component_relation_element.add_child(property_of_relation);

        let mut connection_type = XMLElement::new("ConnectionType");
        connection_type.add_text(component_relation.connection_type.clone());
        component_relation_element.add_child(connection_type);

        let mut relation_text = XMLElement::new("RelationText");
        relation_text.add_text(component_relation.relation_text.clone());
        component_relation_element.add_child(relation_text);

        let mut relation_description = XMLElement::new("RelationDescription");
        relation_description.add_text(component_relation.relation_description.clone());
        component_relation_element.add_child(relation_description);

        let mut style = XMLElement::new("Style");
        style.add_text(component_relation.style.clone());
        component_relation_element.add_child(style);

        xml_root.add_child(component_relation_element);
    }
    Ok(())
}

fn dump_table_viewpacket_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
    main_file_id: u64,
) -> Result<(), rusqlite::Error> {
    for module_view_style in ["Decomposition", "Uses", "UsedBy", "Generalize", "Layered"] {
        retrieve_from_db_and_put_in_xml(db_conn, xml_root, "Module", module_view_style, main_file_id)?;
    }
    for cnc_view_style in [
        "ClientServer",
        "PeerToPeer",
        "PublishSubscribe",
        "PipeAndFilter",
        "SharedData",
    ] {
        retrieve_from_db_and_put_in_xml(db_conn, xml_root, "CnC", cnc_view_style, main_file_id)?;
    }
    for allocation_view_style in ["Deployment", "Install", "Assignment", "Testing"] {
        retrieve_from_db_and_put_in_xml(
            db_conn,
            xml_root,
            "Allocation",
            allocation_view_style,
            main_file_id,
        )?;
    }
    Ok(())
}

fn retrieve_from_db_and_put_in_xml(
    db_conn: &Connection,
    xml_root: &mut XMLElement,
    filter_view_type: &str,
    filter_view_style: &str,
    main_file_id: u64,
) -> Result<(), rusqlite::Error> {
    let viewpacket_vector = get_viewpacket_vector_by_type_and_style_sorted_by_order(
        db_conn,
        filter_view_type,
        filter_view_style,
    )?;
    for viewpacket in viewpacket_vector.into_iter().filter(|viewpacket| viewpacket.file_id == main_file_id) {
        let mut viewpacket_element = XMLElement::new("ViewPacket");
        viewpacket_element.add_attribute("Id", convert_from_id_to_address(viewpacket.viewpacket_id));
        viewpacket_element.add_attribute("ViewType", &viewpacket.view_type);
        viewpacket_element.add_attribute("ViewStyle", &viewpacket.view_style);
        viewpacket_element.add_attribute("SortOrder", &viewpacket.sort_order.to_string());

        let mut title = XMLElement::new("Title");
        title.add_text(viewpacket.title.clone());
        viewpacket_element.add_child(title);

        let mut introduction = XMLElement::new("Introduction");
        introduction.add_text(viewpacket.introduction.clone());
        viewpacket_element.add_child(introduction);

        let mut component_id = XMLElement::new("ComponentId");
        component_id.add_text(convert_from_id_to_address(viewpacket.component_id));
        viewpacket_element.add_child(component_id);

        let mut primary_display_key = XMLElement::new("PrimaryDisplayKey");
        primary_display_key.add_text(viewpacket.primary_display_key.clone());
        viewpacket_element.add_child(primary_display_key);

        let mut context_model_key = XMLElement::new("ContextModelKey");
        context_model_key.add_text(viewpacket.context_model_key.clone());
        viewpacket_element.add_child(context_model_key);

        // The DTD sequence for ViewPacket ends with TeamId,
        // and the reader reads 0 as no team.
        if viewpacket.team_id > 0 {
            let mut team_id = XMLElement::new("TeamId");
            team_id.add_text(convert_from_id_to_address(viewpacket.team_id));
            viewpacket_element.add_child(team_id);
        }

        xml_root.add_child(viewpacket_element);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_create_in_mem_db;
    use std::fs;
    use xmltree::Element;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db_create_in_mem_db(&conn);
        conn.execute(
            "INSERT INTO document (file_id, filename, title, issue, summary) VALUES (1, 'test_sad.xml', 'Title1', 'Issue1', 'Summary1')",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_dump_db_to_xml_creates_file() {
        let conn = setup_test_db();
        let output_file = "test_output.xml";
        // A stale file from an earlier run would hide a failed dump.
        if let Err(error) = fs::remove_file(output_file) {
            assert_eq!(
                error.kind(),
                std::io::ErrorKind::NotFound,
                "cannot remove stale {output_file}"
            );
        }
        dump_db_to_xml(&conn, output_file).expect("dump_db_to_xml failed");
        assert!(fs::metadata(output_file).is_ok());
        fs::remove_file(output_file).unwrap();
    }

    #[test]
    fn test_dump_db_to_xml_writes_team_and_component_team_id() {
        let conn = setup_test_db();
        let team_id = crate::convert_from_address_to_id("1.0.3.1".to_string(), "test");
        crate::insert_into_team(&conn, team_id, "Squad A", "");
        crate::insert_into_component(&conn, 1, 1, "ComponentA", "Purpose", "Summary", team_id, "");
        let output_file = "test_output_team.xml";
        dump_db_to_xml(&conn, output_file).unwrap();
        let xml_content = fs::read_to_string(output_file).unwrap();
        fs::remove_file(output_file).unwrap();

        let root = Element::parse(xml_content.as_bytes()).unwrap();
        let team = root.get_child("Team").expect("missing <Team>");
        assert_eq!(team.attributes["Id"], "1.0.3.1");
        assert_eq!(team.attributes["Name"], "Squad A");
        assert!(team.get_child("Description").is_some());
        let component = root.get_child("Component").expect("missing <Component>");
        let component_team_id = component.get_child("TeamId").expect("missing <TeamId>");
        assert_eq!(component_team_id.get_text().unwrap(), "1.0.3.1");
        assert!(root.get_child("TeamId").is_none());

        // The dump loads back with the component still pointing at an existing team.
        fs::write(output_file, &xml_content).unwrap();
        let reloaded_conn = Connection::open_in_memory().unwrap();
        db_create_in_mem_db(&reloaded_conn);
        crate::db_populate_from_xml(&reloaded_conn, &output_file.to_string());
        fs::remove_file(output_file).unwrap();
        let component = crate::get_component_by_name(&reloaded_conn, "ComponentA".to_string()).unwrap();
        assert_eq!(component.team_id, team_id);
        let team = crate::get_team_by_name(&reloaded_conn, "Squad A").unwrap();
        assert_eq!(team.id, team_id);
    }

    #[test]
    fn test_dump_db_to_xml_xml_structure() {
        let conn = setup_test_db();
        let output_file = "test_output_structure.xml";
        // A stale file from an earlier run would hide a failed dump.
        if let Err(error) = fs::remove_file(output_file) {
            assert_eq!(
                error.kind(),
                std::io::ErrorKind::NotFound,
                "cannot remove stale {output_file}"
            );
        }
        dump_db_to_xml(&conn, output_file).expect("dump_db_to_xml failed");
        let xml_content = fs::read_to_string(output_file).unwrap();
        let root = Element::parse(xml_content.as_bytes()).unwrap();
        assert_eq!(root.name, "SoftwareArchitectureDocumentation");
        fs::remove_file(output_file).unwrap();
    }

    // TODO find a way to test this, right now it fails because the database is empty(ergo missing compont table)
    // #[test]
    // fn test_dump_db_to_xml_empty_db() {
    //     let conn = Connection::open_in_memory().unwrap();
    //     let output_file = "test_output_empty.xml";
    //     // 'let _ =' to ignore the Result from the call.
    //     let _ = dump_db_to_xml(&conn, output_file);
    //     let xml_content = fs::read_to_string(output_file).unwrap();
    //     let root = Element::parse(xml_content.as_bytes()).unwrap();
    //     assert_eq!(root.name, "SoftwareArchitectureDocumentation");
    //     fs::remove_file(output_file).unwrap();
    // }

    #[test]
    fn test_dump_document_to_xml() {
        let conn = setup_test_db();
        let mut xml_root = XMLElement::new("SoftwareArchitectureDocumentation");
        dump_document_to_xml(&mut xml_root, &conn).unwrap();

        let mut buffer = Vec::new();
        xml_root.write(&mut buffer).unwrap();
        let root = Element::parse(buffer.as_slice()).unwrap();

        let document = root.get_child("Document").expect("Document element missing");
        assert_eq!(
            document.get_child("FileId").and_then(|e| e.get_text()).as_deref(),
            Some("1")
        );
        assert_eq!(
            document.get_child("Title").and_then(|e| e.get_text()).as_deref(),
            Some("Title1")
        );
        assert_eq!(
            document.get_child("Issue").and_then(|e| e.get_text()).as_deref(),
            Some("Issue1")
        );
        assert_eq!(
            document.get_child("Summary").and_then(|e| e.get_text()).as_deref(),
            Some("Summary1")
        );
    }

    #[test]
    fn test_dump_document_to_xml_no_document_row() {
        let conn = Connection::open_in_memory().unwrap();
        db_create_in_mem_db(&conn);
        let mut xml_root = XMLElement::new("SoftwareArchitectureDocumentation");

        let result = dump_document_to_xml(&mut xml_root, &conn);

        assert!(result.is_err());
    }

    #[test]
    fn ar_00002_diagram_key_as_in_file_removes_one_reader_prefix() {
        assert_eq!(diagram_key_as_in_file("1-MSC301", 1), "MSC301");
        assert_eq!(diagram_key_as_in_file("1-1-MSC301", 1), "1-MSC301");
        assert_eq!(diagram_key_as_in_file("MSC301", 1), "MSC301");
        assert_eq!(diagram_key_as_in_file("2-MSC301", 1), "2-MSC301");
    }

    #[test]
    fn ar_00002_dump_writes_only_main_file_teams() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO team (id, name, description, file_id) VALUES (?1, ?2, '', ?3)",
            (crate::convert_from_address_to_id("1.0.3.1".to_string(), "test"), "Main team", 1),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO team (id, name, description, file_id) VALUES (?1, ?2, '', ?3)",
            (crate::convert_from_address_to_id("2.0.3.1".to_string(), "test"), "Included team", 2),
        )
        .unwrap();
        crate::insert_into_team(&conn, 3, "Session team", "");
        let output_file = "test_output_main_file_teams.xml";
        dump_db_to_xml(&conn, output_file).unwrap();
        let xml_content = fs::read_to_string(output_file).unwrap();
        fs::remove_file(output_file).unwrap();

        assert!(xml_content.contains("Name=\"Main team\""));
        assert!(xml_content.contains("Name=\"Session team\""));
        assert!(!xml_content.contains("Included team"));
    }

    #[test]
    fn test_convert_id_to_address() {
        let id = 16909060; // corresponds to 1.2.3.4
        let address = convert_id_to_address(id);
        assert_eq!(address, "1.2.3.4");
    }
}