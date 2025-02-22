//! sad2md is a tool to convert Software Architecture Document (SAD) XML files to Markdown.
//!
//! Testing it out:
//! `cargo run test/test_sad.xml && cat sad.md`

use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use xmltree::{ Element, XMLNode };
// A HashMap is a collection of key-value pairs. It allows you to store and retrieve values based on a unique key.
use std::collections::HashMap;
// A HashSet is a collection of unique values. It does not store key-value pairs, only unique keys.
use std::collections::HashSet;

use rusqlite::{ Connection, Result };

const MODULE_VIEW_TYPE: &str = "Module";
const CNC_VIEW_TYPE: &str = "CnC";
const ALLOCATION_VIEW_TYPE: &str = "Allocation";

const VIEW_TYPE_LIST: &[&str] = &[MODULE_VIEW_TYPE, CNC_VIEW_TYPE, ALLOCATION_VIEW_TYPE];

const MODULE_VIEW_TYPE_STYLE_DECOMPOSITION: &str = "Decomposition";
const MODULE_VIEW_TYPE_STYLE_USES: &str = "Uses";
const MODULE_VIEW_TYPE_STYLE_GENERALIZE: &str = "Generalize";
const MODULE_VIEW_TYPE_STYLE_LAYERED: &str = "Layered";

const CNC_VIEW_TYPE_STYLE_CLIENTSERVER: &str = "ClientServer";
const CNC_VIEW_TYPE_STYLE_PEERTOPEER: &str = "PeerToPeer";
const CNC_VIEW_TYPE_STYLE_PUBSUB: &str = "PublishSubscribe";
const CNC_VIEW_TYPE_STYLE_PIPEANDFILTER: &str = "PipeAndFilter";
const CNC_VIEW_TYPE_STYLE_SHAREDDATA: &str = "SharedData";

const ALLOCATION_VIEW_TYPE_STYLE_DEPLOYEMNET: &str = "Deployment";
const ALLOCATION_VIEW_TYPE_STYLE_IMPLEMENTATION: &str = "Implementation";
const ALLOCATION_VIEW_TYPE_STYLE_ASSIGNMENT: &str = "Assignment";

#[derive(Debug)]
struct ViewPacket {
    component_id: i32,
    context_model_key: String,
    primary_display_key: String,
    introduction: String,
    sort_order: i32,
    title: String,
    view_style: String,
    view_type: String,
    viewpacket_id: i32,
}

// TODO Add Id from attribute
#[derive(Debug)]
struct ComponentRelation {
    id: i32,
    component_a_id: i32,
    component_b_id: i32,
    connection_type: String,
    key: String,
    property_of_relation: String,
    relation_text: String,
}

#[derive(Debug)]
struct Component {
    id: i32,
    name: String,
    summary: String,
}

// TODO do I need sort_order and view_packet_id I do not think so
#[derive(Debug)]
struct Behavior {
    id: i32,
    sort_order: i32,
    view_packet_id: i32,
    description: String,
    diagram_key: String,
}

fn create_hardcoded_map() -> HashMap<&'static str, i32> {
    let mut map = HashMap::new();
    map.insert(MODULE_VIEW_TYPE, 1);
    map.insert(CNC_VIEW_TYPE, 2);
    map.insert(ALLOCATION_VIEW_TYPE, 3);
    map.insert(MODULE_VIEW_TYPE_STYLE_DECOMPOSITION, 1);
    map.insert(MODULE_VIEW_TYPE_STYLE_USES, 2);
    map.insert(MODULE_VIEW_TYPE_STYLE_GENERALIZE, 3);
    map.insert(MODULE_VIEW_TYPE_STYLE_LAYERED, 4);
    map.insert(CNC_VIEW_TYPE_STYLE_CLIENTSERVER, 1);
    map.insert(CNC_VIEW_TYPE_STYLE_PEERTOPEER, 2);
    map.insert(CNC_VIEW_TYPE_STYLE_PUBSUB, 3);
    map.insert(CNC_VIEW_TYPE_STYLE_PIPEANDFILTER, 4);
    map.insert(CNC_VIEW_TYPE_STYLE_SHAREDDATA, 5);
    map.insert(ALLOCATION_VIEW_TYPE_STYLE_DEPLOYEMNET, 1);
    map.insert(ALLOCATION_VIEW_TYPE_STYLE_IMPLEMENTATION, 2);
    map.insert(ALLOCATION_VIEW_TYPE_STYLE_ASSIGNMENT, 3);

    map
}

// Module: Decomposition, Uses, Generalize, Layered
// CnC: ClientServer, PeerToPeer, PublishSubscribe, PeerToPeer, PipeAndFilter, PublishSubscribe, SharedData
// Allocation: Deployment, Implemnetation, Assignment
fn create_styles() -> HashMap<&'static str, Vec<&'static str>> {
    let mut map = HashMap::new();
    map.insert(
        MODULE_VIEW_TYPE,
        vec![
            MODULE_VIEW_TYPE_STYLE_DECOMPOSITION,
            MODULE_VIEW_TYPE_STYLE_USES,
            MODULE_VIEW_TYPE_STYLE_GENERALIZE,
            MODULE_VIEW_TYPE_STYLE_LAYERED
        ]
    );
    map.insert(
        CNC_VIEW_TYPE,
        vec![
            CNC_VIEW_TYPE_STYLE_CLIENTSERVER,
            CNC_VIEW_TYPE_STYLE_PEERTOPEER,
            CNC_VIEW_TYPE_STYLE_PUBSUB,
            CNC_VIEW_TYPE_STYLE_PIPEANDFILTER,
            CNC_VIEW_TYPE_STYLE_SHAREDDATA
        ]
    );
    map.insert(
        ALLOCATION_VIEW_TYPE,
        vec![
            ALLOCATION_VIEW_TYPE_STYLE_DEPLOYEMNET,
            ALLOCATION_VIEW_TYPE_STYLE_IMPLEMENTATION,
            ALLOCATION_VIEW_TYPE_STYLE_ASSIGNMENT
        ]
    );
    map
}

fn main() {
    let db_conn = Connection::open_in_memory().expect("Connecting to the SQLite database failed.");

    // get filename from command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];

    // Parse the XML file
    let xml_root = load_xml_file(filename);

    populate_db(&db_conn, &xml_root);

    let markdown_filname = "sad.md";
    let mut markdown_file = File::create(markdown_filname).expect("Unable to create file");

    render_document(&mut markdown_file, &db_conn);
}

fn load_xml_file(filename: &str) -> Element {
    // verify the file exists
    if !std::path::Path::new(filename).exists() {
        eprintln!("File not found: {}", filename);
        std::process::exit(1);
    }

    let file = File::open(filename).expect("Unable to open file");
    let file = BufReader::new(file);

    // Parse the XML file
    Element::parse(file).expect("Unable to parse XML")
}

fn populate_db(db_conn: &Connection, xml_root: &Element) {
    populate_db_with_viewpackets(db_conn, xml_root);
    populate_db_with_components(db_conn, xml_root);
    populate_db_with_componentrelations(db_conn, xml_root);
    populate_db_with_behaviors(db_conn, xml_root);
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
            relation_text TEXT
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
                    db_conn
                        .execute(
                            "INSERT INTO component_relation (id, sort_order, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                            (
                                id,
                                sort_order,
                                component_a_id,
                                component_b_id,
                                &connection_type.to_string(),
                                &key.to_string(),
                                &property_of_relation.to_string(),
                                &relation_text.to_string()
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

fn render_document(markdown_file: &mut File, db_conn: &Connection) {
    let styles_map = create_styles();

    markdown_leadin(markdown_file);

    // get list of keys from the hashmap
    for view_type in VIEW_TYPE_LIST {
        println!("view_type: {}", view_type);
        markdown_file
            .write(&format!("## {}\n\n", view_type).as_bytes())
            .expect("Unable to write to file");
        // TODO ## view_type
        if let Some(styles) = styles_map.get(view_type) {
            for style in styles {
                println!("  Style: {}", style);
                render_viewpacket(markdown_file, db_conn, view_type, style);
                // Get list of Viewpacket elements for the given view_type and style
            }
        }
    }
}

fn render_viewpacket(markdown_file: &mut File, db_conn: &Connection, view_type: &str, style: &str) {
    // TODO maybe call this in the parent function so it only gets called once.
    let type_and_style_to_section_number = create_hardcoded_map();

    let mut stmt = db_conn
        .prepare(
            "SELECT component_id, context_model_key, primary_display_key, introduction, sort_order, title, view_style, view_type, viewpacket_id
        FROM view_packet
        WHERE view_type = ?1 AND view_style = ?2
        ORDER BY sort_order"
        )
        .unwrap();

    let viewpacket_iter = stmt
        .query_map([view_type, style], |row| {
            Ok(ViewPacket {
                component_id: row.get(0)?,
                context_model_key: row.get(1)?,
                primary_display_key: row.get(2)?,
                introduction: row.get(3)?,
                sort_order: row.get(4)?,
                title: row.get(5)?,
                view_style: row.get(6)?,
                view_type: row.get(7)?,
                viewpacket_id: row.get(8)?,
            })
        })
        .unwrap();

    for viewpacket in viewpacket_iter {
        match viewpacket {
            Ok(viewpacket) => {
                let section_number = format!(
                    "{}.{}.{}",
                    type_and_style_to_section_number[view_type],
                    type_and_style_to_section_number[style],
                    viewpacket.sort_order
                );

                let top_component: Option<Component> = match
                    get_component_by_id(db_conn, viewpacket.component_id)
                {
                    Ok(top_component) => Some(top_component),
                    Err(e) => {
                        eprintln!("Error retrieving component: {}", e);
                        None
                    }
                };
                // TODO have a function that generates this header, so I can easily use the function to generate the links
                markdown_file
                    .write(
                        &format!(
                            "### {view_type} {style} view packet {section_number}: {}\n\n",
                            viewpacket.title
                        ).as_bytes()
                    )
                    .expect("Unable to write to file");

                // TODO make the primary display a sub function, so this viewpacket function doesn become huge.

                markdown_file
                    .write(&format!("#### {section_number}: Primary presentation\n\n").as_bytes())
                    .expect("Unable to write to file");

                // generate the primary presentation mermaid diagram
                mermaid_leadin(markdown_file,"graph LR;");
                let top_component: Option<Component> = match
                    get_component_by_id(db_conn, viewpacket.component_id)
                {
                    // TODO can the 'if let Some' code below be put into a code block here?
                    Ok(top_component) => Some(top_component),
                    Err(e) => {
                        eprintln!("Error retrieving component: {}", e);
                        None
                    }
                };
                if let Some(top_component) = top_component {
                    render_graphical_primary_display(
                        markdown_file,
                        db_conn,
                        view_type,
                        style,
                        viewpacket.component_id,
                        viewpacket.primary_display_key.clone()
                    );
                }

                mermaid_leadout(markdown_file);

                let top_component: Option<Component> = match
                    get_component_by_id(db_conn, viewpacket.component_id)
                {
                    // TODO can the 'if let Some' code below be put into a code block here?
                    Ok(top_component) => Some(top_component),
                    Err(e) => {
                        eprintln!("Error retrieving component: {}", e);
                        None
                    }
                };
                if let Some(top_component) = top_component {
                    markdown_file
                        .write(
                            &format!(
                                "* {}: {}\n",
                                top_component.name,
                                top_component.summary
                            ).as_bytes()
                        )
                        .expect("Unable to write to file");

                    render_textual_primary_display(
                        markdown_file,
                        db_conn,
                        view_type,
                        style,
                        viewpacket.component_id,
                        viewpacket.primary_display_key,
                        1
                    );
                }

                markdown_file
                    .write(&format!("\n#### {section_number}: Context diagram\n\n").as_bytes())
                    .expect("Unable to write to file");

                // generate the mermaid diagram for context diagram
                mermaid_leadin(markdown_file, "graph TD;");

                let top_component: Option<Component> = match
                    get_component_by_id(db_conn, viewpacket.component_id)
                {
                    // TODO can the 'if let Some' code below be put into a code block here?
                    Ok(top_component) => Some(top_component),
                    Err(e) => {
                        eprintln!("Error retrieving component: {}", e);
                        None
                    }
                };
                if let Some(top_component) = top_component {
                    render_graphical_context_diagram(
                        markdown_file,
                        db_conn,
                        view_type,
                        style,
                        viewpacket.component_id,
                        viewpacket.context_model_key.clone()
                    );
                }

                mermaid_leadout(markdown_file);

                // TOOD generate the table

                markdown_file
                    .write(
                        &format!(
                            "#### {section_number}: Element catalog - Elements and their properties\n\n"
                        ).as_bytes()
                    )
                    .expect("Unable to write to file");

                // TODO generate list with elements and their properties

                markdown_file
                    .write(
                        &format!(
                            "#### {section_number}: Element catalog - Element behavior\n\n"
                        ).as_bytes()
                    )
                    .expect("Unable to write to file");
                render_graphical_all_behaviors_for_viewpacket(
                    markdown_file,
                    db_conn,
                    view_type,
                    style,
                    viewpacket.viewpacket_id
                );

                markdown_file
                    .write(&format!("#### {section_number}: Related views\n\n").as_bytes())
                    .expect("Unable to write to file");

                markdown_file
                    .write(&format!("* Parent:\n").as_bytes())
                    .expect("Unable to write to file");
                // TODO find the parent viewpacket
                markdown_file
                    .write(&format!("* Siblings:\n").as_bytes())
                    .expect("Unable to write to file");
                // TODO find the siblings viewpacket
                markdown_file
                    .write(&format!("* Children:\n").as_bytes())
                    .expect("Unable to write to file");
                // TODO find the children viewpacket
                markdown_file.write(&format!("\n").as_bytes()).expect("Unable to write to file");
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
}

fn render_graphical_primary_display(
    markdown_file: &mut File,
    db_conn: &Connection,
    view_type: &str,
    style: &str,
    component_id: i32,
    primary_display_key: String
) {
    // TODO do I need this?
    let top_component: Option<Component> = match get_component_by_id(db_conn, component_id) {
        Ok(top_component) => Some(top_component),
        Err(e) => {
            eprintln!("Error retrieving component: {}", e);
            None
        }
    };

    if let Some(top_component) = top_component {
        let component_relations_vector = get_vector_of_related_components_by_id_and_key(
            db_conn,
            component_id,
            primary_display_key.clone()
        );
        match component_relations_vector {
            Ok(component_relations_vector) => {
                for component_relation in component_relations_vector {
                    let component_b = get_component_by_id(
                        db_conn,
                        component_relation.component_b_id
                    );
                    match component_b {
                        Ok(component_b) => {
                            let component_a_name = get_component_name_by_id(db_conn, component_id);
                            let linkable_component_a_name = make_linkable_text(
                                component_a_name.clone()
                            );
                            let linkable_component_b_name = make_linkable_text(
                                component_b.name.clone()
                            );

                            markdown_file
                                .write(
                                    &format!(
                                        "    {}[{}]-->{}[{}]\n",
                                        linkable_component_a_name,
                                        component_a_name,
                                        linkable_component_b_name,
                                        component_b.name
                                    ).as_bytes()
                                )
                                .expect("Unable to write to file");
                            render_graphical_primary_display(
                                markdown_file,
                                db_conn,
                                view_type,
                                style,
                                component_relation.component_b_id,
                                primary_display_key.clone()
                            );
                        }
                        Err(err) => {
                            eprintln!(
                                "Error: returned from get_component_by_id() for component id = {} - {}",
                                component_relation.component_b_id,
                                err
                            );
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: returned from get_vector_of_related_components_by_id_and_key() {}", err);
            }
        }
    }
}

fn render_graphical_context_diagram(
    markdown_file: &mut File,
    db_conn: &Connection,
    view_type: &str,
    style: &str,
    component_id: i32,
    context_model_key: String
) {
    // TODO do I need this?
    let top_component: Option<Component> = match get_component_by_id(db_conn, component_id) {
        Ok(top_component) => Some(top_component),
        Err(e) => {
            eprintln!("Error retrieving component: {}", e);
            None
        }
    };

    if let Some(top_component) = top_component {
        let component_relations_vector = get_vector_of_related_components_by_id_and_key(
            db_conn,
            component_id,
            context_model_key.clone()
        );
        match component_relations_vector {
            Ok(component_relations_vector) => {
                for component_relation in component_relations_vector {
                    let component_b = get_component_by_id(
                        db_conn,
                        component_relation.component_b_id
                    );
                    match component_b {
                        Ok(component_b) => {
                            let component_a_name = get_component_name_by_id(db_conn, component_id);
                            let linkable_component_a_name = make_linkable_text(
                                component_a_name.clone()
                            );
                            let linkable_component_b_name = make_linkable_text(
                                component_b.name.clone()
                            );

                            markdown_file
                                .write(
                                    &format!(
                                        "    {}[{}]-->{}(({}))\n",
                                        linkable_component_b_name,
                                        component_b.name,
                                        linkable_component_a_name,
                                        component_a_name
                                    ).as_bytes()
                                )
                                .expect("Unable to write to file");
                            render_graphical_context_diagram(
                                markdown_file,
                                db_conn,
                                view_type,
                                style,
                                component_relation.component_b_id,
                                context_model_key.clone()
                            );
                        }
                        Err(err) => {
                            eprintln!(
                                "Error: returned from get_component_by_id() for component id = {} - {}",
                                component_relation.component_b_id,
                                err
                            );
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: returned from get_vector_of_related_components_by_id_and_key() {}", err);
            }
        }
    }
}

fn render_textual_primary_display(
    markdown_file: &mut File,
    db_conn: &Connection,
    view_type: &str,
    style: &str,
    component_id: i32,
    primary_display_key: String,
    indent_level: usize
) {
    // TODO do I need this?
    let top_component: Option<Component> = match get_component_by_id(db_conn, component_id) {
        Ok(top_component) => Some(top_component),
        Err(e) => {
            eprintln!("Error retrieving component: {}", e);
            None
        }
    };

    if let Some(top_component) = top_component {
        let indent_spaces = " ".repeat(indent_level * 2);
        let component_relations_vector = get_vector_of_related_components_by_id_and_key(
            db_conn,
            component_id,
            primary_display_key
        );
        match component_relations_vector {
            Ok(component_relations_vector) => {
                let indent_spaces = " ".repeat(indent_level * 2);
                for component_relation in component_relations_vector {
                    let component_b = get_component_by_id(
                        db_conn,
                        component_relation.component_b_id
                    );
                    match component_b {
                        Ok(component_b) => {
                            markdown_file
                                .write(
                                    &format!(
                                        "{}* {}: {}\n",
                                        indent_spaces,
                                        component_b.name,
                                        component_b.summary
                                    ).as_bytes()
                                )
                                .expect("Unable to write to file");
                            render_textual_primary_display(
                                markdown_file,
                                db_conn,
                                view_type,
                                style,
                                component_relation.component_b_id,
                                component_relation.property_of_relation,
                                indent_level + 1
                            );
                        }
                        Err(err) => {
                            eprintln!(
                                "Error: returned from get_component_by_id() for component id = {} - {}",
                                component_relation.component_b_id,
                                err
                            );
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: returned from get_vector_of_related_components_by_id_and_key() {}", err);
            }
        }
        // TODO recursively retrieve the related components.
        // TODO I probably need to use a key for this relation as well.
    }
}



/// Itterates through all behaviors that are linked to the given view packet id.
fn render_graphical_all_behaviors_for_viewpacket(
    markdown_file: &mut File,
    db_conn: &Connection,
    view_type: &str,
    style: &str,
    viewpacket_id: i32
) {
    let behaviors_vector = get_vector_of_behaviors_for_viewpacket_id(db_conn, viewpacket_id);
    match behaviors_vector {
        Ok(behaviors_vector) => {
            for behavior in behaviors_vector {
                // TODO Create a title entry for the behavior
                //

                let component_relations_vector = get_vector_of_related_components_by_key(
                    db_conn,
                    behavior.diagram_key.clone()
                );

                match component_relations_vector {
                    Ok(component_relations_vector) => {
                        mermaid_leadin(markdown_file, "sequenceDiagram");
                        let mut encountered_names: HashSet<String> = HashSet::new();

                        for component_relation in component_relations_vector {
                            let component_b = get_component_by_id(
                                db_conn,
                                component_relation.component_b_id
                            );
                            match component_b {
                                Ok(component_b) => {
                                    let component_a_name = get_component_name_by_id(
                                        db_conn,
                                        component_relation.component_a_id
                                    );
                                    let linkable_component_a_name = make_linkable_text(
                                        component_a_name.clone()
                                    );
                                    let linkable_component_b_name = make_linkable_text(
                                        component_b.name.clone()
                                    );

                                    // TODO Turn these two code blocks into function calls.
                                    if !encountered_names.contains(&linkable_component_a_name) {
                                        // Process the component
                                        markdown_file
                                        .write(
                                            &format!(
                                                "    participant {} as {}\n",
                                                linkable_component_a_name,
                                                component_a_name
                                            ).as_bytes()
                                        )
                                        .expect("Unable to write to file");
                                        // Add the name to the HashSet
                                        encountered_names.insert(linkable_component_a_name.clone());
                                    }
                                    if !encountered_names.contains(&linkable_component_b_name) {
                                        // Process the component
                                        markdown_file
                                        .write(
                                            &format!(
                                                "    participant {} as {}\n",
                                                linkable_component_b_name,
                                                component_b.name
                                            ).as_bytes()
                                        )
                                        .expect("Unable to write to file");
                                        // Add the name to the HashSet
                                        encountered_names.insert(linkable_component_b_name.clone());
                                    }


                                    // TODO C Get PropertyOfRelation and put it in the graph if it is not empty
                                    
                                    let connection_arrow_text = match component_relation.connection_type.to_lowercase().as_str() {
                                        "response" => "-->>",
                                        _ => "->>",
                                    };

                                    markdown_file
                                        .write(
                                            &format!(
                                                "    {}{}{}: {}\n",
                                                linkable_component_a_name,
                                                connection_arrow_text,
                                                linkable_component_b_name,
                                                component_relation.relation_text
                                            ).as_bytes()
                                        )
                                        .expect("Unable to write to file");
                                }
                                Err(err) => {
                                    eprintln!(
                                        "Error: returned from get_component_by_id() for component id = {} - {}",
                                        component_relation.component_b_id,
                                        err
                                    );
                                }
                            }
                        }
                        mermaid_leadout(markdown_file);
                    }
                    Err(err) => {
                        eprintln!("Error: returned from get_vector_of_related_components_by_key() {}", err);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Error: returned from get_vector_of_behaviors_for_viewpacket_id() {}", err);
        }
    }
}

fn get_component_name_by_id(db_conn: &Connection, component_id: i32) -> String {
    let mut stmt = db_conn.prepare("SELECT name FROM component WHERE id = ?1").unwrap();
    let component_name = stmt.query_row([component_id], |row| { Ok(row.get(0)?) }).unwrap();
    component_name
}

fn make_linkable_text(text: String) -> String {
    text.replace(" ", "")
}

/// Get all the behaviors that are linked to the given view packet id.
fn get_vector_of_behaviors_for_viewpacket_id(
    db_conn: &Connection,
    viewpacket_id: i32
) -> Result<Vec<Behavior>> {
    let mut stmt = db_conn.prepare(
        "SELECT id, sort_order, description, diagram_key FROM behavior WHERE view_packet_id = ?1 ORDER BY sort_order"
    )?;
    let behavior = stmt
        .query_map(rusqlite::params![viewpacket_id], |row| {
            Ok(Behavior {
                id: row.get(0)?,
                sort_order: row.get(1)?,
                view_packet_id: viewpacket_id,
                description: row.get(2)?,
                diagram_key: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(behavior)
}

// get a list of related components by component id and key
fn get_vector_of_related_components_by_id_and_key(
    db_conn: &Connection,
    component_id: i32,
    primary_display_key: String
) -> Result<Vec<ComponentRelation>> {
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text FROM component_relation WHERE component_a_id = ?1 AND key = ?2 ORDER BY id"
    )?;
    let component_relations = stmt
        .query_map(rusqlite::params![component_id, primary_display_key], |row| {
            Ok(ComponentRelation {
                component_a_id: row.get(0)?,
                component_b_id: row.get(1)?,
                connection_type: row.get(2)?,
                id: row.get(3)?,
                key: row.get(4)?,
                property_of_relation: row.get(5)?,
                relation_text: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(component_relations)
}

/// Filter only by key, return vector sorted by sort_order.
fn get_vector_of_related_components_by_key(
    db_conn: &Connection,
    key: String
) -> Result<Vec<ComponentRelation>> {
    // TODO probably remove 'id'
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text FROM component_relation WHERE key = ?1 ORDER BY sort_order"
    )?;
    let component_relations = stmt
        .query_map(rusqlite::params![key], |row| {
            Ok(ComponentRelation {
                component_a_id: row.get(0)?,
                component_b_id: row.get(1)?,
                connection_type: row.get(2)?,
                id: row.get(3)?,
                key: row.get(4)?,
                property_of_relation: row.get(5)?,
                relation_text: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(component_relations)
}

fn get_component_by_id(db_conn: &Connection, component_id: i32) -> Result<Component> {
    let mut stmt = db_conn.prepare("SELECT id, name, summary FROM component WHERE id = ?1")?;
    let component = stmt.query_row([component_id], |row| {
        Ok(Component {
            id: row.get(0)?,
            name: row.get(1)?,
            summary: row.get(2)?,
        })
    })?;
    Ok(component)
}

fn mermaid_leadin(markdown_file: &mut File, diagram_type: &str) {
    markdown_file
        .write(format!("```mermaid\n  {}\n", diagram_type).as_bytes())
        .expect("Unable to write to file");
}

fn markdown_leadin(markdown_file: &mut File) {
    markdown_file
        .write("# Software Architecture Document\n\n".as_bytes())
        .expect("Unable to write to file");
}

fn mermaid_leadout(markdown_file: &mut File) {
    markdown_file
        .write("```\n\n".as_bytes())
        .expect("Unable to write to file");
}

