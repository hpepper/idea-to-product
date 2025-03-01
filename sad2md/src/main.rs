//! sad2md is a tool to convert Software Architecture Document (SAD) XML files to Markdown.
//!
//! Testing it out:
//! `cargo run test/test_sad.xml && cat sad.md`

mod db_population;
mod db_retrieval;
mod models;

use db_population::populate_db;
use db_retrieval::{
    get_component_by_id,
    get_component_name_by_id,
    get_vector_of_behaviors_for_viewpacket_id,
    get_vector_of_related_components_by_id_and_key,
    get_vector_of_related_components_by_key,
};
use models::{ Behavior, Component, ComponentRelation, ViewPacket };

use rusqlite::Connection;

use std::fs::File;
use std::io::Write;
// A HashMap is a collection of key-value pairs. It allows you to store and retrieve values based on a unique key.
use std::collections::HashMap;
// A HashSet is a collection of unique values. It does not store key-value pairs, only unique keys.
use std::collections::HashSet;

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

    populate_db(&db_conn, filename);

    let markdown_filname = "Architecture.md";
    let mut markdown_file = File::create(markdown_filname).expect("Unable to create file");

    render_document(&mut markdown_file, &db_conn);
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

                // TODO find out what the leadin need to be for layers
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
                if view_type == MODULE_VIEW_TYPE && style == MODULE_VIEW_TYPE_STYLE_LAYERED {
                    // generate a layered mermaid diagram
                    mermaid_leadin(markdown_file, "block-beta\n    columns 1");
                    if let Some(top_component) = top_component {
                        render_graphical_layered_display(
                            markdown_file,
                            db_conn,
                            view_type,
                            style,
                            viewpacket.component_id,
                            viewpacket.primary_display_key.clone(),
                            true
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
                        0,
                        0
                    );
                }

                } else {
                    // generate the primary presentation mermaid diagram
                    mermaid_leadin(markdown_file, "graph LR;");
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
                    // TODO put this whole part in a function so it can be shared with the layered display, iwth just the parms changed.
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
                            1,
                            1
                        );
                    }
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

/// Renders the graphical representation of the layered display.
/// ### Parameters
/// - `markdown_file` - The markdown file to write to.
/// - `db_conn` - The database connection.
/// - `view_type` - The type of view.
/// - `style` - The style of view.
/// - `component_id` - The id of the upper layer that needs to be connected to a lower layer.
/// - `primary_display_key` - The key for filter the components when selecting from RelatedCp,åpmemts.
/// - `first_layer` - A boolean to indicate if this is the first layer.
fn render_graphical_layered_display(
    markdown_file: &mut File,
    db_conn: &Connection,
    view_type: &str,
    style: &str,
    component_id: i32,
    primary_display_key: String,
    first_layer: bool
) {
    println!(
        "render_graphical_layered_display {} {} {}",
        component_id,
        primary_display_key,
        first_layer
    );

    if first_layer {
        let component_a_name = get_component_name_by_id(db_conn, component_id);
        let linkable_component_a_name = make_linkable_text(component_a_name.clone());
        markdown_file
            .write(
                &format!("    {}[\"{}\"]\n", linkable_component_a_name, component_a_name).as_bytes()
            )
            .expect("Unable to write to file");
    }

    // TODO do I need this?
    let top_component: Option<Component> = match get_component_by_id(db_conn, component_id) {
        Ok(top_component) => Some(top_component),
        Err(e) => {
            eprintln!("Error retrieving component: {}", e);
            None
        }
    };

    // TODO print the current layer, if this is firt layer, before looking for lated layer.
    if let Some(top_component) = top_component {
        let component_relations_vector = get_vector_of_related_components_by_id_and_key(
            db_conn,
            component_id,
            primary_display_key.clone()
        );
        // TODO something along the lines of if there are no more relations the print the b component(it is the lat)
        match component_relations_vector {
            Ok(component_relations_vector) => {
                let last_component = component_relations_vector.len() == 0;
                for component_relation in component_relations_vector {
                    let component_b = get_component_by_id(
                        db_conn,
                        component_relation.component_b_id
                    );
                    match component_b {
                        Ok(component_b) => {
                            let linkable_component_b_name = make_linkable_text(
                                component_b.name.clone()
                            );

                            // TODO the first call must do both layers, the next calls must only do the second layer.
                            markdown_file
                                .write(
                                    &format!(
                                        "    {}[\"{}\"]\n",
                                        linkable_component_b_name,
                                        component_b.name
                                    ).as_bytes()
                                )
                                .expect("Unable to write to file");
                            // TODO recurse the next layer

                            render_graphical_layered_display(
                                markdown_file,
                                db_conn,
                                view_type,
                                style,
                                component_relation.component_b_id,
                                primary_display_key.clone(),
                                false
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
                // TODO it seems this code isn't needed.
                if false {
                    //if last_component {
                    let component_a_name = get_component_name_by_id(db_conn, component_id);
                    let linkable_component_a_name = make_linkable_text(component_a_name.clone());
                    markdown_file
                        .write(
                            &format!(
                                "    {}[\"{}\"]\n",
                                linkable_component_a_name,
                                component_a_name
                            ).as_bytes()
                        )
                        .expect("Unable to write to file");
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
    indent_level: usize,
    indent_increment: usize
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
            primary_display_key.clone()
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
                                //component_relation.property_of_relation,
                                primary_display_key.clone(),
                                indent_level + indent_increment,
                                indent_increment
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

                                    let connection_arrow_text = match
                                        component_relation.connection_type.to_lowercase().as_str()
                                    {
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

fn make_linkable_text(text: String) -> String {
    text.replace(" ", "")
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
    markdown_file.write("```\n\n".as_bytes()).expect("Unable to write to file");
}
