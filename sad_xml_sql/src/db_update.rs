use rusqlite::Connection;

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
