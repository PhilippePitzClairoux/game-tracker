mod process_tree;
mod tracker;
mod time;
mod scheduler;
mod db;
mod errors;
mod session;
mod subtasks;

use std::time::Duration;
use clap::Parser;
use crate::db::SaveStatistics;
use crate::errors::Error;
use crate::scheduler::GameTrackerScheduler;
use crate::session::DailyGamingSession;
use crate::subtasks::{
    ClockTampering, GamesLogger, RampageMode,
    SessionEndGameKiller, WarnSessionEnding};
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
    monitor_only: bool,

    /// Enable rampage mode
    /// (kills all games when detected tampering detected)
    #[arg(long, default_value_t = false)]
    rampage_mode: bool
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
    scheduler.add(GamesLogger::new());
    scheduler.add(ClockTampering::new());
    scheduler.add(SaveStatistics::new()?);

    // kill games once session reaches it end
    if let Some(session_duration) = args.session_duration {
        println!("Session duration enabled - total duration : {}", session_duration.to_string());
        scheduler.add_gaming_session(
                DailyGamingSession::from_duration(
                    session_duration.to_duration()
                )?
        );

        if !args.monitor_only {
            scheduler.add(SessionEndGameKiller::new());
        }

        // setup warning when session end if near
        if args.warn {
            let threshold  = args.warning_threshold.unwrap_or(90.0);
            let warn_session_ending = WarnSessionEnding::from(
                threshold, session_duration.to_seconds()
            );

            println!("User warning enabled - threshold={}%, warning_after=\"{}\"",
                     threshold, format_duration(&warn_session_ending.duration())
            );

            scheduler.add(warn_session_ending);
        }
    }

    let mut rampage_activated: bool = false;
    loop {
        match scheduler.start() {
            Err(Error::TimeTamperingError(_))
            | Err(Error::TimedExecutionTamperingError(_)) => {
                println!("Tampering detected - activating rampage mode...");
                if args.rampage_mode && !rampage_activated {
                    rampage_activated = true;
                    scheduler.add(RampageMode::new());
                }
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
