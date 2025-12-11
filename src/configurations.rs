use std::fs;
use std::io::Read;
use serde::{Deserialize, Serialize};
use crate::errors::Errors;

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfiguration {
    games: Vec<Game>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub name: String,
    pub process_identifier: String,
    pub game_name_regex_extractor: String,
}

impl GlobalConfiguration {

    pub fn new() -> GlobalConfiguration {
        Self {
            games: Vec::new(),
        }
    }

    pub fn try_from(path: &str) -> Result<Self, Errors> {
        let mut file = fs::File::open(path)?;
        let mut buffer = String::new();

        file.read_to_string(&mut buffer)?;
        match toml::from_str::<Self>(&buffer) {
            Ok(config) => Ok(config),
            Err(e) => Err(e.into())
        }
    }

    pub fn get_games_config(&self) -> &Vec<Game> {
        &self.games
    }

}