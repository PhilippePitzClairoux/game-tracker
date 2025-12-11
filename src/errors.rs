use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    #[error("unexpected IO error")]
    IOError(#[from] std::io::Error),

    #[error("could not deserialize toml file")]
    TOMLDeserializeError(#[from] toml::de::Error),
}