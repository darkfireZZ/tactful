use {
    anyhow::{anyhow, bail, Context},
    chrono::Datelike,
    clap::{Parser, Subcommand},
    country_codes::CountryCode,
    std::{
        fs::File,
        io::{self, BufReader, BufWriter, Write},
        path::{Path, PathBuf},
        str::FromStr,
    },
};

mod json;
mod vcard;
mod config;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = config::obtain_config()?;

    let contacts_path = args.store_path.or_else(|| config?.store_path).ok_or(anyhow!("Could not determine store path"))?;
    let contacts = obtain_contacts(&contacts_path)?;

    match args.command {
        Command::Bdays => {
            let today = Date::today();
            let mut bday_items = contacts
                .into_iter()
                .filter_map(|contact| {
                    let bday = contact.birthday.as_ref()?;
                    // Note that if bday is on the 29th February, `next_bday` may NOT represent a valid
                    // date. However, it should still be displayed. I don't want to miss any birthdays
                    // after all.
                    let next_bday = match (bday.month, bday.day) {
                        (Some(month), Some(day)) => {
                            let bday_this_year = Date {
                                year: today.year,
                                month,
                                day,
                            };

                            if bday_this_year >= today {
                                bday_this_year
                            } else {
                                Date {
                                    year: today.year + 1,
                                    month,
                                    day,
                                }
                            }
                        }
                        _ => return None,
                    };

                    Some(BdayItem { next_bday, contact })
                })
                .collect::<Vec<_>>();
            bday_items.sort_unstable_by_key(|item| item.next_bday);

            let mut writer = BufWriter::new(io::stdout());
            for item in bday_items {
                writeln!(
                    &mut writer,
                    "{year:04}-{month:02}-{day:02} {first_name} {last_name}",
                    year = item.next_bday.year,
                    month = item.next_bday.month,
                    day = item.next_bday.day,
                    first_name = item.contact.name.first,
                    last_name = item.contact.name.last,
                )?;
            }

            Ok(())
        }
        Command::Export { format } => {
            let writer = BufWriter::new(io::stdout());

            match format {
                OutputFormat::Json => json::contacts_to_json(writer, &contacts),
                OutputFormat::Vcard => vcard::contacts_to_vcard(writer, &contacts),
            }
        }
        Command::Names => {
            let mut writer = BufWriter::new(io::stdout());

            for contact in contacts {
                writeln!(&mut writer, "{} {}", contact.name.first, contact.name.last)?;
            }

            Ok(())
        }
    }
}

fn obtain_contacts(store_path: &Path) -> anyhow::Result<Vec<Contact>> {
    let contacts_store = File::open(store_path).context("Failed to open contacts store")?;
    json::contacts_from_json(BufReader::new(contacts_store))
}

#[derive(Debug)]
struct BdayItem {
    next_bday: Date,
    contact: Contact,
}

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
    #[arg(short = 's', long = "store")]
    store_path: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Get a list containing the next birthday of every contact, in chronological order
    Bdays,
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
enum OutputFormat {
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

#[derive(Debug)]
pub struct Contact {
    name: Name,
    birthday: Option<PartialDate>,
    phone_numbers: Vec<PhoneNumber>,
    email_addresses: Vec<String>,
    address: Option<Address>,
}

#[derive(Debug)]
struct Name {
    first: String,
    last: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Date {
    year: u16,
    month: u16,
    day: u16,
}

impl Date {
    fn today() -> Self {
        let today = chrono::Local::now().naive_local().date();
        Self {
            year: u16::try_from(today.year())
                .expect("This program will not be executed after the year 65535"),
            month: u16::try_from(today.month()).expect("month <= 12"),
            day: u16::try_from(today.day()).expect("day <= 31"),
        }
    }
}

/// A partial date: Year, month and day are optional.
///
/// All functions and structs that take [`PartialDate`]s assume that the date is valid. All
/// functions that produce [`PartialDate`]s only produce valid dates. Use [`PartialDate::validate`]
/// to validate dates.
#[derive(Debug)]
struct PartialDate {
    year: Option<u16>,
    month: Option<u16>,
    day: Option<u16>,
}

impl PartialDate {
    fn is_leap_year(year: u16) -> bool {
        ((year % 4 == 0) && (year % 100 != 0)) || (year % 400 == 0)
    }

    fn max_days_in_month(month: Option<u16>, year: Option<u16>) -> u16 {
        match month {
            None => 31,
            Some(month) => match month {
                1 => 31,
                2 => year.map_or(29, |year| if Self::is_leap_year(year) { 29 } else { 28 }),
                3 => 31,
                4 => 30,
                5 => 31,
                6 => 30,
                7 => 31,
                8 => 31,
                9 => 30,
                10 => 31,
                11 => 30,
                12 => 31,
                _ => unreachable!("this function expects to receive a valid month"),
            },
        }
    }

    fn validate(&self) -> anyhow::Result<()> {
        if let Some(month) = self.month {
            if month == 0 || month > 12 {
                bail!("Invalid month: {}", month)
            }
        }

        if let Some(day) = self.day {
            if day == 0 || day > Self::max_days_in_month(self.month, self.year) {
                bail!("Invalid day: {}", day)
            }
        }

        Ok(())
    }

    fn to_json_string_repr(&self) -> String {
        let year = match self.year {
            Some(year) => year.to_string(),
            None => "".to_owned(),
        };
        let month = match self.month {
            Some(month) => format!("{:02}", month),
            None => "".to_owned(),
        };
        let day = match self.day {
            Some(day) => format!("{:02}", day),
            None => "".to_owned(),
        };

        format!("{year}-{month}-{day}")
    }

    fn parse_json_component(component: &str) -> anyhow::Result<Option<u16>> {
        Ok(if component.is_empty() {
            None
        } else {
            Some(
                u16::from_str(component)
                    .with_context(|| format!("Invalid component: \"{component}\""))?,
            )
        })
    }

    fn from_json_string_repr(string_repr: &str) -> anyhow::Result<Self> {
        let error_message = || format!("Invalid date format: \"{string_repr}\"");

        let components = string_repr.split('-').collect::<Vec<_>>();

        if components.len() != 3 {
            bail!(error_message());
        }

        let date = Self {
            year: Self::parse_json_component(components[0]).with_context(error_message)?,
            month: Self::parse_json_component(components[1]).with_context(error_message)?,
            day: Self::parse_json_component(components[2]).with_context(error_message)?,
        };

        date.validate()
            .with_context(|| format!("Invalid date \"{string_repr}\""))?;

        Ok(date)
    }

    fn to_vcard_string_repr(&self) -> anyhow::Result<String> {
        if let Some(year) = self.year {
            if year > 9999 {
                bail!("Years greater than 9999 cannot be represented in vCard version 4.0");
            }
        }

        Ok(match (self.year, self.month, self.day) {
            (None, None, Some(day)) => format!("---{day:02}"),
            (None, Some(month), None) => format!("--{month:02}"),
            (None, Some(month), Some(day)) => format!("--{month:02}{day:02}"),
            (Some(year), None, None) => format!("{year:04}"),
            (Some(year), Some(month), None) => format!("{year:04}-{month:02}"),
            (Some(year), Some(month), Some(day)) => format!("{year:04}{month:02}{day:02}"),
            (None, None, None) | (Some(_), None, Some(_)) => {
                bail!("Date cannot be represented in vCard version 4.0")
            }
        })
    }
}

impl From<Date> for PartialDate {
    fn from(date: Date) -> Self {
        Self {
            year: Some(date.year),
            month: Some(date.month),
            day: Some(date.day),
        }
    }
}

/// Represents a telephone number.
///
/// All functions and structs that take [`PhoneNumber`]s assume that the phone number is valid. All
/// functions that produce [`PhoneNumber`]s only produce valid phone numbers. Use
/// [`PhoneNumber::validate`] to validate phone numbers.
#[derive(Debug)]
struct PhoneNumber {
    number: String,
    ty: PhoneNumberType,
}

impl PhoneNumber {
    /// Checks if this phone number is valid.
    ///
    /// This is very rudimentary and does not check a great many things.
    fn validate(&self) -> anyhow::Result<()> {
        let chars = self
            .number
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<Vec<_>>();

        match chars.first() {
            Some(&first_char) => {
                if !(first_char.is_ascii_digit() || first_char == '+') {
                    bail!("Phone number must begin with a digit or a '+'");
                }
            }
            None => {
                bail!("Phone number cannot be empty");
            }
        }

        if !chars.iter().skip(1).all(char::is_ascii_digit) {
            bail!("All except the first character of a phone number must be digits");
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PhoneNumberType {
    Mobile,
    Home,
    Work,
}

#[derive(Debug)]
struct Address {
    street: String,
    number: String,
    locality: String,
    postal_code: String,
    country: CountryCode,
}
