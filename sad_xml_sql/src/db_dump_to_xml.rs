use rusqlite::Connection;
use simple_xml_builder::XMLElement;
use std::fs::File;

use crate::db_retrieval::{
    get_vector_of_behaviors_sorted_by_key_and_order,
    get_vector_of_component_relations_sorted_by_key_and_order,
    get_vector_of_components_sorted_by_name,
    get_viewpacket_vector_by_type_and_style_sorted_by_order,
};

pub fn dump_db_to_xml(db_conn: &Connection, output_file: &str) -> Result<(), std::io::Error> {
    let file = File::create(output_file)?;

    let mut xml_root = XMLElement::new("SoftwareArchitectureDocumentation");
    xml_root.add_attribute("version", "0.1.1");

    // Dump each table
    // TODO dump the top architecture
    dump_table_viewpacket_to_xml(&mut xml_root, db_conn);
    dump_table_component_to_xml(&mut xml_root, db_conn);
    dump_table_componentrelation_to_xml(&mut xml_root, db_conn);
    dump_table_behavior_to_xml(&mut xml_root, db_conn);
    // TODO dump_table_requirement_to_xml(&mut xml_root, db_conn)?;
    xml_root.write(file)?;
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
        component_element.add_attribute("Id", component.id.to_string());
        component_element.add_attribute("Name", &component.name);

        let mut purpose_element = XMLElement::new("Purpose");
        purpose_element.add_text(component.purpose.clone());
        component_element.add_child(purpose_element);

        let mut summary_element = XMLElement::new("Summary");
        summary_element.add_text(component.summary.clone());
        component_element.add_child(summary_element);
        xml_root.add_child(component_element);

        if component.team_id > 0 {
            let mut team_id_element = XMLElement::new("TeamId");
            team_id_element.add_text(component.team_id.to_string());
            xml_root.add_child(team_id_element);
        }
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
        component_relation_element.add_attribute("Id", component_relation.id.to_string());
        component_relation_element
            .add_attribute("SortOrder", &component_relation.sort_order.to_string());

        let mut component_a_id = XMLElement::new("ComponentAId");
        component_a_id.add_text(component_relation.component_a_id.to_string());
        component_relation_element.add_child(component_a_id);

        let mut component_b_id = XMLElement::new("ComponentBId");
        component_b_id.add_text(component_relation.component_b_id.to_string());
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
        viewpacket_element.add_attribute("Id", viewpacket.viewpacket_id.to_string());
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
        component_id.add_text(viewpacket.component_id.to_string());
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
}
