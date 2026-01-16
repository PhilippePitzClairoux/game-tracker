mod errors;
mod process_tree;
mod tracker;
mod time;
mod scheduler;

use std::process::exit;
use std::time::Duration;
use clap::Parser;
use crate::errors::Error;
use crate::scheduler::{timed_game_session, log_games_found, warn_game_session_near_end, GameTrackerScheduler};
use crate::time::{format_duration, SessionDurationParser};
use crate::tracker::GameTracker;

#[derive(Parser, PartialOrd, PartialEq)]
struct Arguments {

    /// Session duration (ex.: "30h 20m 10s", "3:30:00", "30h 2h 30m 6s 6s")
    #[clap(long)]
    session_duration: Option<SessionDurationParser>,

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
    #[clap(long, default_value_t = true)]
    monitor_only: bool
}


fn main() {
    let args = Arguments::parse();
    let mut tracker = GameTracker::new();
    tracker.load_config("configs/linux.toml")
        .expect("Failed to load config");
    let mut scheduler = GameTrackerScheduler::using(Duration::from_secs(args.scan_interval), tracker);

    // log games found
    scheduler.add(log_games_found());

    // kill games once session reaches it end
    if let Some(session_duration) = args.session_duration && !args.monitor_only {
        println!("Session duration enabled - total duration : {}", session_duration.to_string());
        scheduler.add(
            timed_game_session(session_duration.to_seconds())
        );

        // setup warning when session end if near
        if args.warn {
            let threshold  = args.warning_threshold.unwrap_or(90.0);
            let value = ((threshold / 100_f64) * session_duration.to_seconds() as f64)
                .floor() as u64;

            println!("User warning enabled - threshold={}, warning_after=\"{}\"",
                     threshold, format_duration(value)
            );
            scheduler.add(warn_game_session_near_end(threshold, value));
        }
    }

    loop {
        match scheduler.start() {
            Err(Error::DesynchronizedTimerError(value)) => {
                println!("Potential tampering detected - elapsed detected a desynchronization \
                between a timer and the system clock ({} seconds). Restarting scheduler...", value);
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
