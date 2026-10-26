use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::{
    CHANNEL_CAPACITY, SHUTDOWN_JOIN, SqlValue, SqliteWriter, SqliteWriterError, WRITE_DEADLINE,
};

fn assert_send_sync<T: Send + Sync>() {}

fn wait_until_queued(writer: &SqliteWriter, count: usize) {
    let started = Instant::now();
    while writer.queued_for_test() < count {
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "writer queue did not reach {count}"
        );
        thread::yield_now();
    }
}

struct Hold {
    release: Option<mpsc::Sender<()>>,
    thread: Option<thread::JoinHandle<Result<(), SqliteWriterError>>>,
}

impl Hold {
    fn release(mut self) -> Result<(), SqliteWriterError> {
        self.signal();
        self.join()
    }

    fn signal(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
    }

    fn join(&mut self) -> Result<(), SqliteWriterError> {
        match self.thread.take() {
            Some(thread) => thread.join().expect("holder thread"),
            None => Ok(()),
        }
    }
}

impl Drop for Hold {
    fn drop(&mut self) {
        self.signal();
        let _ = self.join();
    }
}

fn hold(writer: &SqliteWriter) -> Hold {
    let (release_tx, release_rx) = mpsc::channel();
    let (entered_tx, entered_rx) = mpsc::channel();
    let writer = writer.clone();
    let thread = thread::spawn(move || {
        writer.transaction(move |_connection| {
            entered_tx
                .send(())
                .map_err(|_| SqliteWriterError::ChannelClosed)?;
            let _ = release_rx.recv();
            Ok(())
        })
    });
    entered_rx.recv().expect("writer entered hold");
    Hold {
        release: Some(release_tx),
        thread: Some(thread),
    }
}

fn file_writer() -> (tempfile::TempDir, SqliteWriter) {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let writer = SqliteWriter::open(dir.path().join("db.sqlite")).expect("open writer");
    (dir, writer)
}

fn names(path: &std::path::Path) -> Vec<String> {
    let connection = rusqlite::Connection::open(path).expect("read connection");
    connection
        .busy_timeout(Duration::from_secs(2))
        .expect("busy timeout");
    let mut statement = connection
        .prepare("SELECT name FROM item ORDER BY id")
        .expect("prepare");
    statement
        .query_map([], |row| row.get(0))
        .expect("query")
        .collect::<Result<Vec<_>, _>>()
        .expect("rows")
}

#[test]
fn writer_is_send_and_sync() {
    assert_send_sync::<SqliteWriter>();
}

#[test]
fn error_display_and_source() {
    assert_eq!(
        SqliteWriterError::ChannelClosed.to_string(),
        "DBSQ.CHANNEL_CLOSED: writer channel closed; Reopen the database with SqliteWriter::open. The writer thread has exited."
    );
    assert_eq!(
        SqliteWriterError::QueueFull.to_string(),
        "DBSQ.QUEUE_FULL: write queue full; Retry the call. The job was not accepted."
    );
    assert_eq!(
        SqliteWriterError::DeadlineExceeded.to_string(),
        "DBSQ.DEADLINE_EXCEEDED: write deadline exceeded; Do not retry a non-idempotent write until you check the database. The job was accepted and may already be committed."
    );
    assert_eq!(SqliteWriterError::QueueFull.code(), "DBSQ.QUEUE_FULL");
    assert!(SqliteWriterError::QueueFull.recovery().contains("Retry"));
    assert!(
        SqliteWriterError::DeadlineExceeded
            .recovery()
            .contains("may already be committed")
    );
    assert!(
        SqliteWriterError::ChannelClosed
            .recovery()
            .contains("Reopen")
    );
    assert!(std::error::Error::source(&SqliteWriterError::QueueFull).is_none());
    let sqlite = SqliteWriterError::sqlite(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(1),
        Some("no such table: missing".to_owned()),
    ));
    assert_eq!(
        sqlite.to_string(),
        "DBSQ.SQLITE: sqlite error 1: no such table: missing; Read the SQLite code and the source error. This crate does not retry the statement."
    );
    assert!(std::error::Error::source(&sqlite).is_some());
    let commit = SqliteWriterError::commit(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(13),
        Some("disk full".to_owned()),
    ));
    assert_eq!(
        commit.to_string(),
        "DBSQ.COMMIT: commit failed: sqlite error 13: disk full; Read the SQLite code and the source error. The commit did not succeed."
    );
    assert!(
        std::error::Error::source(&commit)
            .and_then(|source| source.downcast_ref::<rusqlite::Error>())
            .is_some()
    );
    assert_eq!(commit.code(), "DBSQ.COMMIT");
    assert_eq!(
        SqliteWriterError::ReplyTypeMismatch.to_string(),
        "DBSQ.REPLY_TYPE_MISMATCH: transaction reply type mismatch; Do not retry a non-idempotent write until you check the database. The transaction reply was not the type the body returned."
    );
    assert_eq!(
        SqliteWriterError::ReplyTypeMismatch.code(),
        "DBSQ.REPLY_TYPE_MISMATCH"
    );
    assert!(
        SqliteWriterError::ReplyTypeMismatch
            .recovery()
            .contains("was not the type")
    );
    assert!(std::error::Error::source(&SqliteWriterError::ReplyTypeMismatch).is_none());
}

#[test]
fn open_reports_a_missing_parent_directory() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let path = dir.path().join("missing-dir").join("db.sqlite");
    let error = SqliteWriter::open(&path).expect_err("missing parent");
    assert!(matches!(error, SqliteWriterError::Open { .. }));
    assert!(error.to_string().starts_with("DBSQ.OPEN: open failed:"));
    assert!(error.to_string().contains(error.recovery()));
    assert!(
        std::error::Error::source(&error)
            .and_then(|source| source.downcast_ref::<rusqlite::Error>())
            .is_some()
    );
}

#[test]
fn open_and_execute_round_trip() {
    let (dir, writer) = file_writer();
    writer
        .execute(
            "CREATE TABLE item (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
            &[],
        )
        .expect("create");
    let inserted = writer
        .execute(
            "INSERT INTO item (id, name) VALUES (?1, ?2)",
            &[SqlValue::Integer(1), SqlValue::Text("hello".to_owned())],
        )
        .expect("insert");
    assert_eq!(inserted, 1);
    assert_eq!(
        names(dir.path().join("db.sqlite").as_path()),
        vec!["hello".to_owned()]
    );
    let rendered = format!("{writer:?}");
    assert!(!rendered.contains("hello"));
    assert!(!rendered.contains("INSERT"));
}

#[test]
fn concurrent_executes_reply_after_commit() {
    let (dir, writer) = file_writer();
    let path = dir.path().join("db.sqlite");
    writer
        .execute(
            "CREATE TABLE item (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
            &[],
        )
        .expect("create");
    let held = hold(&writer);
    let first = writer.clone();
    let second = writer.clone();
    let first_path = path.clone();
    let second_path = path.clone();
    let first_call = thread::spawn(move || {
        let inserted = first.execute(
            "INSERT INTO item (id, name) VALUES (?1, ?2)",
            &[SqlValue::Integer(1), SqlValue::Text("a".to_owned())],
        )?;
        Ok::<_, SqliteWriterError>((inserted, names(&first_path).len()))
    });
    wait_until_queued(&writer, 1);
    let second_call = thread::spawn(move || {
        let inserted = second.execute(
            "INSERT INTO item (id, name) VALUES (?1, ?2)",
            &[SqlValue::Integer(2), SqlValue::Text("b".to_owned())],
        )?;
        Ok::<_, SqliteWriterError>((inserted, names(&second_path).len()))
    });
    wait_until_queued(&writer, 2);
    held.release().expect("release hold");
    let (first_inserted, first_visible) = first_call.join().expect("first thread").expect("first");
    let (second_inserted, second_visible) =
        second_call.join().expect("second thread").expect("second");
    assert_eq!(first_inserted, 1);
    assert_eq!(second_inserted, 1);
    assert_eq!(first_visible, 2);
    assert_eq!(second_visible, 2);
    assert_eq!(names(&path), vec!["a".to_owned(), "b".to_owned()]);
}

#[test]
fn transaction_body_is_not_interleaved() {
    let (dir, writer) = file_writer();
    let path = dir.path().join("db.sqlite");
    writer
        .execute(
            "CREATE TABLE item (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
            &[],
        )
        .expect("create");
    let (entered_tx, entered_rx) = mpsc::channel();
    let (proceed_tx, proceed_rx) = mpsc::channel();
    let body_writer = writer.clone();
    let body = thread::spawn(move || {
        body_writer.transaction(move |connection| {
            connection.execute(
                "INSERT INTO item (id, name) VALUES (?1, ?2)",
                &[SqlValue::Integer(1), SqlValue::Text("A".to_owned())],
            )?;
            entered_tx
                .send(())
                .map_err(|_| SqliteWriterError::ChannelClosed)?;
            let _ = proceed_rx.recv();
            connection.execute(
                "INSERT INTO item (id, name) VALUES (?1, ?2)",
                &[SqlValue::Integer(2), SqlValue::Text("B".to_owned())],
            )?;
            let rendered = format!("{connection:?}");
            assert!(!rendered.contains("INSERT"));
            assert!(!rendered.contains("A"));
            Ok(())
        })
    });
    entered_rx.recv().expect("body entered");
    let other = writer.clone();
    let other_call = thread::spawn(move || {
        other.execute(
            "INSERT INTO item (id, name) VALUES (?1, ?2)",
            &[SqlValue::Integer(3), SqlValue::Text("C".to_owned())],
        )
    });
    wait_until_queued(&writer, 1);
    proceed_tx.send(()).expect("proceed");
    body.join().expect("body thread").expect("transaction");
    other_call.join().expect("other thread").expect("execute");
    assert_eq!(
        names(&path),
        vec!["A".to_owned(), "B".to_owned(), "C".to_owned()]
    );
}

#[test]
fn savepoint_replay_keeps_the_statements_around_a_failure() {
    let (dir, writer) = file_writer();
    let path = dir.path().join("db.sqlite");
    writer
        .execute(
            "CREATE TABLE item (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
            &[],
        )
        .expect("create");
    let held = hold(&writer);
    let writers = [writer.clone(), writer.clone(), writer.clone()];
    let statements = [(1, "A", true), (1, "B", false), (3, "C", true)];
    let mut calls = Vec::new();
    for (index, ((id, name, _), caller)) in statements.iter().zip(writers).enumerate() {
        let id = *id;
        let name = (*name).to_owned();
        let call = thread::spawn(move || {
            caller.execute(
                "INSERT INTO item (id, name) VALUES (?1, ?2)",
                &[SqlValue::Integer(id), SqlValue::Text(name)],
            )
        });
        calls.push(call);
        wait_until_queued(&writer, index + 1);
    }
    held.release().expect("release hold");
    let mut results = Vec::new();
    for call in calls {
        results.push(call.join().expect("replay thread"));
    }
    assert!(results[0].is_ok());
    assert!(matches!(results[1], Err(SqliteWriterError::Sqlite { .. })));
    assert!(results[2].is_ok());
    assert_eq!(names(&path), vec!["A".to_owned(), "C".to_owned()]);
}

#[test]
fn dropping_the_last_handle_joins_within_shutdown_join() {
    let writer = SqliteWriter::open(":memory:").expect("open");
    let finished = writer.finished_flag_for_test();
    let clone = writer.clone();
    drop(writer);
    assert!(!finished.load(Ordering::SeqCst));
    let started = Instant::now();
    drop(clone);
    assert!(finished.load(Ordering::SeqCst));
    assert!(started.elapsed() < SHUTDOWN_JOIN);
}

#[test]
fn queue_full_returns_before_the_write_deadline() {
    let writer = SqliteWriter::open(":memory:").expect("open");
    writer
        .execute("CREATE TABLE filler (n INTEGER)", &[])
        .expect("create");
    let held = hold(&writer);
    let mut calls = Vec::with_capacity(CHANNEL_CAPACITY);
    for _ in 0..CHANNEL_CAPACITY {
        let writer = writer.clone();
        calls.push(thread::spawn(move || {
            writer.execute("INSERT INTO filler (n) VALUES (1)", &[])
        }));
    }
    wait_until_queued(&writer, CHANNEL_CAPACITY);
    let started = Instant::now();
    let error = writer
        .execute("INSERT INTO filler (n) VALUES (1)", &[])
        .expect_err("one past capacity");
    assert!(matches!(error, SqliteWriterError::QueueFull));
    assert!(started.elapsed() < WRITE_DEADLINE);
    held.release().expect("release hold");
    for call in calls {
        call.join().expect("filler thread").expect("filler insert");
    }
}

#[test]
fn accepted_job_returns_deadline_exceeded_when_the_writer_is_held() {
    let deadline = Duration::from_millis(200);
    let dir = tempfile::TempDir::new().expect("temp dir");
    let writer =
        SqliteWriter::open_with_deadline(dir.path().join("db.sqlite"), deadline).expect("open");
    let held = hold(&writer);
    let caller = writer.clone();
    let started = Instant::now();
    let call = thread::spawn(move || caller.execute("CREATE TABLE late (n INTEGER)", &[]));
    wait_until_queued(&writer, 1);
    let error = call.join().expect("deadline thread").expect_err("deadline");
    let elapsed = started.elapsed();
    assert!(matches!(error, SqliteWriterError::DeadlineExceeded));
    assert!(elapsed >= Duration::from_millis(150));
    assert!(elapsed < WRITE_DEADLINE);
    // The holder call started before this deadline and is still inside the body,
    // so its own reply wait also exceeds the shortened deadline.
    let _ = held.release();
}

#[test]
fn writer_panic_closes_every_enqueued_caller() {
    let writer = SqliteWriter::open(":memory:").expect("open");
    let held = hold(&writer);
    let panic_writer = writer.clone();
    let first = writer.clone();
    let second = writer.clone();
    let panic_call = thread::spawn(move || panic_writer.panic_for_test());
    wait_until_queued(&writer, 1);
    let first_call = thread::spawn(move || first.execute("CREATE TABLE panic_a (n INTEGER)", &[]));
    wait_until_queued(&writer, 2);
    let second_call =
        thread::spawn(move || second.execute("CREATE TABLE panic_b (n INTEGER)", &[]));
    wait_until_queued(&writer, 3);
    held.release().expect("release hold");
    assert!(matches!(
        panic_call.join().expect("panic thread"),
        Err(SqliteWriterError::ChannelClosed)
    ));
    assert!(matches!(
        first_call.join().expect("first thread"),
        Err(SqliteWriterError::ChannelClosed)
    ));
    assert!(matches!(
        second_call.join().expect("second thread"),
        Err(SqliteWriterError::ChannelClosed)
    ));
    let follow_up = writer.execute("CREATE TABLE after_panic (n INTEGER)", &[]);
    assert!(matches!(follow_up, Err(SqliteWriterError::ChannelClosed)));
}

#[test]
fn open_enforces_foreign_keys_and_wal() {
    let (dir, writer) = file_writer();
    let path = dir.path().join("db.sqlite");
    writer
        .execute("CREATE TABLE parent (id INTEGER PRIMARY KEY)", &[])
        .expect("parent");
    writer
        .execute(
            "CREATE TABLE child (id INTEGER PRIMARY KEY, parent_id INTEGER NOT NULL REFERENCES parent(id))",
            &[],
        )
        .expect("child");
    let error = writer
        .execute(
            "INSERT INTO child (id, parent_id) VALUES (?1, ?2)",
            &[SqlValue::Integer(1), SqlValue::Integer(9)],
        )
        .expect_err("foreign key");
    assert!(matches!(error, SqliteWriterError::Sqlite { .. }), "{error}");
    let connection = rusqlite::Connection::open(&path).expect("read connection");
    let mode: String = connection
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .expect("journal mode");
    assert_eq!(mode.to_ascii_lowercase(), "wal");
}

#[test]
fn writer_thread_is_named_for_the_database_file() {
    let (_dir, writer) = file_writer();
    let name = writer
        .transaction(|_connection| Ok(std::thread::current().name().unwrap_or_default().to_owned()))
        .expect("thread name");
    assert_eq!(name, "sqlite-writer:db.sqlite");
}
