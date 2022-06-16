use anyhow::Result;
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
pub struct PlayerOpts {
    pub avatar: String,
    pub name: String,
    pub uuid: String,
}

#[derive(Deserialize)]
pub struct Template {
    pub name: String,
    pub path: PathBuf,
    pub mode: u32,
}

#[derive(Deserialize)]
pub struct Config {
    pub players: Vec<PlayerOpts>,
    pub output_dir: PathBuf,
    pub template: Template,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        let f = File::open(path)?;
        let mut br = BufReader::new(f);
        let mut buf: Vec<u8> = Vec::new();

        br.read_to_end(&mut buf)?;
        let config: Self = toml::from_slice(&buf)?;

        Ok(config)
    }
}
