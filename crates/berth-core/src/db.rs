use std::path::Path;

use rusqlite::Connection;

use crate::error::{Error, Result};

pub const SCHEMA_VERSION: u32 = 1;

const MIGRATION_1: &str = "
CREATE TABLE packages (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    provider TEXT NOT NULL,
    host TEXT NOT NULL,
    repo_path TEXT NOT NULL,
    asset_regex TEXT,
    bin_path TEXT,
    pinned INTEGER NOT NULL DEFAULT 0,
    allow_prerelease INTEGER NOT NULL DEFAULT 0,
    active_version TEXT,
    installed_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE versions (
    id INTEGER PRIMARY KEY,
    package_id INTEGER NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    tag TEXT NOT NULL,
    asset_name TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    size INTEGER,
    verified INTEGER NOT NULL DEFAULT 0,
    source_url TEXT NOT NULL,
    installed_at TEXT NOT NULL
);
CREATE TABLE http_cache (
    url TEXT PRIMARY KEY,
    etag TEXT,
    last_modified TEXT,
    body_hash TEXT,
    fetched_at TEXT NOT NULL
);
";

const MIGRATIONS: [&str; SCHEMA_VERSION as usize] = [MIGRATION_1];

pub fn open(path: &Path) -> Result<Connection> {
    let connection = Connection::open(path)?;
    connection.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )?;
    migrate(&connection)?;
    tracing::debug!(path = %path.display(), version = SCHEMA_VERSION, "opened berth database");
    Ok(connection)
}

fn migrate(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
    )?;

    let current = current_schema_version(connection)?;
    if current > SCHEMA_VERSION {
        return Err(Error::SchemaTooNew {
            found: current,
            supported: SCHEMA_VERSION,
        });
    }

    for (index, migration) in MIGRATIONS.iter().enumerate() {
        let version = index as u32 + 1;
        if version <= current {
            continue;
        }
        let transaction = connection.unchecked_transaction()?;
        transaction.execute_batch(migration)?;
        transaction.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)",
            [version.to_string()],
        )?;
        transaction.commit()?;
    }

    Ok(())
}

fn current_schema_version(connection: &Connection) -> Result<u32> {
    let value = match connection.query_row(
        "SELECT value FROM meta WHERE key = 'schema_version'",
        [],
        |row| row.get::<_, String>(0),
    ) {
        Ok(value) => value,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(0),
        Err(error) => return Err(error.into()),
    };
    value
        .parse::<u32>()
        .map_err(|_| Error::CorruptState(format!("invalid schema_version: {value}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_schema_and_version() {
        let dir = tempfile::tempdir().unwrap();
        let connection = open(&dir.path().join("berth.db")).unwrap();
        for table in ["meta", "packages", "versions", "http_cache"] {
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "table {table} missing");
        }
        assert_eq!(current_schema_version(&connection).unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn reopening_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("berth.db");
        drop(open(&path).unwrap());
        let connection = open(&path).unwrap();
        assert_eq!(current_schema_version(&connection).unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn refuses_newer_schema() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("berth.db");
        {
            let connection = open(&path).unwrap();
            connection
                .execute(
                    "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', '999')",
                    [],
                )
                .unwrap();
        }
        let error = open(&path).unwrap_err();
        assert!(matches!(
            error,
            Error::SchemaTooNew {
                found: 999,
                supported: SCHEMA_VERSION
            }
        ));
    }
}
