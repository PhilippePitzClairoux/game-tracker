mod process_tree;
mod tracker;
mod time;
mod scheduler;
mod db;
mod errors;
mod session;

use std::process::exit;
use std::time::Duration;
use clap::Parser;
use tampering_profiler_support::Errors::TamperingDetected;
use crate::errors::Error;
use crate::scheduler::{timed_game_session, log_games_found, warn_game_session_near_end, GameTrackerScheduler, clock_tampering, save_stats};
use crate::session::DailyGamingSession;
use crate::time::{format_duration, DurationParser};
use crate::tracker::GameTracker;

#[derive(Parser, PartialOrd, PartialEq)]
struct Arguments {

    /// Session duration (ex.: "30h 20m 10s", "3:30:00", "30h 2h 30m 6s 6s")
    #[clap(long)]
    session_duration: Option<DurationParser>,

    /// Delay between process scans
    #[clap(long, default_value_t = 15)]
    scan_interval: u64,

    /// Send warning of imminent session end
    #[clap(long, default_value_t = false)]
    warn: bool,

    /// Percentage of session played before sending warning
    #[clap(long)]
    warning_threshold: Option<f64>,

    /// Monitor games only
    #[clap(long, default_value_t = false)]
    monitor_only: bool
}


fn main() {
    let args = Arguments::parse();
    let mut tracker = GameTracker::new();
    tracker.load_config("game-tracker/configs/linux.toml")
        .expect("Failed to load config");
    let mut scheduler = GameTrackerScheduler::using(Duration::from_secs(args.scan_interval), tracker);

    // log games found
    scheduler.add(log_games_found());
    scheduler.add(clock_tampering());

    match save_stats() {
        Ok(task) => { scheduler.add(task); },
        Err(e) => println!("will not save statistics to database (save_stats() failed) : {:?}", e),
    }

    // kill games once session reaches it end
    if let Some(session_duration) = args.session_duration{
        println!("Session duration enabled - total duration : {}", session_duration.to_string());
        scheduler.modify_tracker().add_gaming_session(
                DailyGamingSession::from_duration(session_duration.to_duration())
                    .expect("could not create a daily gaming session")
        );

        if !args.monitor_only {
            scheduler.add(timed_game_session());
        }

        // setup warning when session end if near
        if args.warn {
            let threshold  = args.warning_threshold.unwrap_or(90.0);
            let value = chrono::Duration::seconds(
                ((threshold / 100_f64) * session_duration.to_seconds() as f64).floor() as i64
            );

            println!("User warning enabled - threshold={}%, warning_after=\"{}\"",
                     threshold, format_duration(&value)
            );

            scheduler.add(warn_game_session_near_end(threshold, value));
        }
    }

    loop {
        match scheduler.start() {
            Err(Error::DesynchronizedTimerError(value)) => {
                println!("Potential tampering detected - elapsed detected a desynchronization \
                between a timer and the system clock ({} seconds). Restarting scheduler...", value);
            },
            Err(Error::ProfilerError(TamperingDetected(name, duration))) => {
                println!("Tampering detected - execution of {} lasted {} seconds", name, duration);
            }
            Err(Error::ClockTamperingError) => {
                println!("Clock tampering detected - someone changed the local time in order to \
                potentially tamper with allowed session duration");
            }
            Err(unhandled) => {
                println!("There was an unexpected error: {:?}", unhandled);
                break
            },
            _ =>  break
        }
    }

    exit(-1);
}
