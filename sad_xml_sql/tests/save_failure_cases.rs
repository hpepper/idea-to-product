//! Tester checks for GEDT-STORY-00002, task T-47 (GEDT-FR-00046).
//!
//! GEDT-FR-00046 asks for every save failure as an error value,
//! naming the project file and the cause.
//! These tests cover failure classes beyond the read-only directory and full disk:
//! a missing parent directory, a directory as the target, and each unreadable table.
//! A panic counts as a failure, because a panic is not an error value.

use rusqlite::Connection;
use sad_xml_sql::db_dump_to_xml::{dump_db_to_xml, SaveError, SaveErrorCause};
use sad_xml_sql::{db_create_in_mem_db, db_populate_from_xml};
use std::fs;
use std::io::ErrorKind;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

fn load_fixture() -> Connection {
    let fixture = format!("{}/tests/test_sad.xml", env!("CARGO_MANIFEST_DIR"));
    let db_conn = Connection::open_in_memory().expect("cannot open in-memory database");
    db_create_in_mem_db(&db_conn);
    db_populate_from_xml(&db_conn, &fixture);
    db_conn
}

/// A fresh directory under the Cargo test temporary directory, removed on drop.
struct WorkDir {
    path: PathBuf,
}

impl WorkDir {
    fn new(test_name: &str) -> Self {
        let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("{test_name}_{}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path)
                .unwrap_or_else(|err| panic!("cannot remove {}: {err}", path.display()));
        }
        fs::create_dir_all(&path)
            .unwrap_or_else(|err| panic!("cannot create {}: {err}", path.display()));
        Self { path }
    }
}

impl Drop for WorkDir {
    /// A second panic while unwinding aborts the test binary and hides the first failure.
    fn drop(&mut self) {
        if let Err(err) = fs::remove_dir_all(&self.path) {
            let message = format!("cannot remove {}: {err}", self.path.display());
            if std::thread::panicking() {
                eprintln!("{message}");
            } else {
                panic!("{message}");
            }
        }
    }
}

/// The message must be exactly `cannot save <file>: <cause>`,
/// so it names the file and the cause and leaks nothing else (GEDT-FR-00046).
fn assert_message_is_file_and_cause(error: &SaveError, output_file: &str, cause: &str) {
    assert_eq!(error.output_file, output_file);
    assert_eq!(
        error.to_string(),
        format!("cannot save {output_file}: {cause}")
    );
}

#[test]
fn t47_fr_00046_missing_parent_directory_returns_error_naming_file_and_cause() {
    let work_dir = WorkDir::new("t47_missing_parent");
    let output_file = work_dir.path.join("no_such_dir").join("saved_sad.xml");
    let output_file = output_file.to_str().expect("non UTF-8 path");

    let error = dump_db_to_xml(&load_fixture(), output_file)
        .expect_err("save into a missing directory succeeded");

    let SaveErrorCause::Io(cause) = &error.cause else {
        panic!("expected an I/O cause, got {:?}", error.cause);
    };
    assert_eq!(
        cause.kind(),
        ErrorKind::NotFound,
        "unexpected cause {cause}"
    );
    assert_message_is_file_and_cause(&error, output_file, &cause.to_string());
}

#[test]
fn t47_fr_00046_directory_as_target_returns_error_and_keeps_directory() {
    let work_dir = WorkDir::new("t47_directory_target");
    let target_dir = work_dir.path.join("saved_sad.xml");
    fs::create_dir(&target_dir).expect("cannot create the target directory");
    fs::write(target_dir.join("kept.txt"), "kept").expect("cannot write the kept file");
    let output_file = target_dir.to_str().expect("non UTF-8 path");

    let error =
        dump_db_to_xml(&load_fixture(), output_file).expect_err("save onto a directory succeeded");

    let SaveErrorCause::Io(cause) = &error.cause else {
        panic!("expected an I/O cause, got {:?}", error.cause);
    };
    assert_eq!(
        cause.kind(),
        ErrorKind::IsADirectory,
        "unexpected cause {cause}"
    );
    assert_message_is_file_and_cause(&error, output_file, &cause.to_string());
    assert_eq!(
        fs::read_to_string(target_dir.join("kept.txt")).unwrap(),
        "kept"
    );
}

/// SQL cannot bind a table name, so each table gets a literal statement.
fn drop_table(db_conn: &Connection, table: &str) {
    let statement = match table {
        "behavior" => "DROP TABLE behavior",
        "component" => "DROP TABLE component",
        "component_relation" => "DROP TABLE component_relation",
        "document" => "DROP TABLE document",
        "include" => "DROP TABLE include",
        "team" => "DROP TABLE team",
        "view_packet" => "DROP TABLE view_packet",
        _ => panic!("no drop statement for table {table}"),
    };
    db_conn
        .execute(statement, [])
        .unwrap_or_else(|err| panic!("cannot drop {table}: {err}"));
}

/// Every table the save reads stands for a database read failure.
/// Each must come back as a database error naming the file, without a panic,
/// and must leave the earlier save untouched.
#[test]
fn t47_fr_00046_each_unreadable_table_returns_database_error_without_panic() {
    let work_dir = WorkDir::new("t47_unreadable_table");
    let output_file = work_dir.path.join("saved_sad.xml");
    let output_file = output_file.to_str().expect("non UTF-8 path");
    let tables = [
        "behavior",
        "component",
        "component_relation",
        "document",
        "include",
        "team",
        "view_packet",
    ];

    let mut failures = Vec::new();
    for table in tables {
        fs::write(output_file, "previous save").expect("cannot write the previous save");
        let db_conn = load_fixture();
        drop_table(&db_conn, table);

        let outcome = catch_unwind(AssertUnwindSafe(|| dump_db_to_xml(&db_conn, output_file)));

        match outcome {
            Err(_) => failures.push(format!("{table}: dump_db_to_xml panicked")),
            Ok(Ok(())) => failures.push(format!("{table}: save succeeded")),
            Ok(Err(error)) => {
                if !matches!(error.cause, SaveErrorCause::Database(_)) {
                    failures.push(format!("{table}: not a database cause: {:?}", error.cause));
                }
                if error.output_file != output_file
                    || !error
                        .to_string()
                        .starts_with(&format!("cannot save {output_file}: "))
                {
                    failures.push(format!("{table}: message lacks the file: {error}"));
                }
            }
        }
        if fs::read_to_string(output_file).unwrap() != "previous save" {
            failures.push(format!("{table}: the earlier save was changed"));
        }
    }
    assert!(
        failures.is_empty(),
        "save failures not returned as errors:\n{}",
        failures.join("\n")
    );
}

/// A component row whose value does not convert to the model type is a database read failure.
/// The save must return it as a database error naming the file, without a panic.
#[test]
fn t47_fr_00046_malformed_component_row_returns_database_error_without_panic() {
    let work_dir = WorkDir::new("t47_malformed_component_row");
    let output_file = work_dir.path.join("saved_sad.xml");
    let output_file = output_file.to_str().expect("non UTF-8 path");
    let corruptions = [
        "UPDATE component SET team_id = 'not-a-number'",
        "UPDATE component SET team_id = -1",
        "UPDATE component SET purpose = NULL",
    ];

    let mut failures = Vec::new();
    for corruption in corruptions {
        let db_conn = load_fixture();
        db_conn
            .execute(corruption, [])
            .unwrap_or_else(|err| panic!("cannot run {corruption}: {err}"));

        let outcome = catch_unwind(AssertUnwindSafe(|| dump_db_to_xml(&db_conn, output_file)));

        match outcome {
            Err(_) => failures.push(format!("{corruption}: dump_db_to_xml panicked")),
            Ok(Ok(())) => failures.push(format!("{corruption}: save succeeded")),
            Ok(Err(error)) => {
                if !matches!(error.cause, SaveErrorCause::Database(_)) {
                    failures.push(format!(
                        "{corruption}: not a database cause: {:?}",
                        error.cause
                    ));
                }
                if !error
                    .to_string()
                    .starts_with(&format!("cannot save {output_file}: "))
                {
                    failures.push(format!("{corruption}: message lacks the file: {error}"));
                }
            }
        }
        if Path::new(output_file).exists() {
            failures.push(format!("{corruption}: a file was written"));
        }
    }
    assert!(
        failures.is_empty(),
        "malformed rows not returned as errors:\n{}",
        failures.join("\n")
    );
}
