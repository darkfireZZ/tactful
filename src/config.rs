use {
    anyhow::Context,
    serde::Deserialize,
    std::{
        env,
        fs::File,
        io::{ErrorKind, Read},
        path::PathBuf,
    },
};

/// Reads the config file.
///
/// If successful, returns the parsed config file. Returns [`None`] if the config file could not be
/// found.
pub fn obtain_config() -> anyhow::Result<Option<Config>> {
    let config_path = match config_file_path() {
        Some(config_path) => config_path,
        None => return Ok(None),
    };

    match File::open(config_path) {
        Ok(mut file) => {
            let mut config = String::new();
            file.read_to_string(&mut config)
                .context("Failed to read config file")?;

            toml::from_str(&config).context("Failed to parse config file")
        }
        Err(error) => match error.kind() {
            ErrorKind::NotFound => Ok(None),
            _ => Err(error).context("Failed to open config file"),
        },
    }
}

/// Gets the path where the `tactful.toml` config file is expected.
///
/// Returns [`None`] if the config directory could not be found.
fn config_file_path() -> Option<PathBuf> {
    let mut path = config_dir()?;
    path.push("tactful.toml");
    Some(path)
}

/// Gets the config directory.
fn config_dir() -> Option<PathBuf> {
    // Return $XDG_CONFIG_HOME if it exists,
    // otherwise return $HOME/.config if it exists,
    // otherwise return None
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME").map(|home_dir| {
                let mut dir = PathBuf::from(home_dir);
                dir.push(".config");
                dir
            })
        })
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub store_path: Option<PathBuf>,
}
