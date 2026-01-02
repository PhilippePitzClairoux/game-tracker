use std::thread;
use std::time::{Duration, SystemTime};
use crate::errors::Errors;
use crate::notify;
use crate::time::format_duration;
use crate::tracker::GameTracker;

pub struct Task {
    frequency: Duration,
    tracker: GameTracker,
    sub_tasks: Vec<SubTask>,
}

pub type SubTask = Box<dyn FnMut(&mut GameTracker) -> Result<(), Errors>>;

impl Task {
    fn new() -> Self {
        Task {
            frequency: Duration::from_secs(15),
            tracker: GameTracker::new(),
            sub_tasks: Vec::new()
        }
    }

    fn from(frequence: Duration) -> Self {
        Task {
            frequency: frequence,
            tracker: GameTracker::new(),
            sub_tasks: Vec::new()
        }
    }

    pub fn using(frequence: Duration, tracker: GameTracker) -> Self {
        Task {
            frequency: frequence,
            tracker,
            sub_tasks: Vec::new()
        }
    }

    pub fn add(&mut self, f: SubTask) -> &mut Self {
        self.sub_tasks.push(f);
        self
    }

    pub fn start(&mut self) -> Result<(), Errors> {
        loop {
            // time execution
            let start = SystemTime::now();

            // update tracker
            self.tracker.update_time_tracker();


            // execute main function
            for func in self.sub_tasks.iter_mut() {
                func(&mut self.tracker)?;
            }

            // optional wait
            if let Ok(elapsed) = start.elapsed() && !elapsed.is_zero() {
                let wait_remainder = self.frequency - elapsed;
                if wait_remainder <= self.frequency {
                    thread::sleep(wait_remainder);
                }
            }
        }
    }
}


pub fn generate_log_games_found() -> SubTask {
    Box::new(move |tracker: &mut GameTracker| {

        if tracker.gametime_tracker().len() == 0 {
            println!("No games have been found yet!")
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

pub fn generate_kill_games(duration: u64) -> SubTask {
    Box::new(move |tracker: &mut GameTracker| {
        let time_played = tracker.get_total_time_played();

        if time_played >= duration {
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
pub fn generate_warn_user(threshold: f64, value: u64) -> SubTask {
    let mut was_warned = false;

    Box::new(move |tracker: &mut GameTracker| {
        let time_played = tracker.get_total_time_played();
        if !was_warned && time_played >= value {
            println!("Warning threshold reached : {threshold}");
            was_warned = true;

            notify(
                format!("{}% of session played - {}",
                        threshold, format_duration(value)).as_str()
            )?;
        }

        Ok(())
    })
}