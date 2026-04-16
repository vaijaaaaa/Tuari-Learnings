use rusqlite::{params, Connection, Result};

const MIGRATIONS: &[(&str, &str)] = &[(
    "0001_init",
    include_str!("0001_init.sql"),
)];

pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        ",
    )?;

    for (version, sql) in MIGRATIONS {
        let already_applied: i64 = conn.query_row(
            "SELECT COUNT(1) FROM schema_migrations WHERE version = ?1",
            params![version],
            |row| row.get(0),
        )?;

        if already_applied == 0 {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.execute(
                "INSERT INTO schema_migrations (version) VALUES (?1)",
                params![version],
            )?;
            tx.commit()?;
        }
    }

    Ok(())
}
