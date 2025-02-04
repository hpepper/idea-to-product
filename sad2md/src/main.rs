use std::fs::File;
use std::io::BufReader;
use xmltree::{ Element, XMLNode };
use std::collections::HashMap;

use rusqlite::{Connection, Result};

const MODULE_VIEW_TYPE: &str = "Module";
const CNC_VIEW_TYPE: &str = "CnC";
const ALLOCATION_VIEW_TYPE: &str = "Allocation";

const VIEW_TYPE_LIST: &[&str] = &[MODULE_VIEW_TYPE, CNC_VIEW_TYPE, ALLOCATION_VIEW_TYPE];

const MODULE_VIEW_TYPE_STYLE_DECOMPOSITION: &str  = "Decomposition";
const MODULE_VIEW_TYPE_STYLE_USES: &str  = "Uses";
const MODULE_VIEW_TYPE_STYLE_GENERALIZE: &str  = "Generalize";
const MODULE_VIEW_TYPE_STYLE_LAYERED: &str  = "Layered";

const CNC_VIEW_TYPE_STYLE_CLIENTSERVER: &str  = "ClientServer";
const CNC_VIEW_TYPE_STYLE_PEERTOPEER: &str  = "PeerToPeer";
const CNC_VIEW_TYPE_STYLE_PUBSUB: &str  = "PublishSubscribe";
const CNC_VIEW_TYPE_STYLE_PIPEANDFILTER: &str  = "PipeAndFilter";
const CNC_VIEW_TYPE_STYLE_SHAREDDATA: &str  = "SharedData";

const ALLOCATION_VIEW_TYPE_STYLE_DEPLOYEMNET: &str  = "Deployment";
const ALLOCATION_VIEW_TYPE_STYLE_IMPLEMENTATION: &str  = "Implementation";
const ALLOCATION_VIEW_TYPE_STYLE_ASSIGNMENT: &str  = "Assignment";

#[derive(Debug)]
struct ViewPacket {
    component_id: i32,
    context_model_key: String,
    diagram_key: String,
    introduction: String,
    sort_order: i32,
    title: String,
    view_style: String,
    view_type: String,
    viewpacket_id: i32,
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
    map.insert(MODULE_VIEW_TYPE, vec![MODULE_VIEW_TYPE_STYLE_DECOMPOSITION, MODULE_VIEW_TYPE_STYLE_USES, MODULE_VIEW_TYPE_STYLE_GENERALIZE, MODULE_VIEW_TYPE_STYLE_LAYERED]);
    map.insert(
        CNC_VIEW_TYPE,
        vec![CNC_VIEW_TYPE_STYLE_CLIENTSERVER, CNC_VIEW_TYPE_STYLE_PEERTOPEER, CNC_VIEW_TYPE_STYLE_PUBSUB, CNC_VIEW_TYPE_STYLE_PIPEANDFILTER, CNC_VIEW_TYPE_STYLE_SHAREDDATA]
    );
    map.insert(ALLOCATION_VIEW_TYPE, vec![ALLOCATION_VIEW_TYPE_STYLE_DEPLOYEMNET, ALLOCATION_VIEW_TYPE_STYLE_IMPLEMENTATION, ALLOCATION_VIEW_TYPE_STYLE_ASSIGNMENT]);
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

    render_document(&db_conn);


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
    // Create the tables
    db_conn.execute(
        "CREATE TABLE view_packet (
            component_id INTEGER NOT NULL,
            context_model_key TEXT,
            diagram_key TEXT,
            introduction TEXT,
            sort_order INTEGER NOT NULL,
            title TEXT,
            view_style TEXT NOT NULL,
            view_type TEXT NOT NULL,
            viewpacket_id INTEGER PRIMARY KEY
        )",
        [],
    )
    .expect("Unable to create table");

    // Insert the data
    for child in &xml_root.children {
        match child {
            XMLNode::Element(viewpacket) => {
                if viewpacket.name == "ViewPacket" {
                    // TODO can I get the line in the XML that is currently read?
                    let viewpacket_id: i32 = viewpacket.attributes.get("Id").expect("Missing 'Id' attribute for ViewPacket").parse().unwrap();
                    let view_type = viewpacket.attributes.get("ViewType").unwrap();
                    let view_style = viewpacket.attributes.get("ViewStyle").unwrap();
                    let sort_order: i32 = viewpacket.attributes.get("SortOrder").unwrap().parse().unwrap();
                    let title = viewpacket.get_child("Title").unwrap_or_else(|| panic!("<Title> element missing in ViewPacket id= {}", viewpacket_id)).get_text().unwrap_or_else(|| "".to_string().into());
                    let introduction = viewpacket.get_child("Introduction").unwrap_or_else(|| panic!("<Introduction> element missing in ViewPacket id= {}", viewpacket_id)).get_text().unwrap_or_else(|| "".to_string().into());
                    let context_model_key = viewpacket.get_child("ContextModelKey").unwrap_or_else(|| panic!("<ContextModelKey> element missing in ViewPacket id= {}", viewpacket_id)).get_text().unwrap_or_else(|| "".to_string().into());
                    let diagram_key = viewpacket.get_child("DiagramKey").unwrap_or_else(|| panic!("<DiagramKey> element missing in ViewPacket id= {}", viewpacket_id)).get_text().unwrap_or_else(|| "".to_string().into());
                    let component_id: i32 = viewpacket.get_child("ComponentId").unwrap().get_text().unwrap().parse().unwrap();

                    db_conn
                        .execute(
                            "INSERT INTO view_packet (component_id, context_model_key, diagram_key, introduction, sort_order, title, view_style, view_type, viewpacket_id)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            (
                                component_id,
                                &context_model_key.to_string(),
                                &diagram_key.to_string(),
                                &introduction.to_string(),
                                sort_order,
                                &title.to_string(),
                                view_style,
                                view_type,
                                viewpacket_id,
                            ),
                        )
                        .expect("Unable to insert data");
                }
            }
            _ => {}
        }
    }

    
}

fn render_document(db_conn: &Connection) {
    let styles_map = create_styles();
    // get list of keys from the hashmap
    for view_type in VIEW_TYPE_LIST {
        println!("view_type: {}", view_type);
        // TODO ## view_type
        if let Some(styles) = styles_map.get(view_type) {
            for style in styles {
                println!("  Style: {}", style);
                render_viewpacket(db_conn, view_type, style);
                // Get list of Viewpacket elements for the given view_type and style
            }
        }
    }
}

fn render_viewpacket(db_conn: &Connection, view_type: &str, style: &str) {

    // TODO maybe call this in the parent function so it only gets called once.
    let type_and_style_to_section_number = create_hardcoded_map();

    let mut stmt = db_conn.prepare(
        "SELECT component_id, context_model_key, diagram_key, introduction, sort_order, title, view_style, view_type, viewpacket_id
        FROM view_packet
        WHERE view_type = ?1 AND view_style = ?2
        ORDER BY sort_order"
    ).unwrap();

    let viewpacket_iter = stmt.query_map([view_type, style], |row| {
        Ok(ViewPacket {
            component_id: row.get(0)?,
            context_model_key: row.get(1)?,
            diagram_key: row.get(2)?,
            introduction: row.get(3)?,
            sort_order: row.get(4)?,
            title: row.get(5)?,
            view_style: row.get(6)?,
            view_type: row.get(7)?,
            viewpacket_id: row.get(8)?,
        })
    }).unwrap();

    for viewpacket in viewpacket_iter {
        match viewpacket {
            Ok(viewpacket) => {
                println!("### {view_type} {}.{}.{}: {}", type_and_style_to_section_number[view_type], type_and_style_to_section_number[style], viewpacket.sort_order, viewpacket.title);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
            
        }
    }
}