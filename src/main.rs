mod errors;
mod process_tree;
mod tracker;

use clap::Parser;
use crate::tracker::GameTracker;

#[derive(Parser, PartialOrd, Eq, PartialEq)]
struct Arguments {

    /// How long should you be able to play games (in seconds)
    #[clap(long, default_value_t = 10)]
    session_duration: usize,
}

fn main() {
    let args = Arguments::parse();
    let mut tracker = GameTracker::new();
    tracker.load_config("/home/x/RustroverProjects/game-tracker/configs/linux.toml")
        .expect("Failed to load config");

    tracker.update_time_tracker();

    println!("All games found : ");
    for (process, game_name) in tracker.gametime_tracker().iter() {
        println!("{} '{}' up for {} seconds",
                 process.pid(), game_name, process.run_time()
        );

        tracker.kill(process)
            .expect("Failed to kill process");
    }
}
