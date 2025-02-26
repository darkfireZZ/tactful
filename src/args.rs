use {
    anyhow::{anyhow, bail},
    clap::{Parser, Subcommand},
    std::{env, path::PathBuf, str::FromStr},
};

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
    #[arg(short = 's', long = "store")]
    store_path: Option<PathBuf>,
}

impl Args {
    pub fn command(&self) -> &Command {
        &self.command
    }

    pub fn store_path(&self) -> anyhow::Result<PathBuf> {
        self.store_path
            .clone()
            .or_else(|| {
                env::var("HOME").ok().map(|home_dir| {
                    let mut path = PathBuf::from(home_dir);
                    path.push(".contact-store");
                    path
                })
            })
            .ok_or_else(|| anyhow!("Could not find contact store"))
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Get a list containing the next birthday of every contact, in chronological order
    Bdays,
    /// Create an iCalendar file containing the future birthdays of all contacts
    BdaysCalendar,
    /// Output contacts to STDOUT in the given format (by default vCard)
    Export {
        /// The format of the output (vcard/json)
        #[arg(short = 'f', long = "fmt", default_value = "vcard")]
        format: OutputFormat,
    },
    /// Get a list of the names of all contacts
    Names,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Json,
    Vcard,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;
    fn from_str(format: &str) -> anyhow::Result<Self> {
        Ok(match format.to_ascii_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "vcard" => OutputFormat::Vcard,
            _ => bail!("Invalid output format"),
        })
    }
}
