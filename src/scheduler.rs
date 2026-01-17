use std::ops::Add;
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use chrono::{Days, Timelike};
use notify_rust::Notification;
use crate::errors::Error;
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

    pub fn add(&mut self, f: SubTask) -> &mut Self {
        self.sub_tasks.push(f);
        self
    }

    pub fn start(&mut self) -> Result<(), Error> {
        let current_day = chrono::Local::now();

        loop {
            // time execution
            let start = Instant::now();

            // update tracker
            self.tracker.update_time_tracker();


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
            output += format!("{} '{}' has been running for: {}\n",
                              proc.pid(), game_name, format_duration(proc.run_time())).as_str()
        }

        println!("{}", output);
        Ok(())
    })
}

pub fn timed_game_session(duration: u64) -> SubTask {
    Box::new(move |tracker: &mut GameTracker| {
        let time_played = tracker.get_total_time_played();

        if time_played >= duration {
            notify("Play time's over buddy! Go touch grass :-)")?;

            let known_games = tracker.gametime_tracker().iter()
                .flat_map(|(_, proc)| proc.into_iter());

            for proc in known_games {
                tracker.kill(proc);
            }
        }
        Ok(())
    })
}

pub fn warn_game_session_near_end(threshold: f64, duration: u64) -> SubTask {
    let mut was_warned = false;

    Box::new(move |tracker: &mut GameTracker| {
        let time_played = tracker.get_total_time_played();
        if !was_warned && time_played >= duration {
            println!("Warning threshold reached : {threshold}");
            was_warned = true;

            notify(
                format!("{}% of session played - {}",
                        threshold, format_duration(duration)).as_str()
            )?;
        }

        Ok(())
    })
}

pub fn clock_tampering() -> SubTask {
    let start_time = chrono::Local::now();
    let uptime = Instant::now();

    Box::new(move |tracker: &mut GameTracker| {
        println!("{} {}", chrono::Local::now().signed_duration_since(start_time), uptime.elapsed().as_secs());
        let clock_estimation = chrono::Local::now()
            .signed_duration_since(start_time).num_seconds() as u64;
        let instant_estimation = uptime.elapsed().as_secs();

        if clock_estimation > instant_estimation {
            println!("CLOCK TAMPERING DETECTED!")
        }

        Ok(())
    })
}

pub fn timed_execution() -> Duration {
    let start = Instant::now();

    for _ in 0..10_000 {
        std::hint::black_box(
            unsafe {
                std::ptr::read_volatile(&0u8)
            }
        );
    }

    start.elapsed()
}

pub fn timing_tampering() -> SubTask {
    let mut average: u128 = 0;
    for _ in 0..10 {
        average += timed_execution().as_nanos();
    }

    average /= 10;

    Box::new(move |tracker: &mut GameTracker| {
        let elapsed = timed_execution();

        // TODO : this does not work as intented - maybe calculate a % increase based off last iteration ?
        println!("Reading 10_000 bytes took {} secs", elapsed.as_micros());
        if elapsed.as_nanos() > average {
            println!("TAMPERING DETECTED!");
        }

        Ok(())
    })
}
