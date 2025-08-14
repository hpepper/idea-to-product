use rusqlite::Connection;
use simple_xml_builder::XMLElement;
use std::fs::File;

use crate::db_retrieval::get_viewpacket_vector_by_type_and_style_sorted_by_order;

pub fn dump_db_to_xml(db_conn: &Connection, output_file: &str) -> Result<(), std::io::Error> {
    let file = File::create(output_file)?;

    let mut xml_root = XMLElement::new("SoftwareArchitectureDocumentation");
    xml_root.add_attribute("version", "0.1.1");

    // Dump each table
    // TODO dump the top architecture
    dump_table_viewpacket_to_xml(&mut xml_root, db_conn);
    // TODO dump_table_component_to_xml(&mut xml_root, db_conn)?;
    // TODO dump_table_componentrelation_to_xml(&mut xml_root, db_conn)?;
    // TODO dump_table_behavior_to_xml(&mut xml_root, db_conn)?;
    // TODO dump_table_requirement_to_xml(&mut xml_root, db_conn)?;
    xml_root.write(file)?;
    Ok(())
}

fn dump_table_viewpacket_to_xml(
    xml_root: &mut XMLElement,
    db_conn: &Connection,
) -> Result<(), rusqlite::Error> {
    for module_view_style in ["Decomposition", "Uses", "UsedBy", "Generalize", "Layered"] {
        retrieve_from_db_and_put_in_xml(
            db_conn,
            xml_root,
            "Module",
            module_view_style,
        )?;
    }
    for cnc_view_style in ["ClientServer", "PeerToPeer", "PublishSubscribe", "PipeAndFilter", "SharedData"] {
        retrieve_from_db_and_put_in_xml(
            db_conn,
            xml_root,
            "CnC",
            cnc_view_style,
        )?;
    }
    for allocation_view_style in ["Deployment", "Install", "Assignment", "Testing"] {
        retrieve_from_db_and_put_in_xml(
            db_conn,
            xml_root,
            "Allocation",
            allocation_view_style,
        )?;
    }
    Ok(())
}

fn retrieve_from_db_and_put_in_xml(db_conn: &Connection, xml_root: &mut XMLElement, filter_view_type: &str, filter_view_style: &str) -> Result<(), rusqlite::Error> {
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

    #[test]
    fn test_dump_db_to_xml_empty_db() {
        let conn = Connection::open_in_memory().unwrap();
        let output_file = "test_output_empty.xml";
        // 'let _ =' to ignore the Result from the call.
        let _ = dump_db_to_xml(&conn, output_file);
        let xml_content = fs::read_to_string(output_file).unwrap();
        let root = Element::parse(xml_content.as_bytes()).unwrap();
        assert_eq!(root.name, "SoftwareArchitectureDocumentation");
        fs::remove_file(output_file).unwrap();
    }
}
