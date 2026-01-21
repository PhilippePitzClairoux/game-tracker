mod process_tree;
mod tracker;
mod time;
mod scheduler;
mod db;
mod errors;
mod session;

use std::time::Duration;
use clap::Parser;
use crate::errors::Error;
use crate::scheduler::{timed_game_session, log_games_found, warn_game_session_near_end, GameTrackerScheduler, clock_tampering, save_stats, rampage_mode};
use crate::session::DailyGamingSession;
use crate::time::{format_duration, DurationParser};
use crate::tracker::GameTracker;

#[derive(Parser, Debug, PartialOrd, PartialEq)]
struct Arguments {

    /// Session duration (ex.: "30h 20m 10s", "3:30:00", "30h 2h 30m 6s 6s")
    #[arg(long)]
    session_duration: Option<DurationParser>,

    /// Delay between process scans
    #[arg(long, default_value_t = 15)]
    scan_interval: u64,

    /// Send warning of imminent session end
    #[arg(long, default_value_t = false)]
    warn: bool,

    /// Percentage of session played before sending warning
    /// (value must be between 0 and 100 - you can use decimals)
    #[arg(long, value_parser = f64_value_parser)]
    warning_threshold: Option<f64>,

    /// Monitor games only
    #[arg(long, default_value_t = false)]
    monitor_only: bool
}

fn f64_value_parser(v: &str) -> Result<f64, Error> {
    let parsed = v.parse::<f64>()?;
    if (0.0..=100.0).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(Error::InvalidThresholdError)
    }
}


fn main() -> Result<(), Error> {
    let args = Arguments::parse();
    let mut scheduler = GameTrackerScheduler::using(
        Duration::from_secs(args.scan_interval),
        GameTracker::try_from("game-tracker/configs/linux.toml")?
    );

    // log games found
    scheduler.add(log_games_found());
    scheduler.add(clock_tampering());
    scheduler.add(save_stats()?);

    // kill games once session reaches it end
    if let Some(session_duration) = args.session_duration{
        println!("Session duration enabled - total duration : {}", session_duration.to_string());
        scheduler.add_gaming_session(
                DailyGamingSession::from_duration(session_duration.to_duration())?
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
            Err(Error::TimeTamperingError(_))
            | Err(Error::TimedExecutionTamperingError(_)) => {
                println!("Tampering detected - activating rampage mode...");
                scheduler.add(rampage_mode());
            }
            Err(unhandled) => {
                println!("There was an unexpected error: {:?}", unhandled);
                return Err(unhandled);
            },
            _ =>  break
        }
    }

    Ok(())
}
