use rusqlite::{ Connection, Result };
use std::fs::File;
use std::io::BufReader;
use xmltree::{ Element, XMLNode };

pub fn populate_db(db_conn: &Connection, filename: &String) {
    // Parse the XML file
    let xml_root = load_xml_file(filename);

    populate_db_with_viewpackets(db_conn, &xml_root);
    populate_db_with_components(db_conn, &xml_root);
    populate_db_with_componentrelations(db_conn, &xml_root);
    populate_db_with_behaviors(db_conn, &xml_root);
}


fn load_xml_file(filename: &str) -> Element {
    // verify the file exists
    if !std::path::Path::new(filename).exists() {
        eprintln!("!!! File not found: {}", filename);
        std::process::exit(1);
    }

    let file = File::open(filename).expect("Unable to open file");
    let file = BufReader::new(file);

    // Parse the XML file
    Element::parse(file).expect("Unable to parse XML")
}


fn populate_db_with_behaviors(db_conn: &Connection, xml_root: &Element) {
    // Create the table
    db_conn
        .execute(
            "CREATE TABLE behavior (
            id INTEGER PRIMARY KEY,
            sort_order INTEGER,
            view_packet_id INTEGER,
            description TEXT,
            diagram_key TEXT NOT NULL
        )",
            []
        )
        .expect("Unable to create table");

    // Insert the data
    for child in &xml_root.children {
        match child {
            XMLNode::Element(behavior) => {
                if behavior.name == "Behavior" {
                    // TODO can I get the line in the XML that is currently read?
                    let id: i32 = behavior.attributes
                        .get("Id")
                        .expect("Missing 'Id' attribute for Behavior")
                        .parse()
                        .unwrap();
                    let sort_order: i32 = behavior.attributes
                        .get("SortOrder")
                        .unwrap()
                        .parse()
                        .unwrap();
                    let view_packet_id: i32 = behavior
                        .get_child("ViewPacketId")
                        .expect("<ViewPacketId> element missing")
                        .get_text()
                        .expect("<ViewPacketId> element has no data")
                        .parse()
                        .unwrap();
                    let description = behavior
                        .get_child("Description")
                        .unwrap_or_else(||
                            panic!("<Description> element missing in Behavior id= {}", id)
                        )
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let diagram_key = behavior
                        .get_child("DiagramKey")
                        .unwrap_or_else(||
                            panic!("<DiagramKey> element missing in Behavior id= {}", id)
                        )
                        .get_text()
                        .unwrap_or_else(||
                            panic!("<DiagramKey> element empty in Behavior id= {}", id)
                        );

                    db_conn
                        .execute(
                            "INSERT INTO behavior (id, sort_order, view_packet_id, description, diagram_key)
                            VALUES (?1, ?2, ?3, ?4, ?5)",
                            (
                                id,
                                sort_order,
                                view_packet_id,
                                &description.to_string(),
                                &diagram_key.to_string(),
                            )
                        )
                        .expect("Unable to insert data");
                }
            }
            _ => {}
        }
    }
}

fn populate_db_with_components(db_conn: &Connection, xml_root: &Element) {
    // Create the tables
    db_conn
        .execute(
            "CREATE TABLE component (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            summary TEXT
        )",
            []
        )
        .expect("Unable to create table");

    // Insert the data
    for child in &xml_root.children {
        match child {
            XMLNode::Element(component) => {
                if component.name == "Component" {
                    // TODO can I get the line in the XML that is currently read?
                    let id: i32 = component.attributes
                        .get("Id")
                        .expect("Missing 'Id' attribute for Component")
                        .parse()
                        .unwrap();
                    let name = component.attributes.get("Name").unwrap();
                    let summary = component
                        .get_child("Summary")
                        .unwrap_or_else(||
                            panic!("<Summary> element missing in Component id= {}", id)
                        )
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());

                    db_conn
                        .execute(
                            "INSERT INTO component (id, name, summary)
                            VALUES (?1, ?2, ?3)",
                            (id, &name.to_string(), &summary.to_string())
                        )
                        .expect("Unable to insert data");
                }
            }
            _ => {}
        }
    }
}

fn populate_db_with_componentrelations(db_conn: &Connection, xml_root: &Element) {
    // Create the tables
    db_conn
        .execute(
            "CREATE TABLE component_relation (
            id INTEGER NOT NULL,
            sort_order INTEGER NOT NULL,
            component_a_id INTEGER NOT NULL,
            component_b_id INTEGER NOT NULL,
            connection_type TEXT,
            key TEXT,
            property_of_relation TEXT,
            relation_text TEXT,
            relation_description TEXT
        )",
            []
        )
        .expect("Unable to create table");

    // Insert the data
    for child in &xml_root.children {
        match child {
            XMLNode::Element(component_relation) => {
                if component_relation.name == "ComponentRelation" {
                    // TODO can I get the line in the XML that is currently read?

                    let id: i32 = component_relation.attributes
                        .get("Id")
                        .expect("Missing 'Id' attribute for ComponentRelation")
                        .parse()
                        .unwrap();
                    let sort_order: i32 = component_relation.attributes
                        .get("SortOrder")
                        .expect("Missing 'SortOrder' attribute for ComponentRelation")
                        .parse()
                        .unwrap_or(0);
                    let component_a_id: i32 = component_relation
                        .get_child("ComponentAId")
                        .expect("<ComponentAId> element missing")
                        .get_text()
                        .expect("<ComponentAId> element has no data")
                        .parse()
                        .unwrap();
                    let component_b_id: i32 = component_relation
                        .get_child("ComponentBId")
                        .expect("<ComponentBId> element missing")
                        .get_text()
                        .expect("<ComponentBId> element has no data")
                        .parse()
                        .unwrap();
                    let connection_type = component_relation
                        .get_child("ConnectionType")
                        .and_then(|child| child.get_text())
                        .unwrap_or_else(|| "".to_string().into());
                    let key = component_relation
                        .get_child("Key")
                        .expect("<PropertyOfRelation> element missing")
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let property_of_relation = component_relation
                        .get_child("PropertyOfRelation")
                        .expect("<PropertyOfRelation> element missing")
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let relation_text = component_relation
                        .get_child("RelationText")
                        .and_then(|child| child.get_text())
                        .unwrap_or_else(|| "".to_string().into());
                    let relation_description = component_relation
                        .get_child("RelationDescription")
                        .and_then(|child| child.get_text())
                        .unwrap_or_else(|| "".to_string().into());
                    db_conn
                        .execute(
                            "INSERT INTO component_relation (id, sort_order, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            (
                                id,
                                sort_order,
                                component_a_id,
                                component_b_id,
                                &connection_type.to_string(),
                                &key.to_string(),
                                &property_of_relation.to_string(),
                                &relation_text.to_string(),
                                &relation_description.to_string(),
                            )
                        )
                        .expect("Unable to insert data in component_relation");
                }
            }
            _ => {}
        }
    }
}

fn populate_db_with_viewpackets(db_conn: &Connection, xml_root: &Element) {
    // Create the tables
    db_conn
        .execute(
            "CREATE TABLE view_packet (
            component_id INTEGER NOT NULL,
            context_model_key TEXT,
            primary_display_key TEXT,
            introduction TEXT,
            sort_order INTEGER NOT NULL,
            title TEXT,
            view_style TEXT NOT NULL,
            view_type TEXT NOT NULL,
            viewpacket_id INTEGER PRIMARY KEY
        )",
            []
        )
        .expect("Unable to create table");

    // Insert the data
    for child in &xml_root.children {
        match child {
            XMLNode::Element(viewpacket) => {
                if viewpacket.name == "ViewPacket" {
                    // TODO can I get the line in the XML that is currently read?
                    let viewpacket_id: i32 = viewpacket.attributes
                        .get("Id")
                        .expect("Missing 'Id' attribute for ViewPacket")
                        .parse()
                        .unwrap();
                    let view_type = viewpacket.attributes.get("ViewType").unwrap();
                    let view_style = viewpacket.attributes.get("ViewStyle").unwrap();
                    let sort_order: i32 = viewpacket.attributes
                        .get("SortOrder")
                        .unwrap()
                        .parse()
                        .unwrap();
                    let title = viewpacket
                        .get_child("Title")
                        .unwrap_or_else(||
                            panic!("<Title> element missing in ViewPacket id= {}", viewpacket_id)
                        )
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let introduction = viewpacket
                        .get_child("Introduction")
                        .unwrap_or_else(||
                            panic!("<Introduction> element missing in ViewPacket id= {}", viewpacket_id)
                        )
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let context_model_key = viewpacket
                        .get_child("ContextModelKey")
                        .unwrap_or_else(||
                            panic!("<ContextModelKey> element missing in ViewPacket id= {}", viewpacket_id)
                        )
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let primary_display_key = viewpacket
                        .get_child("PrimaryDisplayKey")
                        .unwrap_or_else(||
                            panic!("<PrimaryDisplayKey> element missing in ViewPacket id= {}", viewpacket_id)
                        )
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let component_id: i32 = viewpacket
                        .get_child("ComponentId")
                        .unwrap()
                        .get_text()
                        .unwrap()
                        .parse()
                        .unwrap();

                    db_conn
                        .execute(
                            "INSERT INTO view_packet (component_id, context_model_key, primary_display_key, introduction, sort_order, title, view_style, view_type, viewpacket_id)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            (
                                component_id,
                                &context_model_key.to_string(),
                                &primary_display_key.to_string(),
                                &introduction.to_string(),
                                sort_order,
                                &title.to_string(),
                                view_style,
                                view_type,
                                viewpacket_id,
                            )
                        )
                        .expect("Unable to insert data");
                }
            }
            _ => {}
        }
    }
}
