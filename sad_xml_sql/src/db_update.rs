use rusqlite::{Connection};
use crate::models::Component;


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

pub fn update_component_by_id(db_conn: &Connection, component: &Component) {
    db_conn
        .execute(
            "UPDATE component SET name = ?1, summary = ?2, purpose = ?3 WHERE id = ?4",
            (component.name.clone(), component.summary.clone(), component.purpose.clone(), component.id),
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
        };
        update_component_by_id(&conn, &component);
        let (name, summary, purpose, team_id): (String, String, String, i32) = conn.query_row(
            "SELECT name, summary, purpose, team_id FROM component WHERE id = 1",
            (),
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        ).unwrap();
        assert_eq!(name, "NewName");
        assert_eq!(summary, "NewSummary");
        assert_eq!(purpose, "NewPurpose");
        assert_eq!(team_id, 0);
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
