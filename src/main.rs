use {
    anyhow::{bail, Context},
    std::{fs::File, io::BufWriter, str::FromStr},
    country_codes::CountryCode,
};

mod json;

pub struct Contact {
    name: Name,
    birthday: Option<Date>,
    phone_numbers: Vec<PhoneNumber>,
    email_addresses: Vec<String>,
    address: Option<Address>,
}

struct Name {
    first: String,
    last: String,
}

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
            Some(u16::from_str(component).with_context(|| format!("Invalid component: \"{component}\""))?)
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
}

struct PhoneNumber {
    number: String,
    ty: PhoneNumberType,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum PhoneNumberType {
    Mobile,
    Home,
    Work
}

struct Address {
    street: String,
    number: String,
    locality: String,
    postal_code: String,
    country: CountryCode,
}
