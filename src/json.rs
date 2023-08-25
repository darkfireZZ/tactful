//! JSON representation of the contacts
// TODO expand documentation and add example

use {
    crate::{Address, Contact, Date, Name, PhoneNumber, PhoneNumberType},
    serde::{Deserialize, Serialize},
    std::io::Write,
};

// ========================================================================== //
// =====> structs to encode the structure of the JSON objects <============== //
// ========================================================================== //

#[derive(Deserialize, Serialize)]
struct JsonContact {
    name: JsonName,
    #[serde(skip_serializing_if = "Option::is_none")]
    bday: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    phone: Vec<JsonPhoneNumber>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    email: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<JsonAddress>,
}

#[derive(Deserialize, Serialize)]
struct JsonName {
    first: String,
    last: String,
}

#[derive(Deserialize, Serialize)]
struct JsonPhoneNumber {
    number: String,
    #[serde(rename = "type")]
    ty: JsonPhoneNumberType,
}

#[derive(Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum JsonPhoneNumberType {
    Mobile,
    Work,
    Home,
}

#[derive(Deserialize, Serialize)]
struct JsonAddress {
    street: String,
    number: String,
    locality: String,
    postal_code: String,
    country: String,
}

// ========================================================================== //
// =====> conversion: contacts -> JSON <===================================== //
// ========================================================================== //


pub fn contacts_to_json<W: Write>(writer: W, contacts: &[Contact]) -> anyhow::Result<()> {
    Ok(serde_json::to_writer_pretty(writer, &contacts.into_iter().map(JsonContact::from).collect::<Vec<_>>())?)
}

impl From<&Contact> for JsonContact {
    fn from(contact: &Contact) -> Self {
        Self {
            name: JsonName::from(&contact.name),
            bday: contact.birthday.as_ref().map(Date::to_json_string_repr),
            phone: contact.phone_numbers.iter().map(JsonPhoneNumber::from).collect(),
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
