mod errors;
mod process_tree;
mod tracker;
mod security;

use std::time::Duration;
use chrono::TimeDelta;
use clap::Parser;
use notify_rust::Notification;
use crate::errors::Errors;
use crate::security::{fixed_interval_execution};
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

    /// Percentage of session duration to warn user of imminent end
    #[clap(long, default_value_t = 90.0)]
    warning_threshold_percent: f64,
}

fn format_duration(duration: u64) -> String {
    let delta = TimeDelta::new(duration as i64, 0)
        .expect("could not convert to time delta");

    let days = delta.num_days();
    let hours = delta.num_hours() - (24 * delta.num_days());
    let minutes = delta.num_minutes() - (delta.num_hours() * 60);
    let seconds = delta.num_seconds() - (delta.num_minutes() * 60);

    format!("{} days {} hour(s) {} minute(s) {} second(s)",
        days, hours, minutes, seconds
    )
}

fn notify(msg: &str) -> Result<(), Errors> {
    Notification::new()
        .summary("WARNING")
        .body(msg)
        .show()?;

    Ok(())
}

fn session_duration_seconds(hours: u64, minutes: u64, seconds: u64) -> u64 {
    (hours * 60 * 60) + (minutes * 60) + seconds
}

fn main() {
    let args = Arguments::parse();
    let mut tracker = GameTracker::new();
    tracker.load_config("configs/linux.toml")
        .expect("Failed to load config");
    let session_duration = session_duration_seconds(args.hours, args.minutes, args.seconds);

    // setup warning
    let mut warned = false;
    let warning_threshold = ((args.warning_threshold_percent / 100_f64) * session_duration as f64).floor() as u64;

    fixed_interval_execution(Duration::from_secs(args.scan_interval), move || {
        tracker.update_time_tracker();

        println!("All games found : ");
        for (game_name, process) in tracker.gametime_tracker().iter() {
            println!("{} '{}' up for {}",
                     process.pid(), game_name, format_duration(process.run_time())
            );
        }

        let time_played = tracker.get_total_time_played();
        println!("Total time played : {}\n", format_duration(time_played));

        if !warned && time_played >= warning_threshold {
            println!("Warning threshold reached : {warning_threshold}");
            warned = true;

            notify(
                format!("{}% of session played - closing game in : {}",
                        args.warning_threshold_percent, format_duration(warning_threshold)).as_str()
            ).expect("failed to notify warning threshold reach");
        }

        if time_played >= session_duration {
            notify("Play time's over buddy! Go touch grass :-)")?;
            for (_, game) in tracker.gametime_tracker().iter() {
                tracker.kill(game)?;
            }
        }

        Ok(())
    }).expect("error while tracking running games");
}
