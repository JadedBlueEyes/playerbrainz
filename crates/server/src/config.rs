use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;

pub fn config() -> Result<Config, Box<figment::Error>> {
    let envs = [Env::var("PLAYER_CONFIG")];

    let config = envs
        .into_iter()
        .flatten()
        .map(Toml::file)
        .fold(Figment::new(), |config, file| config.merge(file.nested()))
        .merge(Env::prefixed("PLAYER_"))
        .extract()
        .map_err(Box::new)?;
    Ok(config)
}

#[derive(Deserialize, Clone)]
pub struct Config {
    // pub href: url::Url,
    #[serde(default = "database_default")]
    pub database_url: String,
    #[serde(default = "addr_default")]
    pub listen_addr: String,

    #[serde(default)]
    pub skip_initial_index: bool,
}

fn database_default() -> String {
    "sqlite://music.db?mode=rwc".to_string()
}

fn addr_default() -> String {
    "0.0.0.0:3030".to_string()
}
