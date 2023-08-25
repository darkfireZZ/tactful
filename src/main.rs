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
    Ok(vcard::contacts_to_vcard(writer, &contacts)?)
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

        Ok(Self {
            year: Self::parse_json_component(components[0]).with_context(error_message)?,
            month: Self::parse_json_component(components[1]).with_context(error_message)?,
            day: Self::parse_json_component(components[2]).with_context(error_message)?,
        })
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
