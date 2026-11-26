//! Save failure tests for `dump_db_to_xml` (GEDT-STORY-00002, task T-47, GEDT-FR-00046).
//!
//! Each failed save must return an error value that names the output file and the cause,
//! instead of panicking or printing to the terminal.
//! The fixtures need no terminal and no root: a directory that refuses new files,
//! and the Linux device `/dev/full`, which fails every write with "no space left".

use rusqlite::Connection;
use sad_xml_sql::db_dump_to_xml::{dump_db_to_xml, SaveErrorCause};
use sad_xml_sql::{db_create_in_mem_db, db_populate_from_xml};
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// A Linux character device that fails every write with `ENOSPC`, like a full disk.
const FULL_DISK_FILE: &str = "/dev/full";

fn load_fixture() -> Connection {
    let fixture = format!("{}/tests/test_sad.xml", env!("CARGO_MANIFEST_DIR"));
    let db_conn = Connection::open_in_memory().expect("cannot open in-memory database");
    db_create_in_mem_db(&db_conn);
    db_populate_from_xml(&db_conn, &fixture);
    db_conn
}

/// Returns whether `dir` refuses a new file, and removes the probe file when it does not.
fn refuses_new_files(dir: &Path) -> bool {
    let probe = dir.join(".t47_write_probe");
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
    {
        Ok(_) => {
            fs::remove_file(&probe)
                .unwrap_or_else(|err| panic!("cannot remove {}: {err}", probe.display()));
            false
        }
        Err(err)
            if matches!(
                err.kind(),
                ErrorKind::PermissionDenied | ErrorKind::ReadOnlyFilesystem
            ) =>
        {
            true
        }
        Err(err) => panic!("cannot probe {}: {err}", dir.display()),
    }
}

/// A directory that refuses new files, for the read-only directory fixture.
/// Mode 0o555 stops an unprivileged user.
/// The tool image runs the tests as root of a rootless user namespace,
/// whose `CAP_DAC_OVERRIDE` ignores the mode,
/// so the crate root, which the tool image mounts read-only, stands in there.
/// Drop restores the mode, so the work directory can be removed.
struct ReadOnlyDir {
    path: PathBuf,
    chmod_dir: PathBuf,
}

impl ReadOnlyDir {
    fn new(test_name: &str) -> Self {
        let chmod_dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("{test_name}_{}", std::process::id()));
        if chmod_dir.exists() {
            fs::set_permissions(&chmod_dir, fs::Permissions::from_mode(0o755))
                .unwrap_or_else(|err| panic!("cannot chmod {}: {err}", chmod_dir.display()));
            fs::remove_dir_all(&chmod_dir)
                .unwrap_or_else(|err| panic!("cannot remove {}: {err}", chmod_dir.display()));
        }
        fs::create_dir(&chmod_dir)
            .unwrap_or_else(|err| panic!("cannot create {}: {err}", chmod_dir.display()));
        fs::set_permissions(&chmod_dir, fs::Permissions::from_mode(0o555))
            .unwrap_or_else(|err| panic!("cannot chmod {}: {err}", chmod_dir.display()));
        let read_only_mount = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = [chmod_dir.clone(), read_only_mount]
            .into_iter()
            .find(|candidate| refuses_new_files(candidate))
            .unwrap_or_else(|| {
                panic!("no read-only directory: run the tests unprivileged or in the tool image")
            });
        Self { path, chmod_dir }
    }
}

impl Drop for ReadOnlyDir {
    /// A second panic while unwinding aborts the test binary and hides the first failure.
    fn drop(&mut self) {
        let cleanup = fs::set_permissions(&self.chmod_dir, fs::Permissions::from_mode(0o755))
            .and_then(|()| fs::remove_dir_all(&self.chmod_dir));
        if let Err(err) = cleanup {
            let message = format!("cannot remove {}: {err}", self.chmod_dir.display());
            if std::thread::panicking() {
                eprintln!("{message}");
            } else {
                panic!("{message}");
            }
        }
    }
}

#[test]
fn t47_fr_00046_read_only_directory_returns_error_naming_file_and_cause() {
    let read_only_dir = ReadOnlyDir::new("t47_read_only");
    let output_file = read_only_dir.path.join("saved_sad.xml");
    let output_file = output_file.to_str().expect("non UTF-8 path");

    let error = dump_db_to_xml(&load_fixture(), output_file)
        .expect_err("save into a read-only directory succeeded");

    assert_eq!(error.output_file, output_file);
    let SaveErrorCause::Io(cause) = &error.cause else {
        panic!("expected an I/O cause, got {:?}", error.cause);
    };
    assert!(
        matches!(
            cause.kind(),
            ErrorKind::PermissionDenied | ErrorKind::ReadOnlyFilesystem
        ),
        "unexpected cause {cause}"
    );
    let message = error.to_string();
    assert!(
        message.contains(output_file),
        "message lacks the file: {message}"
    );
    assert!(
        message.contains(&cause.to_string()),
        "message lacks the cause: {message}"
    );
    assert!(
        !Path::new(output_file).exists(),
        "{output_file} was written"
    );
}

#[test]
fn t47_fr_00046_full_disk_returns_error_naming_file_and_cause() {
    assert!(
        Path::new(FULL_DISK_FILE).exists(),
        "{FULL_DISK_FILE} missing"
    );

    let error =
        dump_db_to_xml(&load_fixture(), FULL_DISK_FILE).expect_err("save to a full disk succeeded");

    assert_eq!(error.output_file, FULL_DISK_FILE);
    let SaveErrorCause::Io(cause) = &error.cause else {
        panic!("expected an I/O cause, got {:?}", error.cause);
    };
    assert_eq!(
        cause.kind(),
        ErrorKind::StorageFull,
        "unexpected cause {cause}"
    );
    let message = error.to_string();
    assert!(
        message.contains(FULL_DISK_FILE),
        "message lacks the file: {message}"
    );
    assert!(
        message.contains(&cause.to_string()),
        "message lacks the cause: {message}"
    );
}

/// A database read failure is a save failure too, so it must not panic,
/// and must not truncate the file it would have replaced.
#[test]
fn t47_fr_00046_database_failure_returns_error_and_keeps_existing_file() {
    let work_dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("t47_database_failure_{}", std::process::id()));
    fs::create_dir_all(&work_dir)
        .unwrap_or_else(|err| panic!("cannot create {}: {err}", work_dir.display()));
    let output_file = work_dir.join("saved_sad.xml");
    fs::write(&output_file, "previous save").expect("cannot write the previous save");
    let output_file = output_file.to_str().expect("non UTF-8 path");
    // A database without a `document` row cannot be saved.
    let db_conn = Connection::open_in_memory().expect("cannot open in-memory database");
    db_create_in_mem_db(&db_conn);

    let error =
        dump_db_to_xml(&db_conn, output_file).expect_err("save without a document row succeeded");

    assert_eq!(error.output_file, output_file);
    assert!(
        matches!(error.cause, SaveErrorCause::Database(_)),
        "expected a database cause, got {:?}",
        error.cause
    );
    assert!(
        error.to_string().contains(output_file),
        "message lacks the file: {error}"
    );
    assert_eq!(fs::read_to_string(output_file).unwrap(), "previous save");
    fs::remove_dir_all(&work_dir)
        .unwrap_or_else(|err| panic!("cannot remove {}: {err}", work_dir.display()));
}
