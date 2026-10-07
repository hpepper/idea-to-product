use crate::convert_from_id_to_address;
use rusqlite::Connection;
use simple_xml_builder::XMLElement;
use std::fs::File;

use crate::db_retrieval::{
    get_document,
    get_vector_of_behaviors_sorted_by_key_and_order,
    get_vector_of_component_relations_sorted_by_key_and_order,
    get_vector_of_components_sorted_by_name,
    get_vector_of_teams_sorted_by_name,
    get_viewpacket_vector_by_type_and_style_sorted_by_order,
};

pub fn dump_db_to_xml(db_conn: &Connection, output_file: &str) -> Result<(), std::io::Error> {
    let file = File::create(output_file)?;

    println!("DDD Dumping database to XML file: {}", output_file);

    let mut xml_root = XMLElement::new("SoftwareArchitectureDocumentation");
    xml_root.add_attribute("version", "0.1.1");

    // TODO dump the Document entry
    dump_document_to_xml(&mut xml_root, db_conn).expect("Failed to dump document");
    // Dump each table
    // TODO dump the top architecture
    dump_table_viewpacket_to_xml(&mut xml_root, db_conn).expect("Failed to dump view packets");
    println!("DDD Dumped view packets");
    dump_table_component_to_xml(&mut xml_root, db_conn).expect("Failed to dump components");
    println!("DDD Dumped components");
    dump_table_componentrelation_to_xml(&mut xml_root, db_conn).expect("Failed to dump component relations");
    dump_table_behavior_to_xml(&mut xml_root, db_conn).expect("Failed to dump behaviors");
    dump_table_team_to_xml(&mut xml_root, db_conn).expect("Failed to dump teams");
    // TODO dump_table_requirement_to_xml(&mut xml_root, db_conn)?;
    xml_root.write(file)?;
    Ok(())
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

fn dump_document_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
) -> Result<(), rusqlite::Error> {
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
    Ok(())
}

fn dump_table_behavior_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
) -> Result<(), rusqlite::Error> {
    let behavior_vector = get_vector_of_behaviors_sorted_by_key_and_order(db_conn)?;

    for behavior in behavior_vector {
        let mut behavior_element = XMLElement::new("Behavior");
        behavior_element.add_attribute("Id", behavior.id.to_string());
        behavior_element.add_attribute("SortOrder", &behavior.sort_order.to_string());

        let mut view_packet_id = XMLElement::new("ViewPacketId");
        view_packet_id.add_text(behavior.view_packet_id.to_string());
        behavior_element.add_child(view_packet_id);

        let mut description = XMLElement::new("Description");
        description.add_text(behavior.description.clone());
        behavior_element.add_child(description);

        let mut diagram_key = XMLElement::new("DiagramKey");
        diagram_key.add_text(behavior.diagram_key.clone());
        behavior_element.add_child(diagram_key);

        xml_root.add_child(behavior_element);
    }
    Ok(())
}

fn dump_table_component_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
) -> Result<(), rusqlite::Error> {
    let component_vector = get_vector_of_components_sorted_by_name(db_conn)?;

    for component in component_vector {
        let mut component_element = XMLElement::new("Component");
        component_element.add_attribute("Id", convert_from_id_to_address(component.id));
        component_element.add_attribute("Name", &component.name);

        let mut purpose_element = XMLElement::new("Purpose");
        purpose_element.add_text(component.purpose.clone());
        component_element.add_child(purpose_element);

        let mut summary_element = XMLElement::new("Summary");
        summary_element.add_text(component.summary.clone());
        component_element.add_child(summary_element);

        if !component.title.is_empty() {
            let mut title_element = XMLElement::new("Title");
            title_element.add_text(component.title.clone());
            component_element.add_child(title_element);
        }
        // The XML reader expects <TeamId> inside <Component>, as an "a.b.c.d" address.
        if component.team_id > 0 {
            let mut team_id_element = XMLElement::new("TeamId");
            team_id_element.add_text(convert_from_id_to_address(component.team_id));
            component_element.add_child(team_id_element);
        }
        xml_root.add_child(component_element);
    }
    Ok(())
}

fn dump_table_team_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
) -> Result<(), rusqlite::Error> {
    for team in get_vector_of_teams_sorted_by_name(db_conn)? {
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
) -> Result<(), rusqlite::Error> {
    for module_view_style in ["Decomposition", "Uses", "UsedBy", "Generalize", "Layered"] {
        retrieve_from_db_and_put_in_xml(db_conn, xml_root, "Module", module_view_style)?;
    }
    for cnc_view_style in [
        "ClientServer",
        "PeerToPeer",
        "PublishSubscribe",
        "PipeAndFilter",
        "SharedData",
    ] {
        retrieve_from_db_and_put_in_xml(db_conn, xml_root, "CnC", cnc_view_style)?;
    }
    for allocation_view_style in ["Deployment", "Install", "Assignment", "Testing"] {
        retrieve_from_db_and_put_in_xml(db_conn, xml_root, "Allocation", allocation_view_style)?;
    }
    Ok(())
}

fn retrieve_from_db_and_put_in_xml(
    db_conn: &Connection,
    xml_root: &mut XMLElement,
    filter_view_type: &str,
    filter_view_style: &str,
) -> Result<(), rusqlite::Error> {
    let viewpacket_vector = get_viewpacket_vector_by_type_and_style_sorted_by_order(
        db_conn,
        filter_view_type,
        filter_view_style,
    )?;
    for viewpacket in viewpacket_vector {
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
        let _ = dump_db_to_xml(&conn, output_file);
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
        let _ = dump_db_to_xml(&conn, output_file);
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
    fn test_convert_id_to_address() {
        let id = 16909060; // corresponds to 1.2.3.4
        let address = convert_id_to_address(id);
        assert_eq!(address, "1.2.3.4");
    }
}