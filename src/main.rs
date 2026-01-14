mod errors;
mod process_tree;
mod tracker;
mod time;
mod scheduler;

use std::time::Duration;
use clap::Parser;
use notify_rust::Notification;
use crate::errors::Errors;
use crate::scheduler::{timed_game_session, log_games_found, warn_game_session_near_end, Task};
use crate::time::to_seconds;
use crate::tracker::GameTracker;

#[derive(Parser, PartialOrd, PartialEq)]
struct Arguments {

    /// Number of hours of allowed play time (can be combined with minutes/seconds)
    #[clap(long, default_value_t = 0)]
    hours: u64,

    /// Number of minutes of allowed play time (can be combined with hours/seconds)
    #[clap(long, default_value_t = 0)]
    minutes: u64,

    /// Number of seconds of allowed play time (can be combined with hours/minutes)
    #[clap(long, default_value_t = 0)]
    seconds: u64,

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


fn notify(msg: &str) -> Result<(), Errors> {
    Notification::new()
        .summary("WARNING")
        .body(msg)
        .show()?;

    Ok(())
}


fn main() {
    let args = Arguments::parse();
    let mut tracker = GameTracker::new();
    tracker.load_config("configs/linux.toml")
        .expect("Failed to load config");
    let mut task = Task::using(Duration::from_secs(args.scan_interval), tracker);

    // log games found
    task.add(log_games_found());

    // kill games once session reaches it end
    let session_duration = to_seconds(args.hours, args.minutes, args.seconds);
    if session_duration > 0 && !args.monitor_only {
        task.add(timed_game_session(session_duration));
    }

    // setup warning when session end if near
    if args.warn {
        let threshold  = args.warning_threshold.unwrap_or(90.0);
        let value = ((threshold / 100_f64) * session_duration as f64)
            .floor() as u64;

        task.add(warn_game_session_near_end(threshold, value));
    }

    task.start().expect("failed to run main task...");
}
