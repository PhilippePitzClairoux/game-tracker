use chrono::{DateTime, Duration, Local};
use rusqlite::{params, Connection};
use crate::errors::Error;
use crate::process_tree::ProcessInfo;
use crate::subtasks::SubTask;
use crate::tracker::GamingTracker;

pub fn init_database() -> Result<Connection, Error> {
    let conn = Connection::open("statistics.sqlite")?;
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

pub struct SaveStatistics {
    db_connection: Connection,
}

impl SaveStatistics {
    pub fn new() -> Result<Box<Self>, Error> {
        Ok(Box::new(
            Self {
                db_connection: init_database()?
            }
        ))
    }
}

impl SubTask for SaveStatistics {
    fn execute(&mut self, tracker: &mut GamingTracker) -> Result<(), Error> {
        for (name, processes) in tracker.gametime_tracker() {
            for process in processes {
                self.upsert_process(&process, name)?;
            }
        }

        Ok(())
    }
}

impl SaveStatistics {

    pub fn upsert_process(&self, proc: &ProcessInfo, game_name: &str) -> Result<(), Error> {
        let mut statement = self.db_connection.prepare("
            INSERT INTO game_tracker (pid, name, cmd, game_name, run_time, start_time)
                VALUES (?1, ?2, ?3, ?4, $5, DATETIME(?6, 'unixepoch'))
            ON CONFLICT (pid, name, cmd, start_time) DO UPDATE SET run_time = ?5
        ")?;

        statement.execute(params![proc.pid().as_u32(), proc.name(), proc.cmd(), game_name,
        proc.run_time() as i64, proc.start_time() as i64])?;

        Ok(())
    }

    pub fn time_played_by_date(&self, date: DateTime<Local>) -> Result<Duration, Error> {
        let mut statement = self.db_connection.prepare("
            SELECT SUM(run_time) as total
            FROM game_tracker
            WHERE date(game_tracker.start_time) like date(?1)
        ")?;
        let total: i64 = statement.query_one(
            params![date.to_rfc3339()],
            |row| row.get(0)
        )?;

        Ok(Duration::seconds(total))
    }
}