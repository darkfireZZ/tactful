//! JSON representation of the contacts
//!
//! This module contains the code that serializes contacts to and deserializes them from a JSON
//! representation.

use {
    crate::{Address, Contact, Name, PartialDate, PhoneNumber, PhoneNumberType},
    anyhow::Context,
    serde::{Deserialize, Serialize},
    std::io::{BufReader, Read, Write},
};

// ========================================================================== //
// =====> structs to encode the structure of the JSON objects <============== //
// ========================================================================== //

#[derive(Debug, Deserialize, Serialize)]
struct JsonContact {
    name: JsonName,
    #[serde(skip_serializing_if = "Option::is_none")]
    bday: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    phone: Vec<JsonPhoneNumber>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    email: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<JsonAddress>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonName {
    first: String,
    last: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonPhoneNumber {
    number: String,
    #[serde(rename = "type")]
    ty: JsonPhoneNumberType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum JsonPhoneNumberType {
    Mobile,
    Work,
    Home,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonAddress {
    street: String,
    number: String,
    locality: String,
    postal_code: String,
    country: String,
}

// ========================================================================== //
// =====> serialization <==================================================== //
// ========================================================================== //

pub fn contacts_to_json<'a, C: Iterator<Item = &'a Contact>, W: Write>(
    writer: W,
    contacts: C,
) -> anyhow::Result<()> {
    Ok(serde_json::to_writer(
        writer,
        &contacts.map(JsonContact::from).collect::<Vec<_>>(),
    )?)
}

impl From<&Contact> for JsonContact {
    fn from(contact: &Contact) -> Self {
        Self {
            name: JsonName::from(&contact.name),
            bday: contact
                .birthday
                .as_ref()
                .map(PartialDate::to_json_string_repr),
            phone: contact
                .phone_numbers
                .iter()
                .map(JsonPhoneNumber::from)
                .collect(),
            email: contact.email_addresses.clone(),
            address: contact.address.as_ref().map(JsonAddress::from),
        }
    }
}

impl From<&Name> for JsonName {
    fn from(name: &Name) -> Self {
        Self {
            first: name.first.to_owned(),
            last: name.last.to_owned(),
        }
    }
}

impl From<&PhoneNumber> for JsonPhoneNumber {
    fn from(phone_number: &PhoneNumber) -> Self {
        Self {
            number: phone_number.number.to_owned(),
            ty: phone_number.ty.into(),
        }
    }
}

impl From<PhoneNumberType> for JsonPhoneNumberType {
    fn from(phone_number_type: PhoneNumberType) -> Self {
        match phone_number_type {
            PhoneNumberType::Mobile => JsonPhoneNumberType::Mobile,
            PhoneNumberType::Work => JsonPhoneNumberType::Work,
            PhoneNumberType::Home => JsonPhoneNumberType::Home,
        }
    }
}

impl From<&Address> for JsonAddress {
    fn from(address: &Address) -> Self {
        Self {
            street: address.street.to_owned(),
            number: address.number.to_owned(),
            locality: address.locality.to_owned(),
            postal_code: address.postal_code.to_owned(),
            country: address.country.alpha2.to_owned(),
        }
    }
}

// ========================================================================== //
// =====> deserialization <================================================== //
// ========================================================================== //

pub fn contacts_from_json<R: Read>(reader: R) -> anyhow::Result<Vec<Contact>> {
    let json_contacts: Vec<JsonContact> = serde_json::from_reader(BufReader::new(reader))?;
    json_contacts
        .into_iter()
        .map(Contact::try_from)
        .collect::<anyhow::Result<Vec<_>>>()
        .context("Failed to parse contact store")
}

impl TryFrom<JsonContact> for Contact {
    type Error = anyhow::Error;
    fn try_from(json_contact: JsonContact) -> anyhow::Result<Self> {
        let error_message = || {
            format!(
                "Failed to parse contact \"{} {}\"",
                &json_contact.name.first, &json_contact.name.last
            )
        };

        Ok(Contact {
            name: Name::from(&json_contact.name),
            birthday: json_contact
                .bday
                .map(|date| PartialDate::from_json_string_repr(&date))
                .transpose()
                .with_context(error_message)?,
            phone_numbers: json_contact
                .phone
                .into_iter()
                .map(PhoneNumber::try_from)
                .collect::<Result<Vec<_>, _>>()
                .with_context(error_message)?,
            email_addresses: json_contact.email,
            address: json_contact
                .address
                .map(Address::try_from)
                .transpose()
                .with_context(error_message)?,
        })
    }
}

impl From<&JsonName> for Name {
    fn from(json_name: &JsonName) -> Self {
        Name {
            first: json_name.first.to_owned(),
            last: json_name.last.to_owned(),
        }
    }
}

impl TryFrom<JsonPhoneNumber> for PhoneNumber {
    type Error = anyhow::Error;
    fn try_from(json_phone_number: JsonPhoneNumber) -> anyhow::Result<Self> {
        let phone_number = PhoneNumber {
            number: json_phone_number.number,
            ty: PhoneNumberType::from(json_phone_number.ty),
        };

        phone_number
            .validate()
            .context("Failed to parse phone number")?;

        Ok(phone_number)
    }
}

impl From<JsonPhoneNumberType> for PhoneNumberType {
    fn from(json_phone_number_type: JsonPhoneNumberType) -> Self {
        match json_phone_number_type {
            JsonPhoneNumberType::Mobile => PhoneNumberType::Mobile,
            JsonPhoneNumberType::Work => PhoneNumberType::Work,
            JsonPhoneNumberType::Home => PhoneNumberType::Home,
        }
    }
}

impl TryFrom<JsonAddress> for Address {
    type Error = anyhow::Error;
    fn try_from(json_address: JsonAddress) -> anyhow::Result<Self> {
        Ok(Address {
            street: json_address.street,
            number: json_address.number,
            locality: json_address.locality,
            postal_code: json_address.postal_code,
            country: country_codes::from_alpha2(&json_address.country)
                .context("Failed to parse address")?,
        })
    }
}
