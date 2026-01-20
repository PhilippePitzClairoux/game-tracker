use rusqlite::Connection;
use crate::errors::Error;
use crate::process_tree::ProcessInfo;

pub fn init_database() -> Result<Connection, Error> {
    let conn = Connection::open("db.sqlite")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS game_tracker (
            pid INTEGER NOT NULL,
            name TEXT,
            cmd TEXT,
            game_name TEXT,
            run_time INTEGER NOT NULL,
            start_time DATETIME NOT NULL,
            PRIMARY KEY (pid, name, cmd, start_time)
    )", ())?;


    Ok(conn)
}

pub fn upsert_process(connection: &mut Connection, proc: &ProcessInfo, game_name: &str) -> Result<(), Error> {
    let mut statement = connection.prepare("
    INSERT INTO game_tracker (pid, name, cmd, game_name, run_time, start_time)
        VALUES (?1, ?2, ?3, ?4, $5, DATETIME(?6, 'unixepoch'))
    ON CONFLICT (pid, name, cmd, start_time) DO UPDATE SET run_time = ?5
    ")?;

    statement.execute((proc.pid().as_u32(), proc.name(), proc.cmd(), game_name,
        proc.run_time() as i64, proc.start_time() as i64))?;

    Ok(())
}