use {
    anyhow::Context,
    serde::Deserialize,
    std::{
        fs::File,
        io::{ErrorKind, Read},
        path::PathBuf,
    },
};

const CONFIG_PATH: &'static str = "/Users/darkfire/.config/tactful.toml";

/// Reads the config file.
///
/// If successful, returns the parsed config file. Returns [`None`] if the config file could not be
/// found.
pub fn obtain_config() -> anyhow::Result<Option<Config>> {
    match File::open(CONFIG_PATH) {
        Ok(mut file) => {
            let mut config = String::new();
            file.read_to_string(&mut config).context("Failed to read config file")?;

            toml::from_str(&config).context("Failed to parse config file")
        },
        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => Ok(None),
                _ => Err(error).context("Failed to open config file"),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub store_path: Option<PathBuf>,
}
