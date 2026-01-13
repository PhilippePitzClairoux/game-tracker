use std::collections::{BTreeMap, HashSet};
use std::env::home_dir;
use std::{fs};
use std::fs::DirEntry;
use std::io::Read;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};
use crate::errors::Errors;
use crate::process_tree::{ProcessInfo, ProcessTree};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ExpectedEntityType {
    FILE,
    DIRECTORY,
    BOTH
}

impl Default for ExpectedEntityType {
    fn default() -> Self {
        ExpectedEntityType::BOTH
    }
}

impl ExpectedEntityType {
    fn is_valid(&self, entry: &DirEntry) -> bool {
        match self {
            ExpectedEntityType::FILE => entry.path().is_file(),
            ExpectedEntityType::DIRECTORY => entry.path().is_dir(),
            ExpectedEntityType::BOTH => entry.path().is_file()||entry.path().is_dir(),
            _ => false
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlatformGames {

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

    /// Games in directory will be a EntityType (directory, file or both)
    #[serde(default)]
    expected_type: ExpectedEntityType,

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
            for entry in directories.filter_map(Result::ok).filter(|d| self.expected_type.is_valid(&d)) {
                let file_name = entry.file_name().to_string_lossy().to_string();
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

fn find_game<'a>(p: &'a ProcessInfo, games: &Games) -> Option<(String, &'a ProcessInfo)> {
    for (_, platform) in games.iter() {
        for name in platform.games.iter() {
            if let Some(process) = p.find(name) {
                return Some((name.clone(), process));
            }
        }
    }

    None
}

#[derive(Debug)]
pub struct GameTracker {
    system_processes: System,
    scanner_config: Games,
    current_proc_tree: ProcessTree,
    gametime_tracker: BTreeMap<String, HashSet<ProcessInfo>>,
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
            gametime_tracker: BTreeMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from(sys: System) -> Self {
        GameTracker {
            system_processes: sys,
            scanner_config: BTreeMap::new(),
            current_proc_tree: ProcessTree::new(),
            gametime_tracker: BTreeMap::new(),
        }
    }

    pub fn gametime_tracker(&self) -> &BTreeMap<String, HashSet<ProcessInfo>> {
        &self.gametime_tracker
    }

    pub fn get_total_time_played(&self) -> u64 {
        let mut total: u64 = 0;

        self.gametime_tracker.iter()
            .flat_map(|(_, processes)| processes.into_iter())
            .for_each(|proc| total += proc.run_time());

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

        for (_, process) in self.current_proc_tree.iter() {
            if let Some((game_name, game_process)) = find_game(process, &self.scanner_config) {
                let game_processes = self.gametime_tracker.entry(game_name)
                    .or_insert(HashSet::new());

                if game_processes.contains(&game_process) {
                    // if process is already present, remove it to insert it again (with updated runtime)
                    // ths is 100% a hack
                    game_processes.remove(&game_process);
                }

                game_processes.insert(game_process.clone());
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

