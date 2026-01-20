use std::thread;
use std::time::{Duration, Instant};
use notify_rust::Notification;
use crate::errors::Error;
use crate::db::{init_database, upsert_process};
use crate::time::format_duration;
use crate::tracker::GameTracker;

pub struct GameTrackerScheduler {
    frequency: Duration,
    tracker: GameTracker,
    sub_tasks: Vec<SubTask>,
}

pub type SubTask = Box<dyn FnMut(&mut GameTracker) -> Result<(), Error>>;

impl GameTrackerScheduler {
    fn new() -> Self {
        GameTrackerScheduler {
            frequency: Duration::from_secs(15),
            tracker: GameTracker::new(),
            sub_tasks: Vec::new()
        }
    }

    fn from(frequence: Duration) -> Self {
        GameTrackerScheduler {
            frequency: frequence,
            tracker: GameTracker::new(),
            sub_tasks: Vec::new()
        }
    }

    pub fn using(frequence: Duration, tracker: GameTracker) -> Self {
        GameTrackerScheduler {
            frequency: frequence,
            tracker,
            sub_tasks: Vec::new()
        }
    }

    pub fn modify_tracker(&mut self) -> &mut GameTracker {
        &mut self.tracker
    }

    pub fn add(&mut self, f: SubTask) -> &mut Self {
        self.sub_tasks.push(f);
        self
    }

    pub fn start(&mut self) -> Result<(), Error> {
        loop {
            // time execution
            let start = Instant::now();

            // update tracker
            self.tracker.update_time_tracker()?;


            // execute SubTasks
            for func in self.sub_tasks.iter_mut() {
                func(&mut self.tracker)?;
            }

            // optional wait
            if let Some(remainder) = self.frequency.checked_sub(start.elapsed()) {
                thread::sleep(remainder);
            }
        }
    }
}

fn notify(msg: &str) -> Result<(), Error> {
    Notification::new()
        .summary("WARNING")
        .body(msg)
        .show()?;

    Ok(())
}

pub fn log_games_found() -> SubTask {
    Box::new(move |tracker: &mut GameTracker| {

        if tracker.gametime_tracker().len() == 0 {
            println!("No games have been found yet!");
            return Ok(())
        }

        let mut output= String::new();
        output += "All games found: \n";

        let games_found = tracker.gametime_tracker().iter()
            .flat_map(|(game_name, processes)| {
                processes.iter().map(move |process| (process, game_name))
            });

        for (proc, game_name) in games_found {
            let dur = chrono::Duration::seconds(proc.run_time() as i64);
            output += format!("{} '{}' has been running for: {}\n",
                              proc.pid(), game_name, format_duration(&dur)
            ).as_str()
        }

        println!("{}", output);
        Ok(())
    })
}

pub fn timed_game_session() -> SubTask {
    Box::new(move |tracker: &mut GameTracker| {
        if let Some(session) = tracker.session() && session.is_session_ended() {
            notify("Play time's over buddy! Go touch grass :-)")?;

            let known_games = tracker.gametime_tracker().iter()
                .flat_map(|(_, proc)| proc.into_iter());

            for proc in known_games {
                tracker.kill(proc)?;
            }
        }
        Ok(())
    })
}

pub fn warn_game_session_near_end(threshold: f64, duration: chrono::Duration) -> SubTask {
    let mut was_warned = false;

    Box::new(move |tracker: &mut GameTracker| {
        if let Some(session) = tracker.session() {
            if !was_warned && !session.is_session_ended() && tracker.total_time_played() > duration {
                println!("Warning threshold reached : {threshold}");
                was_warned = true;

                notify(
                    format!(
                        "{}% of session gaming played ({})",
                        threshold, format_duration(&duration)
                    ).as_str()
                )?;
            }

        } else {
            was_warned = false;
        }

        Ok(())
    })
}

pub fn clock_tampering() -> SubTask {
    let start_time = chrono::Local::now();
    let uptime = Instant::now();

    Box::new(move |_: &mut GameTracker| {
        let clock_estimation = chrono::Local::now()
            .signed_duration_since(start_time).num_seconds() as u64;
        let instant_estimation = uptime.elapsed().as_secs();

        if clock_estimation > instant_estimation {
            return Err(Error::ClockTamperingError);
        }

        Ok(())
    })
}

pub fn save_stats() -> Result<SubTask, Error> {
    let mut connection = init_database()?;

    Ok(Box::new(move |tracker: &mut GameTracker| {
        for (name, processes) in tracker.gametime_tracker() {
            for process in processes {
                upsert_process(&mut connection, &process, name)?;
            }
        }
        Ok(())
    }))
}
