use std::collections::{BTreeMap, HashMap};
use std::env::home_dir;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};
use crate::errors::Errors;
use crate::process_tree::{ProcessInfo, ProcessTree};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct  PlatformGames {

    /// Platform name
    #[serde(skip)]
    name: String,

    /// name of games found!
    #[serde(skip)]
    games: Vec<String>,

    /// Paths to search in home directory
    #[serde(default)]
    home_paths: Vec<PathBuf>,

    /// Absolute path(s) to search for games
    #[serde(default)]
    absolute_paths: Vec<PathBuf>,

    /// Ignore list
    #[serde(default)]
    ignore: Vec<String>,
}

fn ignore_file(name: &String, ignore: &Vec<String>) -> bool {
    for ig in ignore {
        if name.starts_with(ig) {
            return true;
        }
    }

    false
}

impl PlatformGames {
    fn load(&mut self) {
        match home_dir() {
            Some(home) => {
                self.load_home_game_names(&home)
            },
            None => println!(
                "Could not find home directory - cannot load {:?} home paths...",
                self.home_paths
            ),
        }

        self.absolute_paths.clone().iter()
            .for_each(|p| self.load_game_names_from_path(p));
    }

    fn load_game_names_from_path(&mut self, p: &PathBuf) {
        if let Ok(directories) = fs::read_dir(p) {
            for dir in directories.filter_map(Result::ok).filter(|d| d.path().is_dir()) {
                let file_name = dir.file_name().to_string_lossy().to_string();
                if !ignore_file(&file_name, &self.ignore) {
                    self.games.push(file_name);
                }
            }
        }
    }

    fn load_home_game_names(&mut self, base_path: &PathBuf) {
        self.home_paths.clone().iter().for_each(|p| {
            self.load_game_names_from_path(&base_path.join(p))
        });
    }

}

type Games = BTreeMap<String, PlatformGames>;

fn find_game<'a, 'b>(p: &'b ProcessInfo, games: &'a Games) -> Option<(String, &'a PlatformGames, ProcessInfo)> {
    for (_, platform) in games.iter() {
        for name in platform.games.iter() {
            if let Some((_, process)) = p.find(name) {
                return Some((name.clone(), platform, process.clone()));
            }
        }
    }

    None
}

#[derive(Debug)]
pub struct GameTracker{
    system_processes: System,
    scanner_config: Games,
    current_proc_tree: ProcessTree,
    gametime_tracker: HashMap<String, ProcessInfo>,
}

impl GameTracker {

    pub fn new() -> Self {
        GameTracker {
            system_processes:
            System::new_with_specifics(
                RefreshKind::nothing()
                    .with_processes(
                        ProcessRefreshKind::nothing()
                            .with_cmd(UpdateKind::OnlyIfNotSet)
                            .with_cwd(UpdateKind::OnlyIfNotSet)
                            .with_exe(UpdateKind::OnlyIfNotSet)
                            .with_user(UpdateKind::OnlyIfNotSet)
            )),
            scanner_config: BTreeMap::new(),
            current_proc_tree: ProcessTree::new(),
            gametime_tracker: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from(sys: System) -> Self {
        GameTracker {
            system_processes: sys,
            scanner_config: BTreeMap::new(),
            current_proc_tree: ProcessTree::new(),
            gametime_tracker: HashMap::new(),
        }
    }

    pub fn gametime_tracker(&self) -> &HashMap<String, ProcessInfo> {
        &self.gametime_tracker
    }

    pub fn get_total_time_played(&self) -> u64 {
        let mut total: u64 = 0;
        for game in self.gametime_tracker.values() {
            total += game.run_time()
        }

        total
    }

    #[allow(dead_code)]
    pub fn try_from(config_path: &str) -> Result<Self, Errors> {
        let mut s = Self::new();
        s.load_config(config_path)?;

        Ok(s)
    }

    #[allow(dead_code)]
    pub fn process_tree(&self) -> &ProcessTree {
        &self.current_proc_tree
    }

    pub fn load_config(&mut self, config_path: &str) -> Result<(), Errors> {
        let mut file = fs::File::open(config_path)?;
        let mut buffer = vec![];

        file.read_to_end(buffer.as_mut())?;
        match toml::from_slice::<Games>(&buffer) {
            Ok(mut config) => {
                config.iter_mut()
                    .for_each(|(platform_name, platform)| {
                        platform.name = platform_name.clone();
                        platform.load();
                });

                self.scanner_config = config;
                Ok(())
            },
            Err(e) => Err(e.into())
        }
    }

    pub fn update_time_tracker(&mut self) {
        self.system_processes.refresh_all();
        self.current_proc_tree = ProcessTree::from(self.system_processes.processes());

        for (_, proc_info) in self.current_proc_tree.iter() {
            if let Some((game_name, _, process_info)) = find_game(proc_info, &self.scanner_config) {
                self.gametime_tracker.entry(game_name).insert_entry(process_info);
            }
        }
    }


    pub fn kill(&self, p: &ProcessInfo) -> Result<bool, Errors> {

        match p.children() {
            Some(children) => {
                children.iter().for_each(|(_, child)| {
                    self.kill(child).expect("could not kill child process");
                })
            }
            None => ()
        }

        match self.system_processes.process(p.pid()) {
            Some(p) => Ok(p.kill()),
            None => Ok(true)
        }
    }
}