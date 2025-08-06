use crate::models::{Behavior, Component, ComponentRelation, ContextModel, ViewPacket};

use rusqlite::{Connection, Result};

pub fn get_component_name_by_id(db_conn: &Connection, component_id: i32) -> String {
    let mut stmt = db_conn
        .prepare("SELECT name FROM component WHERE id = ?1")
        .unwrap();
    let component_name = stmt
        .query_row([component_id], |row| Ok(row.get(0)?))
        .unwrap();
    component_name
}

pub fn get_vector_of_component_names_sorted(db_conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = db_conn
        .prepare("SELECT name FROM component ORDER BY name COLLATE NOCASE ASC")
        .unwrap();
    let component_names = stmt
        .query_map([], |row| Ok(row.get(0)?))
        .unwrap();
    component_names
        .collect::<Result<Vec<String>, _>>()
}

pub fn get_vector_of_context_model_by_key(
    db_conn: &Connection,
    key: &str,
) -> Result<Vec<ContextModel>> {
    let mut stmt = db_conn.prepare(
        "SELECT entity, entity_type, description, reference FROM context_model WHERE key = ?1 ORDER BY LOWER(entity)"
    )?;
    let context_model = stmt
        .query_map(rusqlite::params![key], |row| {
            Ok(ContextModel {
                entity: row.get(0)?,
                entity_type: row.get(1)?,
                description: row.get(2)?,
                reference: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(context_model)
}

/// Get all the behaviors that are linked to the given view packet id.
pub fn get_vector_of_behaviors_for_viewpacket_id(
    db_conn: &Connection,
    viewpacket_id: i32,
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

/**
 * if search_component_id == 0, then return all view packets for the given view_type and style
 * if search_component_id != 0, then return all view packets for the given component_id except for the given viewpacket_id
 */
pub fn get_vector_of_viewpacket_by_component_id_except_component_id(
    db_conn: &Connection,
    search_component_id: i32,
    view_type: &str,
    style: &str,
    exclude_viewpacket_id: i32,
) -> Result<Vec<ViewPacket>> {
    let viewpacket_vector = if search_component_id == 0 {
        let mut stmt = db_conn.prepare(        "SELECT component_id, context_model_key, primary_display_key, introduction, sort_order, title, view_style, view_type, viewpacket_id FROM view_packet  WHERE view_type = ?1 AND view_style = ?2 ORDER BY sort_order")?;
        // TODO how can I refactor the let viewpacket_vector_for_style so I could just ust the stmt without the temporary variable?
        let viewpacket_vector_for_style = stmt
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
            })?
            .collect::<Result<Vec<_>, _>>()?;
        viewpacket_vector_for_style
    } else {
        let mut stmt_specific_component_id = db_conn
    .prepare(
        "SELECT component_id, context_model_key, primary_display_key, introduction, sort_order, title, view_style, view_type, viewpacket_id
    FROM view_packet
    WHERE component_id = ?1 AND viewpacket_id != ?2
    ORDER BY sort_order"
    )
    .unwrap();
        let viewpacket_vector_for_component_id = stmt_specific_component_id
            .query_map([search_component_id, exclude_viewpacket_id], |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()?;
        viewpacket_vector_for_component_id
    };

    Ok(viewpacket_vector)
}

/**
 * Get all components that uses the given component id and has the style.
 */
pub fn get_vector_of_usedby_components_by_id_and_key(
    db_conn: &Connection,
    component_id: i32,
    style: String,
) -> Result<Vec<ComponentRelation>> {
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text, relation_description, style FROM component_relation WHERE component_b_id = ?1 AND style = ?2 ORDER BY id"
    )?;
    let component_relations = stmt
        .query_map(rusqlite::params![component_id, style], |row| {
            Ok(ComponentRelation {
                component_a_id: row.get(0)?,
                component_b_id: row.get(1)?,
                connection_type: row.get(2)?,
                id: row.get(3)?,
                key: row.get(4)?,
                property_of_relation: row.get(5)?,
                relation_text: row.get(6)?,
                relation_description: row.get(7)?,
                style: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(component_relations)
}

pub fn get_vector_of_related_components_by_id_and_key(
    db_conn: &Connection,
    component_id: i32,
    primary_display_key: String,
) -> Result<Vec<ComponentRelation>> {
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text, relation_description, style FROM component_relation WHERE component_a_id = ?1 AND key = ?2 ORDER BY id"
    )?;
    let component_relations = stmt
        .query_map(
            rusqlite::params![component_id, primary_display_key],
            |row| {
                Ok(ComponentRelation {
                    component_a_id: row.get(0)?,
                    component_b_id: row.get(1)?,
                    connection_type: row.get(2)?,
                    id: row.get(3)?,
                    key: row.get(4)?,
                    property_of_relation: row.get(5)?,
                    relation_text: row.get(6)?,
                    relation_description: row.get(7)?,
                    style: row.get(8)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(component_relations)
}

// get a list of related components by component id and key, both up and down.
pub fn get_vector_of_related_components_by_id_and_key_both_directions(
    db_conn: &Connection,
    component_id: i32,
    primary_display_key: String,
) -> Result<Vec<ComponentRelation>> {
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text, relation_description, style FROM component_relation WHERE (component_a_id = ?1 OR component_b_id = ?1) AND key = ?2 ORDER BY id"
    )?;
    let component_relations = stmt
        .query_map(
            rusqlite::params![component_id, primary_display_key],
            |row| {
                Ok(ComponentRelation {
                    component_a_id: row.get(0)?,
                    component_b_id: row.get(1)?,
                    connection_type: row.get(2)?,
                    id: row.get(3)?,
                    key: row.get(4)?,
                    property_of_relation: row.get(5)?,
                    relation_text: row.get(6)?,
                    relation_description: row.get(7)?,
                    style: row.get(8)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(component_relations)
}

/// Filter only by key, return vector sorted by sort_order.
pub fn get_vector_of_related_components_by_key(
    db_conn: &Connection,
    key: String,
) -> Result<Vec<ComponentRelation>> {
    // TODO probably remove 'id'
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text, relation_description, style FROM component_relation WHERE key = ?1 ORDER BY sort_order"
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
                relation_description: row.get(7)?,
                style: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(component_relations)
}

pub fn get_component_by_id(db_conn: &Connection, component_id: i32) -> Result<Component> {
    let mut stmt =
        db_conn.prepare("SELECT id, name, purpose, summary FROM component WHERE id = ?1")?;
    let component = stmt.query_row([component_id], |row| {
        Ok(Component {
            id: row.get(0)?,
            name: row.get(1)?,
            purpose: row.get(2)?,
            summary: row.get(3)?,
        })
    })?;
    Ok(component)
}

/**
 * get component_relation.component_a_id where component_relation.component_b_id is component_id
 * get viewpackets where view_packet.component_id is component_relation.component_a_id and view_packet.ViewStyle is style
 */
pub fn get_vector_of_viewpacket_parents_by_component_id_and_style_and_key(
    db_conn: &Connection,
    component_id: i32,
    style: &str,
    primary_display_key: &str,
) -> Result<Vec<ViewPacket>> {
    let mut stmt = db_conn.prepare(
        "SELECT component_id, context_model_key, primary_display_key, introduction, sort_order, title, view_style, view_type, viewpacket_id
    FROM view_packet
    WHERE component_id IN (SELECT component_a_id FROM component_relation WHERE component_b_id = ?1 AND key != ?2) AND view_style = ?3
    ORDER BY sort_order"
    )?;
    let viewpacket_vector_for_style = stmt
        .query_map(rusqlite::params![component_id, primary_display_key, style], |row| {
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
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(viewpacket_vector_for_style)
}

// T E S T I N G

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // Create tables
        conn.execute_batch(
            "
            CREATE TABLE component (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                purpose TEXT,
                summary TEXT
            );
            CREATE TABLE context_model (
                key TEXT,
                entity TEXT,
                entity_type TEXT,
                description TEXT,
                reference TEXT
            );
            CREATE TABLE behavior (
                id INTEGER PRIMARY KEY,
                sort_order INTEGER,
                view_packet_id INTEGER,
                description TEXT,
                diagram_key TEXT
            );
            CREATE TABLE view_packet (
                component_id INTEGER,
                context_model_key TEXT,
                primary_display_key TEXT,
                introduction TEXT,
                sort_order INTEGER,
                title TEXT,
                view_style TEXT,
                view_type TEXT,
                viewpacket_id INTEGER PRIMARY KEY
            );
            CREATE TABLE component_relation (
                component_a_id INTEGER,
                component_b_id INTEGER,
                connection_type TEXT,
                id INTEGER PRIMARY KEY,
                key TEXT,
                property_of_relation TEXT,
                relation_text TEXT,
                relation_description TEXT,
                style TEXT,
                sort_order INTEGER
            );
            ",
        )
        .unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO component (id, name, purpose, summary) VALUES (1, 'ComponentA', 'PurposeA', 'SummaryA')",[],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO context_model (key, entity, entity_type, description, reference) VALUES ('key1', 'Entity1', 'Type1', 'Desc1', 'Ref1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO behavior (id, sort_order, view_packet_id, description, diagram_key) VALUES (1, 1, 10, 'Behavior1', 'Diagram1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO view_packet (component_id, context_model_key, primary_display_key, introduction, sort_order, title, view_style, view_type, viewpacket_id) VALUES (1, 'key1', 'display1', 'Intro1', 1, 'Title1', 'Style1', 'Type1', 10)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO component_relation (component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text, relation_description, style, sort_order) VALUES (1, 2, 'type', 1, 'display1', 'prop', 'rel_text', 'rel_desc', 'Style1', 1)",
            [],
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_get_component_name_by_id() {
        let conn = setup_test_db();
        let name = get_component_name_by_id(&conn, 1);
        assert_eq!(name, "ComponentA");
    }

    #[test]
    fn test_get_vector_of_context_model_by_key() {
        let conn = setup_test_db();
        let result = get_vector_of_context_model_by_key(&conn, "key1").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].entity, "Entity1");
    }

    #[test]
    fn test_get_vector_of_behaviors_for_viewpacket_id() {
        let conn = setup_test_db();
        let result = get_vector_of_behaviors_for_viewpacket_id(&conn, 10).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "Behavior1");
    }

    #[test]
    fn test_get_vector_of_viewpacket_by_component_id_except_component_id_zero() {
        let conn = setup_test_db();
        let result = get_vector_of_viewpacket_by_component_id_except_component_id(
            &conn,
            0,
            "Type1",
            "Style1",
            999,
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Title1");
    }

    #[test]
    fn test_get_vector_of_viewpacket_by_component_id_except_component_id_nonzero() {
        let conn = setup_test_db();
        let result = get_vector_of_viewpacket_by_component_id_except_component_id(
            &conn,
            1,
            "Type1",
            "Style1",
            999,
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Title1");
    }

    #[test]
    fn test_get_vector_of_usedby_components_by_id_and_key() {
        let conn = setup_test_db();
        let result = get_vector_of_usedby_components_by_id_and_key(&conn, 2, "Style1".to_string()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].component_a_id, 1);
    }

    #[test]
    fn test_get_vector_of_related_components_by_id_and_key() {
        let conn = setup_test_db();
        let result = get_vector_of_related_components_by_id_and_key(&conn, 1, "display1".to_string()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].component_b_id, 2);
    }

    #[test]
    fn test_get_vector_of_related_components_by_id_and_key_both_directions() {
        let conn = setup_test_db();
        let result = get_vector_of_related_components_by_id_and_key_both_directions(&conn, 2, "display1".to_string()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].component_a_id, 1);
    }

    #[test]
    fn test_get_vector_of_related_components_by_key() {
        let conn = setup_test_db();
        let result = get_vector_of_related_components_by_key(&conn, "display1".to_string()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].component_a_id, 1);
    }

    #[test]
    fn test_get_component_by_id() {
        let conn = setup_test_db();
        let component = get_component_by_id(&conn, 1).unwrap();
        assert_eq!(component.name, "ComponentA");
    }

    #[test]
    fn test_get_vector_of_viewpacket_parents_by_component_id_and_style_and_key() {
        let conn = setup_test_db();
        // Insert parent component and relation
        conn.execute(
            "INSERT INTO component (id, name, purpose, summary) VALUES (3, 'ParentComponent', 'PurposeP', 'SummaryP')",
            [],
        ).unwrap();
        conn.execute("INSERT INTO component_relation (id, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description, style, sort_order) VALUES (101, 11, 1, 'type', 'Decomp001', 'prop', 'rel_text', 'rel_desc', 'Empty', 2)",[],).unwrap();
        conn.execute("INSERT INTO component_relation (id, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description, style, sort_order) VALUES (102, 12, 1, 'type', 'Decomp002', 'prop', 'rel_text', 'rel_desc', 'Empty', 2)",[],).unwrap();
        conn.execute("INSERT INTO component_relation (id, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description, style, sort_order) VALUES (103, 13, 1, 'type', 'Decomp003', 'prop', 'rel_text', 'rel_desc', 'Empty', 2)",[],).unwrap();

        conn.execute("INSERT INTO component_relation (id, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description, style, sort_order) VALUES (104, 12, 1, 'type', 'Uses04', 'prop', 'rel_text', 'rel_desc', 'Empty', 1)",[],).unwrap();

        conn.execute("INSERT INTO view_packet (viewpacket_id, view_type, view_style, component_id, primary_display_key, context_model_key, introduction, sort_order, title) VALUES (1, 'Module', 'Decomposition', 11, 'Decomp001', 'Ctxt01', 'Intro1', 1, 'Title1')",[],).unwrap();
        conn.execute("INSERT INTO view_packet (viewpacket_id, view_type, view_style, component_id, primary_display_key, context_model_key, introduction, sort_order, title) VALUES (2, 'Module', 'Decomposition', 12, 'Decomp002', 'Ctxt02', 'Intro2', 1, 'Title2')",[],).unwrap();
        conn.execute("INSERT INTO view_packet (viewpacket_id, view_type, view_style, component_id, primary_display_key, context_model_key, introduction, sort_order, title) VALUES (3, 'Module', 'Decomposition', 13, 'Decomp003', 'Ctxt03', 'Intro3', 1, 'Title3')",[],).unwrap();

        conn.execute("INSERT INTO view_packet (viewpacket_id, view_type, view_style, component_id, primary_display_key, context_model_key, introduction, sort_order, title) VALUES (4, 'Module', 'Uses', 12, 'Uses04', 'Ctxt04', 'Intro4', 1, 'Title4')",[],).unwrap();

        let result = get_vector_of_viewpacket_parents_by_component_id_and_style_and_key(
            &conn,
            1,
            "Decomposition",
            "Decomp001",
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "Title2");
        assert_eq!(result[1].title, "Title3");
    }
}
