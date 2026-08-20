use rusqlite::Connection;

pub fn db_create_in_mem_db(db_conn: &Connection) {
    create_table_behaviors(db_conn);
    create_table_componentrelations(db_conn);
    create_table_components(db_conn);
    create_table_context_model(db_conn);
    create_table_document(db_conn);
    create_table_include(db_conn);
    create_table_team(db_conn);
    create_table_viewpackets(db_conn);
}

fn create_table_behaviors(db_conn: &Connection) {
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
            [],
        )
        .expect("Unable to create table - behavior");
}

fn create_table_components(db_conn: &Connection) {
    // Create the tables
    db_conn
        .execute(
            "CREATE TABLE component (
            file_id INTEGER,
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            purpose TEXT,
            summary TEXT,
            team_id INTEGER
        )",
            [],
        )
        .expect("Unable to create table");
}

fn create_table_componentrelations(db_conn: &Connection) {
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
            relation_description TEXT,
            style TEXT
        )",
            [],
        )
        .expect("Unable to create table");
}

// This table gets populated when a call to get_vector_of_context_model_by_key() is executed.
fn create_table_context_model(db_conn: &Connection) {
    // Create the tables
    db_conn
        .execute(
            "CREATE TABLE IF NOT EXISTS context_model (
        key TEXT,
        entity TEXT,
        entity_type TEXT,
        description TEXT,
        reference TEXT
    )",
            [],
        )
        .expect("Failed to create context_model table");
}

fn create_table_document(db_conn: &Connection) {
    // Create the tables
    db_conn
        .execute(
            "CREATE TABLE IF NOT EXISTS document (
        file_id INTEGER PRIMARY KEY,
        filename TEXT,
        title TEXT,
        issue TEXT,
        summary TEXT
    )",
            [],
        )
        .expect("Failed to create file table");
}


fn create_table_include(db_conn: &Connection) {
    // Create the tables
    db_conn
        .execute(
            "CREATE TABLE IF NOT EXISTS include (
        file_id INTEGER PRIMARY KEY,
        filename TEXT,
        url TEXT
    )",
            [],
        )
        .expect("Failed to create include table");
}

fn create_table_team(db_conn: &Connection) {
    // Create the tables
    db_conn
        .execute(
            "CREATE TABLE IF NOT EXISTS team (
        description TEXT,
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL
    )",
            [],
        )
        .expect("Failed to create team table");
}

// fn create_table_team_member(db_conn: &Connection) {
//     // Create the tables
//     db_conn
//         .execute(
//             "CREATE TABLE IF NOT EXISTS team_member (
//         name TEXT NOT NULL,
//         role TEXT,
//         team_id INTEGER
//     )",
//             [],
//         )
//         .expect("Failed to create team_member table");
// }

fn create_table_viewpackets(db_conn: &Connection) {
    // Create the tables
    db_conn
        .execute(
            "CREATE TABLE view_packet (
            component_id INTEGER NOT NULL,
            context_model_key TEXT,
            primary_display_key TEXT,
            file_id INTEGER,
            introduction TEXT,
            sort_order INTEGER NOT NULL,
            team_id INTEGER,
            title TEXT,
            view_style TEXT NOT NULL,
            view_type TEXT NOT NULL,
            viewpacket_id INTEGER PRIMARY KEY
        )",
            [],
        )
        .expect("Unable to create table");
}
