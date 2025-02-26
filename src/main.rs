use {
    crate::args::{Args, Command, OutputFormat},
    anyhow::{bail, Context},
    chrono::{Datelike, Timelike},
    clap::Parser,
    country_codes::CountryCode,
    ical::{Calendar, Event, RecurrenceFrequency, RecurrenceRule, StartDateTime},
    std::{
        io::{self, BufWriter, Write},
        str::FromStr,
    },
    store::ContactStore,
};

mod args;
mod json;
mod store;
mod vcard;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let store_path = args.store_path()?;
    let store = ContactStore::from_path(store_path)?;

    match args.command() {
        Command::Bdays => {
            let today = Date::today();
            let mut bday_items = store
                .contacts()
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

                    Some(BdayItem {
                        next_bday,
                        contact: contact.clone(),
                    })
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
        Command::BdaysCalendar => {
            let mut calendar = Calendar::new();
            calendar.set_product_identifier(concat!(
                "nicolabruhin.com ",
                env!("CARGO_PKG_NAME"),
                " ",
                env!("CARGO_PKG_VERSION")
            ));
            for contact in store.contacts() {
                let Some(bday) = &contact.birthday else {
                    continue;
                };
                let (Some(month), Some(day)) = (bday.month, bday.day) else {
                    continue;
                };
                let month = month as u8;
                let day = day as u8;
                let now = chrono::Local::now();
                let now_ical = ical::DateTime {
                    date: ical::Date::new(now.year() as u16, now.month() as u8, now.day() as u8),
                    time: ical::Time::new_utc(
                        now.hour() as u8,
                        now.minute() as u8,
                        now.second() as u8,
                    ),
                };
                if let Some(year) = bday.year {
                    // If we know the year of birth, we can add the age to the summary.
                    let mut date = ical::Date::new(year, month, day);
                    // People usually don't live longer than 150 years.
                    for age in 0..150 {
                        let mut event = Event::new(StartDateTime::from(date), now_ical);
                        event.set_summary(format!(
                            "{} {} ({age})",
                            contact.name.first, contact.name.last
                        ));
                        calendar.add_component(event);
                        date.set_year(date.year() + 1);
                        // Not adding events after 10 years in the future saves space.
                        if date.year() > now_ical.date.year() + 10 {
                            break;
                        }
                    }
                } else {
                    // If we don't know the year of birth, we simply add a recurring event starting
                    // from the previous year.
                    let start_date =
                        StartDateTime::from(ical::Date::new(now_ical.date.year() - 1, month, day));
                    let mut event = Event::new(start_date, now_ical);
                    event.set_summary(format!("{} {}", contact.name.first, contact.name.last));
                    event.set_recurrence_rule(RecurrenceRule::new(RecurrenceFrequency::Yearly));
                    calendar.add_component(event);
                }
            }
            let writer = BufWriter::new(io::stdout());
            calendar.write(writer).context("Failed to write calendar")?;
            Ok(())
        }
        Command::Export { format } => {
            let writer = BufWriter::new(io::stdout());

            match format {
                OutputFormat::Json => json::contacts_to_json(writer, store.contacts()),
                OutputFormat::Vcard => vcard::contacts_to_vcard(writer, store.contacts()),
            }
        }
        Command::Names => {
            let mut writer = BufWriter::new(io::stdout());

            for contact in store.contacts() {
                writeln!(&mut writer, "{} {}", contact.name.first, contact.name.last)?;
            }

            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
struct BdayItem {
    next_bday: Date,
    contact: Contact,
}

#[derive(Clone, Debug)]
pub struct Contact {
    name: Name,
    birthday: Option<PartialDate>,
    phone_numbers: Vec<PhoneNumber>,
    email_addresses: Vec<String>,
    address: Option<Address>,
}

#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
struct Address {
    street: String,
    number: String,
    locality: String,
    postal_code: String,
    country: CountryCode,
}
