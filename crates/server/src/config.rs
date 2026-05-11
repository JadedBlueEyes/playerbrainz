use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;
use snafu::ResultExt;

pub fn config() -> Result<Config, ConfigError> {
    let envs = [Env::var("PLAYER_CONFIG")];

    let mut config = envs
        .into_iter()
        .flatten()
        .map(Toml::file)
        .fold(Figment::new(), |config, file| config.merge(file.nested()))
        .merge(Env::prefixed("PLAYER_"))
        .extract()
        .map_err(Box::new)
        .context(FigmentSnafu)?;
    check(&mut config)?;
    Ok(config)
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub href: url::Url,

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

#[derive(Debug, snafu::Snafu)]
pub enum ConfigError {
    #[snafu(display("Configured server href '{}' cannot be a base URL", href))]
    HrefCannotBeBase { href: url::Url },
    #[snafu(display("{source}"))]
    Figment { source: Box<figment::Error> },
}

fn check(config: &mut Config) -> Result<(), ConfigError> {
    if config.href.cannot_be_a_base() {
        HrefCannotBeBaseSnafu {
            href: config.href.clone(),
        }
        .fail()?
    }

    config.href.set_path("/");
    config.href.set_query(None);
    config.href.set_fragment(None);

    Ok(())
}
