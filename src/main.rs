use {
    anyhow::{bail, Context},
    country_codes::CountryCode,
    std::{
        fs::File,
        io::{BufReader, BufWriter},
        str::FromStr,
    },
};

fn main() -> anyhow::Result<()> {
    let json_path = "./contacts.json";
    let json_file = File::open(json_path)?;
    let contacts = json::contacts_from_json(BufReader::new(json_file))?;

    let vcf_path = ".test.vcf";
    let vcf_file = File::create(vcf_path)?;
    let writer = BufWriter::new(vcf_file);
    vcard::contacts_to_vcard(writer, &contacts)
}

mod json;
mod vcard;

#[derive(Debug)]
pub struct Contact {
    name: Name,
    birthday: Option<Date>,
    phone_numbers: Vec<PhoneNumber>,
    email_addresses: Vec<String>,
    address: Option<Address>,
}

#[derive(Debug)]
struct Name {
    first: String,
    last: String,
}

#[derive(Debug)]
struct Date {
    year: Option<u16>,
    month: Option<u16>,
    day: Option<u16>,
}

impl Date {
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
            .filter(|c| c.is_whitespace())
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
