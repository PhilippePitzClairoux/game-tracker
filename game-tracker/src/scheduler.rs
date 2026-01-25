use std::thread;
use std::time::{Duration, Instant};
use crate::errors::{Error};
use crate::session::DailyGamingSession;
use crate::subtasks::SubTask;
use crate::tracker::GameTracker;

pub struct GameTrackerScheduler {
    frequency: Duration,
    tracker: GameTracker,
    sub_tasks: Vec<Box<dyn SubTask>>,
}

impl GameTrackerScheduler {

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


    pub fn add_gaming_session(&mut self, session: DailyGamingSession) {
        self.tracker.add_gaming_session(session)
    }

    pub fn add(&mut self, f: Box<dyn SubTask>) -> &mut Self {
        self.sub_tasks.push(f);
        self
    }

    pub fn start(&mut self) -> Result<(), Error> {
        loop {
            // time execution
            let start = Instant::now();

            // update tracker
            self.tracker.update_time_tracker()?;

            // execute SubTasks
            for sub_task in self.sub_tasks.iter_mut() {
                sub_task.execute(&mut self.tracker)?;
            }

            // optional wait
            if let Some(remainder) = self.frequency.checked_sub(start.elapsed()) {
                thread::sleep(remainder);
            }
        }
    }
}
