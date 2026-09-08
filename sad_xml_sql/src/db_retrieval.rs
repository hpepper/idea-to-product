use crate::models::{Behavior, Component, ComponentRelation, ContextModel, ViewPacket};

use rusqlite::{Connection, Result};

pub fn get_component_by_id(db_conn: &Connection, component_id: u64) -> Result<Component> {
    let mut stmt = db_conn.prepare(
        "SELECT file_id, id, name, purpose, summary, team_id FROM component WHERE id = ?1",
    )?;
    let component = stmt.query_row([component_id], |row| {
        Ok(Component {
            file_id: row.get(0)?,
            id: row.get(1)?,
            name: row.get(2)?,
            purpose: row.get(3)?,
            summary: row.get(4)?,
            team_id: row.get(5)?,
        })
    })?;
    Ok(component)
}

pub fn get_component_by_name(db_conn: &Connection, name: String) -> Result<Component> {
    let mut stmt = db_conn.prepare(
        "SELECT file_id,id, name, purpose, summary, team_id FROM component WHERE name = ?1",
    )?;
    let component = stmt.query_row([name], |row| {
        Ok(Component {
            file_id: row.get(0)?,
            id: row.get(1)?,
            name: row.get(2)?,
            purpose: row.get(3)?,
            summary: row.get(4)?,
            team_id: row.get(5)?,
        })
    })?;
    Ok(component)
}

pub fn get_component_name_by_id(db_conn: &Connection, component_id: u64) -> String {
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
    let component_names = stmt.query_map([], |row| Ok(row.get(0)?)).unwrap();
    component_names.collect::<Result<Vec<String>, _>>()
}

pub fn get_vector_of_components_sorted_by_name(db_conn: &Connection) -> Result<Vec<Component>> {
    //println!("Preparing statement to fetch components sorted by name");
    let mut stmt = db_conn
        .prepare("SELECT file_id, id, name, purpose, summary, team_id FROM component ORDER BY name COLLATE NOCASE ASC")
        .unwrap();
    //println!("Statement prepared, querying components...");
    let component_names = stmt
        .query_map([], |row| {
            //println!("Fetched component: id={}, name={}", id, name);
            Ok(Component {
                file_id: row.get(0)?,
                id: row.get(1)?,
                name: row.get(2)?,
                purpose: row.get(3)?,
                summary: row.get(4)?,
                team_id: row.get(5)?,
            })
        })
        .unwrap();
    let result = component_names.collect::<Result<Vec<Component>, _>>()?;
    //println!("Total components fetched: {}", result.len());
    Ok(result)
}

/// get vector of components referenrenced in component relations by a key and sorted by component name,
pub fn get_vector_of_components_by_key_sorted_by_name(
    db_conn: &Connection,
    key: &str,
) -> Result<Vec<Component>> {
    let mut stmt = db_conn.prepare(
        "SELECT DISTINCT c.file_id, c.id, c.name, c.purpose, c.summary, c.team_id
         FROM component c
         WHERE c.id IN (
             SELECT component_a_id FROM component_relation WHERE key = ?1
             UNION
             SELECT component_b_id FROM component_relation WHERE key = ?1
         )
         ORDER BY c.name COLLATE NOCASE ASC",
    )?;
    let components = stmt
        .query_map([key], |row| {
            Ok(Component {
                file_id: row.get(0)?,
                id: row.get(1)?,
                name: row.get(2)?,
                purpose: row.get(3)?,
                summary: row.get(4)?,
                team_id: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(components)
}



pub fn get_components_vector_by_team_id_sorted_by_name(
    db_conn: &Connection,
    requested_team_id: u64,
) -> Result<Vec<Component>> {
    let mut stmt = db_conn
        .prepare("SELECT file_id, id, name, purpose, summary, team_id FROM component WHERE team_id = ?1 ORDER BY name COLLATE NOCASE ASC")
        .unwrap();
    let component_names = stmt
        .query_map([requested_team_id], |row| {
            Ok(Component {
                file_id: row.get(0)?,
                id: row.get(1)?,
                name: row.get(2)?,
                purpose: row.get(3)?,
                summary: row.get(4)?,
                team_id: row.get(5)?,
            })
        })
        .unwrap();
    component_names.collect::<Result<Vec<Component>, _>>()
}

// TODO  first read the information from the component_relation table and write into the context_model.
pub fn get_vector_of_context_model_by_key(
    db_conn: &Connection,
    key: &str,
) -> Result<Vec<ContextModel>> {
    let mut stmt = db_conn.prepare(
        "SELECT component_relation.component_b_id, component_relation.key, component_relation.relation_text, component_relation.relation_description, component.name, component.summary FROM component_relation JOIN component WHERE component.id = component_relation.component_b_id AND component_relation.key = ?1"
    )?;
    let unsorted_context_list = stmt
        .query_map(rusqlite::params![key], |row| {
            Ok((
                ContextModel {
                    entity: row.get(4)?,
                    entity_type: "adjacent".to_string(),
                    description: row.get(5)?,
                    reference: "".to_string(), // TODO populate when component has the reference field.
                },
                ContextModel {
                    entity: row.get(2)?,
                    entity_type: "connection".to_string(),
                    description: row.get(3)?,
                    reference: "".to_string(), // populate if I ever find a reference need.
                },
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|(a, b)| vec![a, b]) // flatten tuple into Vec<ContextModel>
        .collect::<Vec<_>>();

    let mut stmt = db_conn.prepare(
        "INSERT INTO context_model (key, entity, entity_type, description, reference) 
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;

    for context_entry in &unsorted_context_list {
        if !context_entry.entity.is_ascii() {
            stmt.execute(rusqlite::params![
                key, // the same key you queried with
                context_entry.entity,
                context_entry.entity_type,
                context_entry.description,
                context_entry.reference,
            ])?;
        }
    }

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
    viewpacket_id: u64,
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

pub fn get_vector_of_behaviors_sorted_by_key_and_order(
    db_conn: &Connection,
) -> Result<Vec<Behavior>> {
    let mut stmt = db_conn.prepare(
        "SELECT id, description, diagram_key, sort_order, view_packet_id FROM behavior  ORDER BY diagram_key, sort_order"
    )?;
    let behavior = stmt
        .query_map(rusqlite::params![], |row| {
            Ok(Behavior {
                id: row.get(0)?,
                description: row.get(1)?,
                diagram_key: row.get(2)?,
                sort_order: row.get(3)?,
                view_packet_id: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(behavior)
}

/// Get all component relations originating from `component_id` (as `component_a_id`) matching `primary_display_key`.
///
/// # Arguments
/// * `db_conn` - open database connection
/// * `component_id` - id of the component acting as `component_a_id`
/// * `primary_display_key` - relation key to filter on
///
/// # Returns
/// Component relations sorted by `sort_order`.
pub fn get_vector_of_component_relations_by_id_and_key(
    db_conn: &Connection,
    component_id: u64,
    primary_display_key: String,
) -> Result<Vec<ComponentRelation>> {
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text, relation_description, sort_order, style FROM component_relation WHERE component_a_id = ?1 AND key = ?2 ORDER BY sort_order"
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
                    sort_order: row.get(8)?,
                    style: row.get(9)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(component_relations)
}

// get a list of related components by component id and key, both up and down.
pub fn get_vector_of_component_relations_by_id_and_key_both_directions(
    db_conn: &Connection,
    component_id: u64,
    primary_display_key: String,
) -> Result<Vec<ComponentRelation>> {
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text, relation_description, sort_order, style FROM component_relation WHERE (component_a_id = ?1 OR component_b_id = ?1) AND key = ?2 ORDER BY sort_order"
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
                    sort_order: row.get(8)?,
                    style: row.get(9)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(component_relations)
}

/// Filter only by key, return vector sorted by sort_order.
///
/// # Arguments
/// * `db_conn` - open database connection
/// * `primary_display_key` - relation key to filter on
///
/// # Returns
/// Component relations sorted by `sort_order`.
pub fn get_vector_of_component_relations_by_key(
    db_conn: &Connection,
    key: String,
) -> Result<Vec<ComponentRelation>> {
    // TODO probably remove 'id'
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text, relation_description, sort_order, style FROM component_relation WHERE key = ?1 ORDER BY sort_order"
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
                sort_order: row.get(8)?,
                style: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(component_relations)
}

pub fn get_vector_of_component_relations_sorted_by_key_and_order(
    db_conn: &Connection,
) -> Result<Vec<ComponentRelation>> {
    // TODO probably remove 'id'
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text, relation_description, sort_order, style FROM component_relation ORDER BY key,sort_order"
    )?;
    let component_relations = stmt
        .query_map(rusqlite::params![], |row| {
            Ok(ComponentRelation {
                component_a_id: row.get(0)?,
                component_b_id: row.get(1)?,
                connection_type: row.get(2)?,
                id: row.get(3)?,
                key: row.get(4)?,
                property_of_relation: row.get(5)?,
                relation_text: row.get(6)?,
                relation_description: row.get(7)?,
                sort_order: row.get(8)?,
                style: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(component_relations)
}

// TODO should this also filter on file_id?
pub fn get_vector_of_viewpacket_titles_sorted(db_conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = db_conn
        .prepare("SELECT title FROM view_packet ORDER BY title COLLATE NOCASE ASC")
        .unwrap();
    let viewpacket_titles = stmt.query_map([], |row| Ok(row.get(0)?)).unwrap();
    viewpacket_titles.collect::<Result<Vec<String>, _>>()
}

pub fn get_local_file_id(db_conn: &Connection) -> Result<u64> {
    // TODO get rid of limit
    let mut stmt = db_conn.prepare("SELECT file_id FROM document LIMIT 1")?;
    let file_id: u64 = stmt.query_row([], |row| row.get(0))?;
    Ok(file_id)
}

pub fn get_include_url_by_file_id(db_conn: &Connection, file_id: u64) -> Result<String> {
    let mut stmt = db_conn.prepare("SELECT url FROM include WHERE file_id = ?1")?;
    let url: String = stmt.query_row([file_id], |row| row.get(0))?;
    Ok(url)
}

// TODO at some point refactor this and get_vector_of_any_viewpacket_by_component_id_excluding_viewpacket_id
pub fn get_vector_of_local_viewpacket_by_component_id_excluding_viewpacket_id(
    db_conn: &Connection,
    search_component_id: u64,
    view_type: &str,
    style: &str,
    exclude_viewpacket_id: u64,
) -> Result<Vec<ViewPacket>> {
    let file_id = get_local_file_id(db_conn)?;

    let viewpacket_vector = if search_component_id == 0 {
        let mut stmt = db_conn.prepare("SELECT component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id FROM view_packet  WHERE view_type = ?1 AND view_style = ?2 AND file_id = ?3 ORDER BY sort_order")?;
        // TODO how can I refactor the let viewpacket_vector_for_style so I could just ust the stmt without the temporary variable?
        let viewpacket_vector_for_style = stmt
            .query_map((view_type, style, file_id), |row| {
                Ok(ViewPacket {
                    component_id: row.get(0)?,
                    context_model_key: row.get(1)?,
                    primary_display_key: row.get(2)?,
                    file_id: row.get(3)?,
                    introduction: row.get(4)?,
                    sort_order: row.get(5)?,
                    team_id: row.get(6)?,
                    title: row.get(7)?,
                    view_style: row.get(8)?,
                    view_type: row.get(9)?,
                    viewpacket_id: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        viewpacket_vector_for_style
    } else {
        let mut stmt_specific_component_id = db_conn
    .prepare(
        "SELECT component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id
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
                    file_id: row.get(3)?,
                    introduction: row.get(4)?,
                    sort_order: row.get(5)?,
                    team_id: row.get(6)?,
                    title: row.get(7)?,
                    view_style: row.get(8)?,
                    view_type: row.get(9)?,
                    viewpacket_id: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        viewpacket_vector_for_component_id
    };

    Ok(viewpacket_vector)
}

/**
 * if search_component_id == 0, then return all view packets for the given view_type and style
 * if search_component_id != 0, then return all view packets for the given component_id except for the given viewpacket_id
 */
pub fn get_vector_of_any_viewpacket_by_component_id_excluding_viewpacket_id(
    db_conn: &Connection,
    search_component_id: u64,
    view_type: &str,
    style: &str,
    exclude_viewpacket_id: u64,
) -> Result<Vec<ViewPacket>> {
    let viewpacket_vector = if search_component_id == 0 {
        let mut stmt = db_conn.prepare("SELECT component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id FROM view_packet  WHERE view_type = ?1 AND view_style = ?2 ORDER BY sort_order")?;
        // TODO how can I refactor the let viewpacket_vector_for_style so I could just ust the stmt without the temporary variable?
        let viewpacket_vector_for_style = stmt
            .query_map([view_type, style], |row| {
                Ok(ViewPacket {
                    component_id: row.get(0)?,
                    context_model_key: row.get(1)?,
                    primary_display_key: row.get(2)?,
                    file_id: row.get(3)?,
                    introduction: row.get(4)?,
                    sort_order: row.get(5)?,
                    team_id: row.get(6)?,
                    title: row.get(7)?,
                    view_style: row.get(8)?,
                    view_type: row.get(9)?,
                    viewpacket_id: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        viewpacket_vector_for_style
    } else {
        let mut stmt_specific_component_id = db_conn
    .prepare(
        "SELECT component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id
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
                    file_id: row.get(3)?,
                    introduction: row.get(4)?,
                    sort_order: row.get(5)?,
                    team_id: row.get(6)?,
                    title: row.get(7)?,
                    view_style: row.get(8)?,
                    view_type: row.get(9)?,
                    viewpacket_id: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        viewpacket_vector_for_component_id
    };

    Ok(viewpacket_vector)
}

// TODO only get this for the local file_id
pub fn get_viewpacket_by_team_id(db_conn: &Connection, search_team_id: u64) -> Option<ViewPacket> {
    if search_team_id == 0 {
        return None;
    }
    let mut stmt = db_conn.prepare("SELECT component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id FROM view_packet  WHERE team_id = ?1 ORDER BY sort_order").unwrap();
    let viewpacket = stmt.query_row([search_team_id], |row| {
        Ok(ViewPacket {
            component_id: row.get(0)?,
            context_model_key: row.get(1)?,
            primary_display_key: row.get(2)?,
            file_id: row.get(3)?,
            introduction: row.get(4)?,
            sort_order: row.get(5)?,
            team_id: row.get(6)?,
            title: row.get(7)?,
            view_style: row.get(8)?,
            view_type: row.get(9)?,
            viewpacket_id: row.get(10)?,
        })
    });

    if let Ok(viewpacket) = viewpacket {
        Some(viewpacket)
    } else {
        None
    }
}

pub fn get_viewpacket_by_title(db_conn: &Connection, title: &str) -> Result<ViewPacket> {
    let mut stmt = db_conn.prepare(        "SELECT component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id FROM view_packet  WHERE title = ?1")?;
    let viewpacket = stmt.query_row([title], |row| {
        Ok(ViewPacket {
            component_id: row.get(0)?,
            context_model_key: row.get(1)?,
            primary_display_key: row.get(2)?,
            file_id: row.get(3)?,
            introduction: row.get(4)?,
            sort_order: row.get(5)?,
            team_id: row.get(6)?,
            title: row.get(7)?,
            view_style: row.get(8)?,
            view_type: row.get(9)?,
            viewpacket_id: row.get(10)?,
        })
    })?;

    Ok(viewpacket)
}

/// Get the view packet for the given style and id, or None.
pub fn get_viewpacket_by_style_and_component_id(
    db_conn: &Connection,
    style: &str,
    search_component_id: u64,
) -> Option<ViewPacket> {
    let mut stmt = db_conn.prepare(
        "SELECT component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id FROM view_packet WHERE view_style = ?1 AND component_id = ?2",
    ).unwrap();
    let viewpacket = stmt.query_row((&style, search_component_id), |row| {
        Ok(ViewPacket {
            component_id: row.get(0)?,
            context_model_key: row.get(1)?,
            primary_display_key: row.get(2)?,
            file_id: row.get(3)?,
            introduction: row.get(4)?,
            sort_order: row.get(5)?,
            team_id: row.get(6)?,
            title: row.get(7)?,
            view_style: row.get(8)?,
            view_type: row.get(9)?,
            viewpacket_id: row.get(10)?,
        })
    });

    if let Ok(viewpacket) = viewpacket {
        Some(viewpacket)
    } else {
        None
    }
}

pub fn get_viewpacket_vector_by_type_and_style_sorted_by_order(
    db_conn: &Connection,
    filter_view_type: &str,
    filter_view_style: &str,
) -> Result<Vec<ViewPacket>> {
    let mut stmt = db_conn.prepare(        "SELECT component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id FROM view_packet  WHERE view_type = ?1 AND view_style = ?2")?;
    let viewpacket_vector = stmt
        .query_map([filter_view_type, filter_view_style], |row| {
            Ok(ViewPacket {
                component_id: row.get(0)?,
                context_model_key: row.get(1)?,
                primary_display_key: row.get(2)?,
                file_id: row.get(3)?,
                introduction: row.get(4)?,
                sort_order: row.get(5)?,
                team_id: row.get(6)?,
                title: row.get(7)?,
                view_style: row.get(8)?,
                view_type: row.get(9)?,
                viewpacket_id: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(viewpacket_vector)
}

/**
 * Get all components that uses the given component id and has the style.
 */
pub fn get_vector_of_usedby_components_by_id_and_key(
    db_conn: &Connection,
    component_id: u64,
    style: String,
) -> Result<Vec<ComponentRelation>> {
    let mut stmt = db_conn.prepare(
        "SELECT component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text, relation_description, sort_order, style FROM component_relation WHERE component_b_id = ?1 AND style = ?2 ORDER BY id"
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
                sort_order: row.get(8)?,
                style: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(component_relations)
}

/**
 * get component_relation.component_a_id where component_relation.component_b_id is component_id
 * get viewpackets where view_packet.component_id is component_relation.component_a_id and view_packet.ViewStyle is style
 */
pub fn get_vector_of_viewpacket_parents_by_component_id_and_style_and_key(
    db_conn: &Connection,
    component_id: u64,
    style: &str,
    primary_display_key: &str,
) -> Result<Vec<ViewPacket>> {
    let mut stmt = db_conn.prepare(
        "SELECT component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id
    FROM view_packet
    WHERE component_id IN (SELECT component_a_id FROM component_relation WHERE component_b_id = ?1 AND key != ?2) AND view_style = ?3
    ORDER BY sort_order"
    )?;
    let viewpacket_vector_for_style = stmt
        .query_map(
            rusqlite::params![component_id, primary_display_key, style],
            |row| {
                Ok(ViewPacket {
                    component_id: row.get(0)?,
                    context_model_key: row.get(1)?,
                    primary_display_key: row.get(2)?,
                    file_id: row.get(3)?,
                    introduction: row.get(4)?,
                    sort_order: row.get(5)?,
                    team_id: row.get(6)?,
                    title: row.get(7)?,
                    view_style: row.get(8)?,
                    view_type: row.get(9)?,
                    viewpacket_id: row.get(10)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(viewpacket_vector_for_style)
}

// T E S T I N G

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_create_in_mem_db;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db_create_in_mem_db(&conn);

        let team_id = 1;
        let file_id: u64 = 8;

        // Insert test data
        conn.execute(
            "INSERT INTO component (file_id, id, name, purpose, summary, team_id) VALUES (?1, 1, 'ComponentA', 'PurposeA', 'SummaryA', 0)",[file_id],
        ).unwrap();
        conn.execute(
            "INSERT INTO component (file_id, id, name, purpose, summary, team_id) VALUES (?1, 2, 'ComponentB', 'PurposeB', 'SummaryB', ?2)",[file_id, team_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO component (file_id, id, name, purpose, summary, team_id) VALUES (?1, 3, 'ComponentC', 'PurposeC', 'SummaryC', ?2)",[file_id, team_id],
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
            "INSERT INTO view_packet (component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id) VALUES (1, 'key1', 'display1', ?1, 'Intro1', 1, 0, 'Title1', 'Decomposition', 'Type1', 10)",
            [file_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO view_packet (component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id) VALUES (0, 'key1', 'display1', ?1, 'Intro1', 1, ?2, 'Team Assignment', 'Assignment', 'Type1', 11)",
            [file_id, team_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO view_packet (component_id, context_model_key, primary_display_key, file_id, introduction, sort_order, team_id, title, view_style, view_type, viewpacket_id) VALUES (0, 'key1', 'display1', ?1, 'Intro1', 1, ?2, 'empty Team Assignment', 'Assignment', 'Type1', 12)",
            [file_id, team_id],
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
    fn test_get_components_vector_by_team_id_sorted_by_name() {
        let conn = setup_test_db();
        let result = get_components_vector_by_team_id_sorted_by_name(&conn, 1).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "ComponentB");
        assert_eq!(result[1].name, "ComponentC");
    }
    #[test]
    fn test_get_vector_of_context_model_by_key() {
        let conn = setup_test_db();
        let result = get_vector_of_context_model_by_key(&conn, "key1").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].entity, "Entity1");
    }

    #[test]
    fn test_get_vector_of_components_by_key_sorted_by_name() {
        let conn = setup_test_db();
        // ComponentC only ever appears as component_a_id for this key, ComponentB only as component_b_id.
        conn.execute(
            "INSERT INTO component_relation (component_a_id, component_b_id, connection_type, id, key, property_of_relation, relation_text, relation_description, style, sort_order) VALUES (3, 2, 'type', 2, 'deploy1', 'prop', 'rel_text', 'rel_desc', 'Style1', 1)",
            [],
        )
        .unwrap();

        let result = get_vector_of_components_by_key_sorted_by_name(&conn, "deploy1").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "ComponentB");
        assert_eq!(result[1].name, "ComponentC");
    }

    #[test]
    fn test_get_vector_of_behaviors_for_viewpacket_id() {
        let conn = setup_test_db();
        let result = get_vector_of_behaviors_for_viewpacket_id(&conn, 10).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "Behavior1");
    }

    #[test]
    fn test_get_vector_of_any_viewpacket_by_component_id_excluding_viewpacket_id_zero() {
        let conn = setup_test_db();
        let result = get_vector_of_any_viewpacket_by_component_id_excluding_viewpacket_id(
            &conn,
            0,
            "Type1",
            "Decomposition",
            999,
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Title1");
    }

    #[test]
    fn test_get_vector_of_any_viewpacket_by_component_id_excluding_viewpacket_id_nonzero() {
        let conn = setup_test_db();
        let result = get_vector_of_any_viewpacket_by_component_id_excluding_viewpacket_id(
            &conn, 1, "Type1", "Style1", 999,
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Title1");
    }

    #[test]
    fn test_get_vector_of_usedby_components_by_id_and_key() {
        let conn = setup_test_db();
        let result =
            get_vector_of_usedby_components_by_id_and_key(&conn, 2, "Style1".to_string()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].component_a_id, 1);
    }

    #[test]
    fn test_get_vector_of_component_relations_by_id_and_key() {
        let conn = setup_test_db();
        let result =
            get_vector_of_component_relations_by_id_and_key(&conn, 1, "display1".to_string())
                .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].component_b_id, 2);
    }

    #[test]
    fn test_get_vector_of_component_relations_by_id_and_key_both_directions() {
        let conn = setup_test_db();
        let result = get_vector_of_component_relations_by_id_and_key_both_directions(
            &conn,
            2,
            "display1".to_string(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].component_a_id, 1);
    }

    #[test]
    fn test_get_vector_of_component_relations_by_key() {
        let conn = setup_test_db();
        let result =
            get_vector_of_component_relations_by_key(&conn, "display1".to_string()).unwrap();
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
            "INSERT INTO component (id, name, purpose, summary) VALUES (203, 'ParentComponent', 'PurposeP', 'SummaryP')",
            [],
        ).unwrap();
        conn.execute("INSERT INTO component_relation (id, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description, style, sort_order) VALUES (101, 11, 1, 'type', 'Decomp001', 'prop', 'rel_text', 'rel_desc', 'Empty', 2)",[],).unwrap();
        conn.execute("INSERT INTO component_relation (id, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description, style, sort_order) VALUES (102, 12, 1, 'type', 'Decomp002', 'prop', 'rel_text', 'rel_desc', 'Empty', 2)",[],).unwrap();
        conn.execute("INSERT INTO component_relation (id, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description, style, sort_order) VALUES (103, 13, 1, 'type', 'Decomp003', 'prop', 'rel_text', 'rel_desc', 'Empty', 2)",[],).unwrap();

        conn.execute("INSERT INTO component_relation (id, component_a_id, component_b_id, connection_type, key, property_of_relation, relation_text, relation_description, style, sort_order) VALUES (104, 12, 1, 'type', 'Uses04', 'prop', 'rel_text', 'rel_desc', 'Empty', 1)",[],).unwrap();

        conn.execute("INSERT INTO view_packet (viewpacket_id, view_type, view_style, component_id, primary_display_key, context_model_key, file_id, introduction, sort_order, team_id, title) VALUES (1, 'Module', 'Decomposition', 11, 'Decomp001', 'Ctxt01', 0, 'Intro1', 1, 0, 'Title1')",[],).unwrap();
        conn.execute("INSERT INTO view_packet (viewpacket_id, view_type, view_style, component_id, primary_display_key, context_model_key, file_id, introduction, sort_order, team_id, title) VALUES (2, 'Module', 'Decomposition', 12, 'Decomp002', 'Ctxt02', 0, 'Intro2', 1, 0, 'Title2')",[],).unwrap();
        conn.execute("INSERT INTO view_packet (viewpacket_id, view_type, view_style, component_id, primary_display_key, context_model_key, file_id, introduction, sort_order, team_id, title) VALUES (3, 'Module', 'Decomposition', 13, 'Decomp003', 'Ctxt03', 0, 'Intro3', 1, 0, 'Title3')",[],).unwrap();

        conn.execute("INSERT INTO view_packet (viewpacket_id, view_type, view_style, component_id, primary_display_key, context_model_key, file_id, introduction, sort_order, team_id, title) VALUES (4, 'Module', 'Uses', 12, 'Uses04', 'Ctxt04', 0, 'Intro4', 1, 0, 'Title4')",[],).unwrap();

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

    #[test]
    fn test_get_viewpacket_by_team_id() {
        let conn = setup_test_db();
        let result = get_viewpacket_by_team_id(&conn, 1);
        assert!(result.is_some());
        let viewpacket = result.unwrap();
        assert_eq!(viewpacket.title, "Team Assignment");
    }

    #[test]
    fn test_get_viewpacket_by_zero_team_id() {
        let conn = setup_test_db();
        let result = get_viewpacket_by_team_id(&conn, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_viewpacket_by_team_id_non_existing_id() {
        let conn = setup_test_db();
        let result = get_viewpacket_by_team_id(&conn, 999);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_viewpacket_by_style_and_component_id() {
        let conn = setup_test_db();
        let result = get_viewpacket_by_style_and_component_id(&conn, "Decomposition", 1);
        assert!(result.is_some());
        let viewpacket = result.unwrap();
        assert_eq!(viewpacket.title, "Title1");
    }
    #[test]
    fn test_get_viewpacket_by_style_and_component_id_non_existing() {
        let conn = setup_test_db();
        let result = get_viewpacket_by_style_and_component_id(&conn, "Decomposition", 999);
        assert!(result.is_none());
    }
}
