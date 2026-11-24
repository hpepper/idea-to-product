//! Tester checks for GEDT-STORY-00019, task T-41: unsigned IDs after the `rusqlite` 0.41 upgrade.
//!
//! `rusqlite` 0.41 converts `u64` to and from SQL only with its `fallible_uint` feature.
//! SQLite stores a signed 64-bit integer, so an ID above `i64::MAX` must fail to bind,
//! as in `rusqlite` 0.35, and must never wrap into a negative stored ID.
//! A negative stored ID must fail to read as `u64`, and must never wrap into a large ID.
//! The IDs stay bound parameters (GEDT-AR-00016); only the parameter values vary here.

use rusqlite::Connection;
use sad_xml_sql::{
    db_create_in_mem_db, get_component_by_name, insert_into_component, update_component_by_id,
    Component,
};
use std::panic::{catch_unwind, AssertUnwindSafe};

/// The smallest ID that SQLite cannot store as a signed 64-bit integer.
const FIRST_UNSTORABLE_ID: u64 = i64::MAX as u64 + 1;

fn empty_db() -> Connection {
    let db_conn = Connection::open_in_memory().expect("cannot open in-memory database");
    db_create_in_mem_db(&db_conn);
    db_conn
}

fn component_count(db_conn: &Connection) -> i64 {
    db_conn
        .query_row("SELECT COUNT(*) FROM component", [], |row| row.get(0))
        .expect("cannot count components")
}

fn insert_component(db_conn: &Connection, id: u64, name: &str) {
    insert_into_component(db_conn, 1, id, name, "purpose", "summary", 0, "title");
}

/// `insert_into_component` panics on a failed insert, by its documented contract.
fn insert_panics(db_conn: &Connection, id: u64) -> bool {
    catch_unwind(AssertUnwindSafe(|| insert_component(db_conn, id, "Unstorable"))).is_err()
}

#[test]
fn t41_ar_00016_largest_storable_id_round_trips() {
    let db_conn = empty_db();
    let largest = i64::MAX as u64;

    insert_component(&db_conn, largest, "Largest");

    let stored: i64 = db_conn
        .query_row("SELECT id FROM component WHERE name = 'Largest'", [], |row| row.get(0))
        .expect("inserted component missing");
    assert_eq!(stored, i64::MAX);
    let read_back = get_component_by_name(&db_conn, "Largest".to_string()).expect("read back");
    assert_eq!(read_back.id, largest);
}

#[test]
fn t41_ar_00016_id_above_i64_max_is_rejected_on_insert() {
    for id in [FIRST_UNSTORABLE_ID, u64::MAX] {
        let db_conn = empty_db();

        assert!(insert_panics(&db_conn, id), "insert of ID {id} did not fail");
        assert_eq!(component_count(&db_conn), 0, "insert of ID {id} stored a row");
    }
}

/// A wrapping conversion would turn `FIRST_UNSTORABLE_ID` into `i64::MIN`
/// and update the row stored under that ID.
#[test]
fn t41_ar_00016_id_above_i64_max_does_not_update_wrapped_row() {
    let db_conn = empty_db();
    db_conn
        .execute(
            "INSERT INTO component (file_id, id, name, purpose, summary, team_id, title)
            VALUES (1, ?1, 'Wrapped', 'original purpose', 'original summary', 0, 'original title')",
            [i64::MIN],
        )
        .expect("cannot insert the wrapped-ID row");
    let edit = Component {
        file_id: 1,
        id: FIRST_UNSTORABLE_ID,
        name: "Wrapped".to_string(),
        purpose: "changed purpose".to_string(),
        summary: "changed summary".to_string(),
        team_id: 0,
        title: "changed title".to_string(),
    };

    let update = catch_unwind(AssertUnwindSafe(|| update_component_by_id(&db_conn, &edit)));

    assert!(update.is_err(), "update with ID {FIRST_UNSTORABLE_ID} did not fail");
    let purpose: String = db_conn
        .query_row("SELECT purpose FROM component WHERE id = ?1", [i64::MIN], |row| row.get(0))
        .expect("wrapped-ID row missing");
    assert_eq!(purpose, "original purpose");
}

#[test]
fn t41_ar_00016_negative_stored_id_is_not_read_as_u64() {
    for stored_id in [-1_i64, i64::MIN] {
        let db_conn = empty_db();
        db_conn
            .execute(
                "INSERT INTO component (file_id, id, name, purpose, summary, team_id, title)
                VALUES (1, ?1, 'Negative', '', '', 0, '')",
                [stored_id],
            )
            .expect("cannot insert the negative-ID row");

        let read_back = get_component_by_name(&db_conn, "Negative".to_string());

        assert!(
            read_back.is_err(),
            "stored ID {stored_id} was read as {:?}",
            read_back.map(|component| component.id)
        );
    }
}
