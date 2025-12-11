use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System, UpdateKind};
use crate::configurations::GlobalConfiguration;
use crate::errors::Errors;
use crate::process_tree::{ProcessInfo, ProcessTree};

#[derive(Debug)]
pub struct GameTracker {
    system_processes: System,
    scanner_config: GlobalConfiguration,
    total_game_time: u64,
    current_proc_tree: ProcessTree,
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
            scanner_config: GlobalConfiguration::new(),
            total_game_time: 0,
            current_proc_tree: ProcessTree::new()
        }
    }

    pub fn from(sys: System) -> Self {
        GameTracker {
            system_processes: sys,
            scanner_config: GlobalConfiguration::new(),
            total_game_time: 0,
            current_proc_tree: ProcessTree::new()
        }
    }

    pub fn try_from(config_path: &str) -> Result<Self, Errors> {
        let mut s = Self::new();
        s.load_config(config_path)?;

        Ok(s)
    }

    pub fn process_tree(&self) -> &ProcessTree {
        &self.current_proc_tree
    }

    pub fn load_config(&mut self, config_path: &str) -> Result<(), Errors> {
        self.scanner_config = GlobalConfiguration::try_from(config_path)?;
        Ok(())
    }

    pub fn scan_for_games(&self) -> Vec<(Pid, ProcessInfo)> {
        let mut games_found: Vec<(Pid, ProcessInfo)> = Vec::new();

        for game in self.scanner_config.get_games_config() {

            for (_, proc_info) in self.current_proc_tree.iter() {
                match proc_info.find(game.process_identifier.as_str()) {
                    Some((p, proc)) => {
                        games_found.push((p.clone(), proc.clone()))
                    },
                    None => ()
                }
            }
        }

        games_found
    }

    pub fn update_time_tracker(&mut self) {
        self.system_processes.refresh_all();
        self.current_proc_tree = ProcessTree::from(self.system_processes.processes());

        for (_, proc) in self.scan_for_games() {
            self.total_game_time += proc.run_time();
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