mod errors;
mod process_tree;
mod tracker;

use std::thread;
use std::time::Duration;
use chrono::TimeDelta;
use clap::Parser;
use notify_rust::Notification;
use crate::tracker::GameTracker;

#[derive(Parser, PartialOrd, Eq, PartialEq)]
struct Arguments {

    /// Number of hours of allowed play time (can be combined with minutes/seconds)
    #[clap(long, default_value = "0")]
    hours: u64,

    /// Number of minutes of allowed play time (can be combined with hours/seconds)
    #[clap(long, default_value = "0")]
    minutes: u64,

    /// Number of seconds of allowed play time (can be combined with hours/minutes)
    #[clap(long, default_value = "0")]
    seconds: u64,

    /// Delay between process scans
    #[clap(long, default_value_t = 15)]
    scan_interval: u64,
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

fn notify(msg: &str) {
    Notification::new()
        .summary("WARNING")
        .body(msg)
        .show()
        .expect("could not send notification");
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

    loop {
        tracker.update_time_tracker();

        println!("All games found : ");
        for (game_name, process) in tracker.gametime_tracker().iter() {
            println!("{} '{}' up for {}",
                     process.pid(), game_name, format_duration(process.run_time())
            );
        }

        let time_played = tracker.get_total_time_played();
        println!("Total time played : {}\n", format_duration(time_played));

        if time_played >= session_duration {
            notify("Play time's over buddy! Go touch grss :-)");
            tracker.gametime_tracker().iter()
                .for_each(|(_, game)| {
                    tracker.kill(game).expect("could not kill game");
                });
        }

        thread::sleep(Duration::from_secs(args.scan_interval));
    }
}
