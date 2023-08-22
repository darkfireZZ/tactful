use rust_iso3166::CountryCode;

fn main() {
    println!("Hello, world!");
}

struct Contact {
    name: Name,
    birthday: Option<Date>,
    phone_numbers: Vec<PhoneNumber>,
    email_addresses: Vec<EmailAddress>,
    address: Option<Address>,
}

struct Name {
    first: String,
    last: String,
}

struct Date {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
}

struct PhoneNumber {
    number: String,
    ty: PhoneNumberType,
}

enum PhoneNumberType {
    Mobile,
    Home,
    Work
}

struct EmailAddress {
    address: String,
    ty: EmailAddressType,
}

enum EmailAddressType {
    Personal,
    Work,
}

struct Address {
    street: String,
    number: String,
    locality: String,
    postal_code: String,
    country: CountryCode,
}
