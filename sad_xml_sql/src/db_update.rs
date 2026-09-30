use rusqlite::{Connection};
use crate::models::Component;
use crate::db_dump_to_xml::convert_id_to_address;

/// TODO be able to detect duplicates and do not insert them,
pub fn insert_into_context_model_ignore_duplicates(
    db_conn: &Connection,
    key: &str,
    entity: &str,
    entity_type: &str,
    description: &str,
    reference: &str,
) {
    if !entity.is_empty() {
        // Insert into context_model table, ignoring duplicates
        db_conn
        .execute(
            "INSERT OR IGNORE INTO context_model (key, entity, entity_type, description, reference)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (key, entity, entity_type, description, reference),
        )
        .expect("Failed to insert into context_model");
    }
}

pub fn insert_into_document(
    db_conn: &Connection,
    file_id: u64,
    filename: &str,
    title: &str,
    issue: &str,
    summary: &str,
) {
    db_conn
        .execute(
            "INSERT INTO document (file_id, filename, title, issue, summary)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            (file_id, filename, title, issue, summary),
        )
        .expect("Unable to insert data into document");
}

/// Insert a component into the `component` table.
///
/// ### Parameters
/// - `db_conn` - The database connection.
/// - `file_id` - The id of the file the component belongs to.
/// - `id` - The component id, unique across the database.
/// - `name` - The component name.
/// - `purpose` - The component purpose.
/// - `summary` - The component summary.
/// - `team_id` - The id of the team assigned to the component, or `0` if none is assigned.
/// - `title` - The human readable title of the component, or `""` if none is set.
///
/// ### Panics
/// Panics if the insert fails, for example when `id` is already in use.
pub fn insert_into_component(
    db_conn: &Connection,
    file_id: u64,
    id: u64,
    name: &str,
    purpose: &str,
    summary: &str,
    team_id: u64,
    title: &str,
) {
    db_conn
        .execute(
            "INSERT INTO component (file_id, id, name, purpose, summary, team_id, title)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (file_id, id, name, purpose, summary, team_id, title),
        )
        .expect(&format!("Unable to insert data id: {}", convert_id_to_address(id)));
}

pub fn insert_into_view_packet(
    db_conn: &Connection,
    component_id: u64,
    context_model_key: &str,
    file_id: u64,
    primary_display_key: &str,
    introduction: &str,
    sort_order: u64,
    team_id: u64,
    title: &str,
    view_style: &str,
    view_type: &str,
    viewpacket_id: u64,
) {
    db_conn
        .execute(
            "INSERT INTO view_packet (component_id, context_model_key, file_id, primary_display_key, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (
                component_id,
                context_model_key,
                file_id,
                primary_display_key,
                introduction,
                sort_order,
                team_id,
                title,
                view_style,
                view_type,
                viewpacket_id,
            ),
        )
        .expect("Unable to insert data");
}

pub fn insert_into_component_relation(
    db_conn: &Connection,
    id: u64,
    sort_order: u64,
    component_a_id: u64,
    component_b_id: u64,
    connection_type: &str,
    key: &str,
    property_of_relation: &str,
    relation_text: &str,
    relation_description: &str,
    style: &str,
) {
    db_conn
        .execute(
            "INSERT INTO component_relation (id, sort_order, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description, style)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (
                id,
                sort_order,
                component_a_id,
                component_b_id,
                connection_type,
                key,
                property_of_relation,
                relation_text,
                relation_description,
                style,
            ),
        )
        .expect("Unable to insert data in component_relation");
}

pub fn update_component_by_id(db_conn: &Connection, component: &Component) {
    db_conn
        .execute(
            "UPDATE component SET name = ?1, summary = ?2, purpose = ?3, title = ?4 WHERE id = ?5",
            (component.name.clone(), component.summary.clone(), component.purpose.clone(), component.title.clone(), component.id),
        )
        .expect("Failed to update component");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_create_in_mem_db;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db_create_in_mem_db(&conn);
        conn
    }

    #[test]
    fn test_insert_into_context_model_ignore_duplicates_inserts() {
        let conn = setup_db();
        insert_into_context_model_ignore_duplicates(
            &conn,
            "k1", "entity1", "type1", "desc1", "ref1"
        );
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM context_model WHERE key = 'k1' AND entity = 'entity1'",
            (),
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_into_context_model_ignore_duplicates_ignores_duplicate() {
        let conn = setup_db();
        insert_into_context_model_ignore_duplicates(
            &conn,
            "k1", "entity1", "type1", "desc1", "ref1"
        );
        insert_into_context_model_ignore_duplicates(
            &conn,
            "k1", "entity1", "type1", "desc1", "ref1"
        );
        insert_into_context_model_ignore_duplicates(
            &conn,
            "k1", "entity1", "type1", "desc1", "ref1"
        );
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM context_model WHERE key = 'k1' AND entity = 'entity1'",
            (),
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_into_context_model_ignore_duplicates_empty_entity() {
        let conn = setup_db();
        insert_into_context_model_ignore_duplicates(
            &conn,
            "k1", "", "type1", "desc1", "ref1"
        );
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM context_model",
            (),
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_insert_into_component_inserts() {
        let conn = setup_db();
        insert_into_component(&conn, 1, 2, "Name1", "Purpose1", "Summary1", 3, "Title1");
        let (file_id, name, purpose, summary, team_id, title): (u64, String, String, String, u64, String) = conn
            .query_row(
                "SELECT file_id, name, purpose, summary, team_id, title FROM component WHERE id = 2",
                (),
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .unwrap();
        assert_eq!(file_id, 1);
        assert_eq!(name, "Name1");
        assert_eq!(purpose, "Purpose1");
        assert_eq!(summary, "Summary1");
        assert_eq!(team_id, 3);
        assert_eq!(title, "Title1");
    }

    #[test]
    fn test_insert_into_component_zero_team_id() {
        let conn = setup_db();
        insert_into_component(&conn, 1, 2, "Name1", "Purpose1", "Summary1", 0, "");
        let team_id: u64 = conn
            .query_row(
                "SELECT team_id FROM component WHERE id = 2",
                (),
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(team_id, 0);
    }

    #[test]
    #[should_panic(expected = "Unable to insert data id")]
    fn test_insert_into_component_duplicate_id_panics() {
        let conn = setup_db();
        insert_into_component(&conn, 1, 2, "Name1", "Purpose1", "Summary1", 0, "");
        insert_into_component(&conn, 1, 2, "Name2", "Purpose2", "Summary2", 0, "");
    }

    #[test]
    fn test_insert_into_view_packet_inserts() {
        let conn = setup_db();
        insert_into_view_packet(
            &conn,
            1,
            "ContextKey1",
            2,
            "PrimaryKey1",
            "Introduction1",
            4,
            5,
            "Title1",
            "Decomposition",
            "Module",
            6,
        );
        let (component_id, context_model_key, file_id, primary_display_key, introduction, sort_order, team_id, title, view_style, view_type): (
            u64,
            String,
            u64,
            String,
            String,
            u64,
            u64,
            String,
            String,
            String,
        ) = conn
            .query_row(
                "SELECT component_id, context_model_key, file_id, primary_display_key, introduction, sort_order, team_id, title, view_style, view_type FROM view_packet WHERE viewpacket_id = 6",
                (),
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(component_id, 1);
        assert_eq!(context_model_key, "ContextKey1");
        assert_eq!(file_id, 2);
        assert_eq!(primary_display_key, "PrimaryKey1");
        assert_eq!(introduction, "Introduction1");
        assert_eq!(sort_order, 4);
        assert_eq!(team_id, 5);
        assert_eq!(title, "Title1");
        assert_eq!(view_style, "Decomposition");
        assert_eq!(view_type, "Module");
    }

    #[test]
    #[should_panic(expected = "Unable to insert data")]
    fn test_insert_into_view_packet_duplicate_id_panics() {
        let conn = setup_db();
        insert_into_view_packet(
            &conn, 1, "Key1", 2, "Primary1", "Intro1", 4, 5, "Title1", "Decomposition", "Module",
            6,
        );
        insert_into_view_packet(
            &conn, 1, "Key2", 2, "Primary2", "Intro2", 4, 5, "Title2", "Decomposition", "Module",
            6,
        );
    }

    #[test]
    fn test_insert_into_component_relation_inserts() {
        let conn = setup_db();
        insert_into_component_relation(
            &conn,
            1,
            2,
            3,
            4,
            "Uses",
            "Key1",
            "PropertyOfRelation1",
            "RelationText1",
            "RelationDescription1",
            "Style1",
        );
        let (
            sort_order,
            component_a_id,
            component_b_id,
            connection_type,
            key,
            property_of_relation,
            relation_text,
            relation_description,
            style,
        ): (u64, u64, u64, String, String, String, String, String, String) = conn
            .query_row(
                "SELECT sort_order, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description, style FROM component_relation WHERE id = 1",
                (),
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(sort_order, 2);
        assert_eq!(component_a_id, 3);
        assert_eq!(component_b_id, 4);
        assert_eq!(connection_type, "Uses");
        assert_eq!(key, "Key1");
        assert_eq!(property_of_relation, "PropertyOfRelation1");
        assert_eq!(relation_text, "RelationText1");
        assert_eq!(relation_description, "RelationDescription1");
        assert_eq!(style, "Style1");
    }

    #[test]
    fn test_insert_into_component_relation_allows_duplicate_ids() {
        let conn = setup_db();
        insert_into_component_relation(
            &conn, 1, 1, 2, 3, "Uses", "Key1", "", "", "", "",
        );
        insert_into_component_relation(
            &conn, 1, 2, 4, 5, "Uses", "Key2", "", "", "", "",
        );
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM component_relation WHERE id = 1",
                (),
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_update_component_by_id_updates_row() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO component (file_id, id, name, summary, purpose, team_id) VALUES (0,1, 'OldName', 'OldSummary', 'OldPurpose', 0)",
            (),
        ).unwrap();
        let component = Component {
            file_id: 0,
            id: 1,
            name: "NewName".to_string(),
            summary: "NewSummary".to_string(),
            purpose: "NewPurpose".to_string(),
            team_id: 0,
            title: "NewTitle".to_string(),
        };
        update_component_by_id(&conn, &component);
        let (name, summary, purpose, team_id, title): (String, String, String, i32, String) = conn.query_row(
            "SELECT name, summary, purpose, team_id, title FROM component WHERE id = 1",
            (),
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        ).unwrap();
        assert_eq!(name, "NewName");
        assert_eq!(summary, "NewSummary");
        assert_eq!(purpose, "NewPurpose");
        assert_eq!(team_id, 0);
        assert_eq!(title, "NewTitle");
    }

    #[test]
    fn test_update_component_by_id_nonexistent_id() {
        let conn = setup_db();
        let component = Component {
            file_id: 0,
            id: 42,
            name: "Name".to_string(),
            summary: "Summary".to_string(),
            purpose: "Purpose".to_string(),
            team_id: 0,
            title: "".to_string(),
        };
        update_component_by_id(&conn, &component);
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM component WHERE id = 42",
            (),
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 0);
    }
}
