use std::collections::{BTreeMap, HashSet};
use std::env::home_dir;
use std::{fs};
use std::fs::DirEntry;
use std::io::Read;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};
use crate::errors::Error;
use tampering_profiler::check_tampering;
use crate::process_tree::{ProcessInfo, ProcessTree};
use crate::session::DailyGamingSession;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ExpectedEntityType {
    EXECUTABLE,
    DIRECTORY,
    BOTH
}

impl Default for ExpectedEntityType {
    fn default() -> Self {
        ExpectedEntityType::BOTH
    }
}

impl ExpectedEntityType {
    fn matches(&self, entry: &DirEntry) -> bool {
        match self {
            ExpectedEntityType::EXECUTABLE => entry.path().is_file(),
            ExpectedEntityType::DIRECTORY => entry.path().is_dir(),
            ExpectedEntityType::BOTH => entry.path().is_file()||entry.path().is_dir(),
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameLocator {

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
    search_entity_type: ExpectedEntityType,

    /// Ignore list
    #[serde(default)]
    ignore: Vec<String>,
}

fn should_be_ignored(name: &String, ignore: &Vec<String>) -> bool {
    for ig in ignore {
        if name.starts_with(ig) {
            return true;
        }
    }

    false
}

impl GameLocator {
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
            for entry in directories.filter_map(Result::ok).filter(|d| self.search_entity_type.matches(&d)) {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if !should_be_ignored(&file_name, &self.ignore) {
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

type Games = BTreeMap<String, GameLocator>;

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
    installed_games: Games,
    snapshot: ProcessTree,
    games_found: BTreeMap<String, HashSet<ProcessInfo>>,
    gaming_session: Option<DailyGamingSession>
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
            installed_games: Games::new(),
            snapshot: ProcessTree::new(),
            games_found: BTreeMap::new(),
            gaming_session: None
        }
    }

    pub fn add_gaming_session(&mut self, gaming_session: DailyGamingSession) {
        self.gaming_session = Some(gaming_session);
    }

    pub fn gametime_tracker(&self) -> &BTreeMap<String, HashSet<ProcessInfo>> {
        &self.games_found
    }

    pub fn total_time_played(&self) -> chrono::Duration {
        let mut total_seconds: u64 = 0;

        self.games_found.iter()
            .flat_map(|(_, processes)| processes.into_iter())
            .for_each(|proc| total_seconds += proc.run_time());

        chrono::Duration::seconds(total_seconds as i64)
    }

    pub fn session(&self) -> Option<&DailyGamingSession> {
        self.gaming_session.as_ref()
    }

    pub fn try_from(config_path: &str) -> Result<Self, Error> {
        let mut s = Self::new();
        s.load_config(config_path)?;

        Ok(s)
    }

    #[check_tampering]
    pub fn load_config(&mut self, config_path: &str) -> Result<(), Error> {
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

                self.installed_games = config;
                Ok(())
            },
            Err(e) => Err(e.into())
        }
    }

    #[check_tampering]
    pub fn update_time_tracker(&mut self) -> Result<(), Error> {
        self.system_processes.refresh_all();
        self.snapshot = ProcessTree::from(self.system_processes.processes());

        for (_, process) in self.snapshot.iter() {
            if let Some((game_name, game_process)) = find_game(process, &self.installed_games) {
                let game_processes = self.games_found.entry(game_name)
                    .or_insert(HashSet::new());

                if game_processes.contains(&game_process) {
                    // if process is already present, remove it to insert it again (with updated runtime)
                    // ths is 100% a hack
                    game_processes.remove(&game_process);
                }

                game_processes.insert(game_process.clone());
            }
        }

        let time_played = self.total_time_played();
        if let Some(gaming_session_tracker) = self.gaming_session.as_mut() {
            if gaming_session_tracker.is_passed_midnight() {
                gaming_session_tracker.restart_session()?;
            }

            if gaming_session_tracker.is_session_over(time_played) {
                gaming_session_tracker.end_session();
            }
        }

        Ok(())
    }

    #[check_tampering]
    pub fn kill(&self, p: &ProcessInfo) -> Result<bool, Error> {
        match self.system_processes.process(p.pid()) {
            Some(p) => Ok(p.kill()),
            None => Ok(true)
        }
    }
}

