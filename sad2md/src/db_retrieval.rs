use crate::models::{Behavior, Component, ComponentRelation, ViewPacket};

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

// get a list of related components by component id and key
pub fn get_vector_of_related_components_by_id_and_key(
    db_conn: &Connection,
    component_id: i32,
    primary_display_key: String,
) -> Result<Vec<ComponentRelation>> {
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text FROM component_relation WHERE component_a_id = ?1 AND key = ?2 ORDER BY id"
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

pub fn get_component_by_id(db_conn: &Connection, component_id: i32) -> Result<Component> {
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
