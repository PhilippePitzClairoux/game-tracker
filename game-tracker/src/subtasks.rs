use std::time::Instant;
use chrono::{DateTime, Local};
use notify_rust::Notification;
use crate::errors::{Error, TimeTampering};
use crate::process_tree::ProcessInfo;
use crate::time::format_duration;
use crate::tracker::GamingTracker;

pub trait SubTask {

    /// Main function that executes SubTask
    fn execute(&mut self, tracker: &mut GamingTracker) -> Result<(), Error>;

}

fn notify(msg: &str) -> Result<(), Error> {
    Notification::new()
        .summary("WARNING")
        .body(msg)
        .show()?;

    Ok(())
}


pub struct GamesLogger;

impl GamesLogger {
    pub fn new() -> Box<Self> {
        Box::new(GamesLogger)
    }
}

impl SubTask for GamesLogger {
    fn execute(&mut self, tracker: &mut GamingTracker) -> Result<(), Error> {
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
    }
}

pub struct SessionEndGameKiller;

impl SessionEndGameKiller {
    pub fn new() -> Box<Self> {
        Box::new(SessionEndGameKiller)
    }
}

impl SubTask for SessionEndGameKiller {
    fn execute(&mut self, tracker: &mut GamingTracker) -> Result<(), Error> {
        if let Some(session) = tracker.session() && session.is_session_ended() {
            notify("Play time's over buddy! Go touch grass :-)")?;

            let known_games = tracker.gametime_tracker().iter()
                .flat_map(|(_, proc)| proc.into_iter());

            for proc in known_games {
                tracker.kill(proc)?;
            }
        }
        Ok(())
    }
}

pub struct WarnSessionEnding {
    was_warned: bool,
    threshold: f64,
    duration: chrono::Duration,
}

impl WarnSessionEnding {
    pub fn from(threshold: f64, session_duration: i64) -> Box<Self> {
        let value = chrono::Duration::seconds(
            ((threshold / 100_f64) * session_duration as f64).floor() as i64
        );

        Box::new(Self {
            duration: value,
            was_warned: false,
            threshold,
        })
    }

    pub fn duration(&self) -> chrono::Duration {
        self.duration
    }

}

impl SubTask for WarnSessionEnding {
    fn execute(&mut self, tracker: &mut GamingTracker) -> Result<(), Error> {
        if let Some(session) = tracker.session() {
            if !self.was_warned && !session.is_session_ended()
                && tracker.total_time_played() > self.duration {
                println!("Warning threshold reached : {}", self.threshold);
                self.was_warned = true;

                notify(
                    format!(
                        "{}% of session gaming played ({})",
                        self.threshold, format_duration(&self.duration)
                    ).as_str()
                )?;
            }

        } else {
            self.was_warned = false;
        }

        Ok(())
    }
}

pub struct ClockTampering {
    start_time: DateTime<Local>,
    uptime: Instant,
    detected: bool
}

impl ClockTampering {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            start_time: Local::now(),
            uptime: Instant::now(),
            detected: false,
        })
    }
}

impl SubTask for ClockTampering {
    fn execute(&mut self, _: &mut GamingTracker) -> Result<(), Error> {

        if self.detected {
            return Ok(());
        }

        let clock_estimation = Local::now()
            .signed_duration_since(self.start_time).num_seconds() as u64;
        let instant_estimation = self.uptime.elapsed().as_secs();

        if clock_estimation > instant_estimation {
            self.detected = true;
            return Err(TimeTampering::ClockTamperingError.into());
        }

        Ok(())
    }
}

pub struct RampageMode;

impl RampageMode {

    pub fn new() -> Box<Self> {
        Box::new(
            Self {}
        )
    }

}


impl SubTask for RampageMode {

    fn execute(&mut self, tracker: &mut GamingTracker) -> Result<(), Error> {
        let games: Vec<&ProcessInfo> = tracker.gametime_tracker().iter()
            .flat_map(|(_, proc)| proc.into_iter())
            .collect();

        for game in games {
            tracker.kill(game)?;
        }

        Ok(())
    }
}