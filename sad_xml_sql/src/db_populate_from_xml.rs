use rusqlite::Connection;
use std::fs::File;
use std::io::BufReader;
use xmltree::{Element, XMLNode};
use std::path::Path;

use crate::db_dump_to_xml::convert_id_to_address;

// Requires create_database() to have been called.
pub fn db_populate_from_xml(db_conn: &Connection, filename: &String) {
    let path = Path::new(filename);
    let mut parent = path.parent().expect("Failed to get parent directory").to_str().unwrap_or(".");
    if parent == "" {
        parent = ".";
    }
    // Parse the XML file
    let xml_root = load_xml_file(filename);

    // TODO pass file_id into each subsequent call, except for the teams.
    let file_id = populate_db_with_document(db_conn, &xml_root, filename);
    populate_db_with_viewpackets(db_conn, &xml_root, file_id);
    populate_db_with_behaviors(db_conn, &xml_root, file_id);
    populate_db_with_components(db_conn, &xml_root, file_id);
    populate_db_with_componentrelations(db_conn, &xml_root);
    populate_db_with_includes(db_conn, &xml_root, parent);
    populate_db_with_teams(db_conn, &xml_root);
}

fn update_file_id_for_filename(db_conn: &Connection, file_id: u64, filename: String) {
    db_conn
        .execute(
            "UPDATE include SET file_id = ?1 WHERE filename = ?2",
            (file_id, filename),
        )
        .expect("Unable to update file_id for include");
}

pub fn db_populate_from_include_xml(db_conn: &Connection, filename: &String, original_filename: String) {
    println!("DDD loading including file: {}", filename);
    // Parse the XML file
    let xml_root = load_xml_file(filename);

    let file_id = get_file_id_from_include_xml(&xml_root);
    update_file_id_for_filename(db_conn, file_id, original_filename);
    populate_db_with_viewpackets(db_conn, &xml_root, file_id);

    populate_db_with_components(db_conn, &xml_root, file_id);
    populate_db_with_teams(db_conn, &xml_root);
}
// TODO itterate though include elements of the document of the first file.
// TODO do not load viewpackets from include files.

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

/// Convert an address of the form "file_id.a.b.c" into a numerical ID.
fn convert_from_address_to_id(addr: String, location: &str) -> u64 {
    // TODO: Implement the conversion logic
    // TODO split "a.b.c" into parts and calculate the id.
    let parts: Vec<&str> = addr.split('.').collect();
    if parts.len() == 4 {
        let alpha: u64 = parts[0].parse().unwrap_or(0);
        let bravo: u64 = parts[1].parse().unwrap_or(0);
        let charlie: u64 = parts[2].parse().unwrap_or(0);
        let delta: u64 = parts[3].parse().unwrap_or(0);
        return alpha * 256 * 256 * 256 + bravo * 256 * 256 + charlie * 256 + delta;
    } else {
        panic!(
            "!!! Warning: Address '{}' is not in the correct format 'a.b.c.d'. Location: {}",
            addr, location
        );
    }
}

fn get_file_id_from_include_xml(xml_root: &Element) -> u64 {
    // Insert the data
    let mut file_id = 0;
    for child in &xml_root.children {
        match child {
            XMLNode::Element(document) => {
                if document.name == "Document" {
                    file_id = document
                        .get_child("FileId")
                        .unwrap_or_else(|| panic!("<FileId> element missing in document"))
                        .get_text()
                        .unwrap_or_else(|| panic!("<FileId> element has no data in document"))
                        .parse()
                        .unwrap_or_else(|_| {
                            panic!("<FileId> element contains invalid number in document")
                        });
                }
            }
            _ => {}
        }
    }
    file_id
}


fn populate_db_with_behaviors(db_conn: &Connection, xml_root: &Element, file_id: u64) {
    // Insert the data
    for child in &xml_root.children {
        match child {
            XMLNode::Element(behavior) => {
                if behavior.name == "Behavior" {
                    // TODO can I get the line in the XML that is currently read?
                    let id_addr: String = behavior
                        .attributes
                        .get("Id")
                        .expect("Missing 'Id' attribute for Behavior")
                        .parse()
                        .unwrap();
                    let sort_order: u64 = behavior
                        .attributes
                        .get("SortOrder")
                        .unwrap()
                        .parse()
                        .unwrap();
                    let view_packet_addr: String = behavior
                        .get_child("ViewPacketId")
                        .expect("<ViewPacketId> element missing")
                        .get_text()
                        .expect("<ViewPacketId> element has no data")
                        .parse()
                        .unwrap();
                    let description = behavior
                        .get_child("Description")
                        .unwrap_or_else(|| {
                            panic!("<Description> element missing in Behavior id= {}", id_addr)
                        })
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let local_diagram_key = behavior
                        .get_child("DiagramKey")
                        .unwrap_or_else(|| {
                            panic!("<DiagramKey> element missing in Behavior id= {}", id_addr)
                        })
                        .get_text()
                        .unwrap_or_else(|| {
                            panic!("<DiagramKey> element empty in Behavior id= {}", id_addr)
                        });
                    let id: u64 =
                        convert_from_address_to_id( id_addr.to_string(), "Behavior - id");
                    let view_packet_id: u64 = convert_from_address_to_id(
                        
                        view_packet_addr.to_string(),
                        "Behavior - ViewPacketId",
                    );
                    let diagram_key = format!("{}-{}", file_id, local_diagram_key);
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

fn populate_db_with_components(db_conn: &Connection, xml_root: &Element, file_id: u64) {
    // Insert the data
    for child in &xml_root.children {
        match child {
            XMLNode::Element(component) => {
                if component.name == "Component" {
                    // TODO can I get the line in the XML that is currently read?
                    let id_addr: String = component
                        .attributes
                        .get("Id")
                        .expect("Missing 'Id' attribute for Component")
                        .to_string();

                    let name = component.attributes.get("Name").unwrap();
                    let purpose = component
                        .get_child("Purpose")
                        .unwrap_or_else(|| {
                            panic!("<Purpose> element missing in Component id= {}", id_addr)
                        })
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let summary = component
                        .get_child("Summary")
                        .unwrap_or_else(|| {
                            panic!("<Summary> element missing in Component id= {}", id_addr)
                        })
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let team_id: String = match component.get_child("TeamId") {
                        Some(team_id_elem) => {
                            let team_id_text = team_id_elem.get_text().unwrap_or_else(|| {
                                panic!("<TeamId> element has no data in Component id= {}", id_addr)
                            });
                            team_id_text.parse().unwrap()
                        }
                        None => "0.0.0.0".into(),
                    };
                    let id: u64 =
                        convert_from_address_to_id( id_addr.to_string(), "Component - id");
                    let team_id: u64 = convert_from_address_to_id(
                        team_id.to_string(),
                        "Component - TeamId",
                    );
                    db_conn
                        .execute(
                            "INSERT INTO component (file_id, id, name, purpose, summary, team_id)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            (
                                file_id,
                                id,
                                &name.to_string(),
                                &purpose.to_string(),
                                &summary.to_string(),
                                team_id,
                            ),
                        )
                        .expect(&format!("Unable to insert data id: {}", convert_id_to_address(id)));
                }
            }
            _ => {}
        }
    }
}

fn populate_db_with_componentrelations(db_conn: &Connection, xml_root: &Element) {
    // Insert the data
    for child in &xml_root.children {
        match child {
            XMLNode::Element(component_relation) => {
                if component_relation.name == "ComponentRelation" {
                    // TODO can I get the line in the XML that is currently read?

                    let id_addr: String = component_relation
                        .attributes
                        .get("Id")
                        .expect("Missing 'Id' attribute for ComponentRelation")
                        .parse()
                        .unwrap();
                    let sort_order: u64 = component_relation
                        .attributes
                        .get("SortOrder")
                        .expect("Missing 'SortOrder' attribute for ComponentRelation")
                        .parse()
                        .unwrap_or(0);
                    let component_a_addr: String = component_relation
                        .get_child("ComponentAId")
                        .expect("<ComponentAId> element missing")
                        .get_text()
                        .expect("<ComponentAId> element has no data").to_string();
                    let component_b_addr: String = component_relation
                        .get_child("ComponentBId")
                        .expect("<ComponentBId> element missing")
                        .get_text()
                        .expect("<ComponentBId> element has no data").to_string();
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
                    let style = component_relation
                        .get_child("Style")
                        .and_then(|child| child.get_text())
                        .unwrap_or_else(|| "".to_string().into());

                        let id: u64 = convert_from_address_to_id(id_addr, "ComponentRelation - id");
                        let component_a_id: u64 = convert_from_address_to_id(component_a_addr, "ComponentRelation - ComponentAId");
                        let component_b_id: u64 = convert_from_address_to_id(component_b_addr, "ComponentRelation - ComponentBId");
                    db_conn
                        .execute(
                            "INSERT INTO component_relation (id, sort_order, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description, style)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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
                                &style.to_string(),
                            )
                        )
                        .expect("Unable to insert data in component_relation");
                }
            }
            _ => {}
        }
    }
}

fn populate_db_with_document(db_conn: &Connection, xml_root: &Element, filename: &String) -> u64 {
    // Insert the data
    let mut file_id = 0;
    for child in &xml_root.children {
        match child {
            XMLNode::Element(document) => {
                if document.name == "Document" {
                    file_id = document
                        .get_child("FileId")
                        .unwrap_or_else(|| panic!("<FileId> element missing in document"))
                        .get_text()
                        .unwrap_or_else(|| panic!("<FileId> element has no data in document"))
                        .parse()
                        .unwrap_or_else(|_| {
                            panic!("<FileId> element contains invalid number in document")
                        });

                    let title = document
                        .get_child("Title")
                        .and_then(|child| child.get_text())
                        .unwrap_or_else(|| "".to_string().into());

                    let issue = document
                        .get_child("Issue")
                        .and_then(|child| child.get_text())
                        .unwrap_or_else(|| "".to_string().into());

                    let summary = document
                        .get_child("Summary")
                        .and_then(|child| child.get_text())
                        .unwrap_or_else(|| "".to_string().into());

                    // TODO also read the members and their roles and put in the member table.
                    db_conn
                        .execute(
                            "INSERT INTO document (file_id, filename, title, issue, summary)
                            VALUES (?1, ?2, ?3, ?4, ?5)",
                            (file_id, filename, title, issue, summary),
                        )
                        .expect("Unable to insert data into document");
                }
            }
            _ => {}
        }
    }
    file_id
}

fn populate_db_with_includes(db_conn: &Connection, xml_root: &Element, parent: &str) {
    // Insert the data
    for child in &xml_root.children {
        match child {
            XMLNode::Element(include) => {
                if include.name == "Include" {
                    let url: String = include
                        .attributes
                        .get("url")
                        .expect("Missing 'url' attribute for Include")
                        .parse()
                        .unwrap();
                    // TODO secure this against path traversal attacks.
                    let filename: String = include
                        .get_text()
                        .expect("<Include> element has no data")
                        .parse()
                        .unwrap();
                    db_conn
                        .execute(
                            "INSERT INTO include (url, filename)
                            VALUES (?1, ?2)",
                            (url, filename.clone()),
                        )
                        .expect("Unable to insert data");
                    let full_path = format!("{}/{}", parent, filename);
                    db_populate_from_include_xml(db_conn, &full_path, filename);
                }
            }
            _ => {}
        }
    }
}

fn populate_db_with_teams(db_conn: &Connection, xml_root: &Element) {
    // Insert the data
    for child in &xml_root.children {
        match child {
            XMLNode::Element(team) => {
                if team.name == "Team" {
                    let id_addr: String = team
                        .attributes
                        .get("Id")
                        .expect("Missing 'Id' attribute for team")
                        .to_string();
                    let name = team.attributes.get("Name").unwrap();
                    let description = team
                        .get_child("Description")
                        .unwrap_or_else(|| {
                            panic!("<Description> element missing in team id= {}", id_addr)
                        })
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let id: u64 = convert_from_address_to_id(id_addr, "Team - id");
                    // TODO also read the members and their roles and put in the member table.
                    db_conn
                        .execute(
                            "INSERT INTO team (id, name, description)
                            VALUES (?1, ?2, ?3)",
                            (id, &name.to_string(), &description.to_string()),
                        )
                        .expect("Unable to insert data into team");
                }
            }
            _ => {}
        }
    }
}

// TODO maybe use the file_id for the diagram keys.
fn populate_db_with_viewpackets(db_conn: &Connection, xml_root: &Element, file_id: u64) {
    // Insert the data
    for child in &xml_root.children {
        match child {
            XMLNode::Element(viewpacket) => {
                if viewpacket.name == "ViewPacket" {
                    // TODO can I get the line in the XML that is currently read?
                    let viewpacket_addr: String = viewpacket
                        .attributes
                        .get("Id")
                        .expect("Missing 'Id' attribute for ViewPacket")
                        .to_string();
                    let view_type = viewpacket.attributes.get("ViewType").unwrap();
                    let view_style = viewpacket.attributes.get("ViewStyle").unwrap();
                    let sort_order: u64 = viewpacket
                        .attributes
                        .get("SortOrder")
                        .unwrap()
                        .parse()
                        .unwrap();
                    let title = viewpacket
                        .get_child("Title")
                        .unwrap_or_else(|| {
                            panic!(
                                "<Title> element missing in ViewPacket id= {}",
                                viewpacket_addr
                            )
                        })
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let introduction = viewpacket
                        .get_child("Introduction")
                        .unwrap_or_else(|| {
                            panic!(
                                "<Introduction> element missing in ViewPacket id= {}",
                                viewpacket_addr
                            )
                        })
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let context_model_key = viewpacket
                        .get_child("ContextModelKey")
                        .unwrap_or_else(|| {
                            panic!(
                                "<ContextModelKey> element missing in ViewPacket id= {}",
                                viewpacket_addr
                            )
                        })
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let primary_display_key = viewpacket
                        .get_child("PrimaryDisplayKey")
                        .unwrap_or_else(|| {
                            panic!(
                                "<PrimaryDisplayKey> element missing in ViewPacket id= {}",
                                viewpacket_addr
                            )
                        })
                        .get_text()
                        .unwrap_or_else(|| "".to_string().into());
                    let component_addr: String = match viewpacket.get_child("ComponentId") {
                        Some(component_id_elem) => {
                            let component_id_text =
                                component_id_elem.get_text().unwrap_or("0.0.0.0".into());
                            component_id_text.parse().unwrap()
                        }
                        None => "0.0.0.0".to_string(),
                    };

                    let team_addr: String = match viewpacket.get_child("TeamId") {
                        Some(team_id_elem) => {
                            let team_id_text = team_id_elem.get_text().unwrap_or("0.0.0.0".into());
                            team_id_text.parse().unwrap()
                        }
                        None => "0.0.0.0".to_string(),
                    };

                    if view_style == "Assignment" {
                        if team_addr == "0.0.0.0" {
                            eprintln!(
                                "!!! Warning: ViewPacket id= {} has viewStyle 'Assignment' but no TeamId assigned.",
                                viewpacket_addr
                            );
                        }
                    } else {
                        if component_addr == "0.0.0.0" {
                            eprintln!(
                                "!!! Warning: ViewPacket id= {} has no ComponentId assigned.",
                                viewpacket_addr
                            );
                        }
                    }

                    let viewpacket_id: u64 =
                        convert_from_address_to_id(viewpacket_addr, "ViewPacket - id");
                    let component_id: u64 = convert_from_address_to_id(
                        component_addr,
                        "ViewPacket - Component",
                    );
                    let team_id: u64 =
                        convert_from_address_to_id(team_addr, "ViewPacket - Team");
                    db_conn
                        .execute(
                            "INSERT INTO view_packet (component_id, context_model_key, file_id, primary_display_key, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                            (
                                component_id,
                                &context_model_key.to_string(),
                                file_id,
                                &primary_display_key.to_string(),
                                &introduction.to_string(),
                                sort_order,
                                team_id,
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
