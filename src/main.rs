mod configurations;
mod errors;
mod process_tree;
mod tracker;

use clap::Parser;
use regex::Regex;
use crate::tracker::GameTracker;

#[derive(Parser, PartialOrd, Eq, PartialEq)]
struct Arguments {

    /// Location of steam games
    #[clap(short, long, default_value_t = String::from("/home/x/.local/share/Steam/steamapps/common/"))]
    steam_games: String,

    /// How long should you be able to play games (in seconds)
    #[clap(long, default_value_t = 10)]
    session_duration: usize,
}


fn main() {
    let args = Arguments::parse();
    let mut tracker = GameTracker::new();
    let name_extractor = Regex::new(format!("{}([^/]+)", args.steam_games).as_str())
        .expect("Failed to create name extractor");

    tracker.update_time_tracker();
    tracker.process_tree().iter()
        .for_each( |(_, node)| println!("{}", node.to_string(0)));

    // todo : implement notifications 5 minutes before closing game
    println!("All games found : ");
    for (pid, proc_info) in tracker.scan_for_games().iter() {
        println!("{} '{}' up for {} seconds",
                 pid, proc_info.extract_game_name(&name_extractor), proc_info.run_time()
        );

        tracker.kill(proc_info)
            .expect("Failed to kill process");
    }
}
