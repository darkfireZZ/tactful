use {
    crate::{Contact, PhoneNumberType},
    anyhow::Context,
    ical_vcard::{Contentline, Identifier, Param, ParamValue, Value},
    std::{io::Write, iter::IntoIterator},
};

pub fn contacts_to_vcard<'a, C, W>(writer: W, contacts: C) -> anyhow::Result<()>
where
    C: IntoIterator<Item = &'a Contact>,
    W: Write,
{
    let mut writer = ical_vcard::Writer::new(writer);
    for contact in contacts {
        writer
            .write_all(
                contact_to_contentlines(contact)
                    .context("Contact could not be serialized to vCard")?,
            )
            .context("Failed to write contacts to vCard")?;
    }
    Ok(())
}

// TODO this function is very ugly, make it cleaner. The problem is mostly due to the `ical_vcard`
// crate not being very ergonomic. All its functions return results which adds a lot of clutter and
// thus makes this function very unreadable. Solution: Add panicking variants of the functions to
// `ical_vcard`.
fn contact_to_contentlines(contact: &Contact) -> anyhow::Result<Vec<Contentline<'static>>> {
    let mut contentlines = vec![
        Contentline {
            group: None,
            name: Identifier::new("BEGIN").expect("valid identifier"),
            params: Vec::new(),
            value: Value::new("VCARD").expect("valid value"),
        },
        Contentline {
            group: None,
            name: Identifier::new("VERSION").expect("valid identifier"),
            params: Vec::new(),
            value: Value::new("4.0").expect("valid value"),
        },
        Contentline {
            group: None,
            name: Identifier::new("N").expect("valid identifier"),
            params: Vec::new(),
            value: Value::new(format!("{};{};;;", &contact.name.last, &contact.name.first))
                .context(
                    "Failed to write name to contentline because it contains control characters",
                )?,
        },
    ];

    for phone_number in &contact.phone_numbers {
        let param_value = match phone_number.ty {
            PhoneNumberType::Mobile => "cell",
            PhoneNumberType::Home => "home",
            PhoneNumberType::Work => "work",
        };
        let formatted_number = phone_number
            .number
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        contentlines.push(Contentline {
            group: None,
            name: Identifier::new("TEL").expect("valid identifier"),
            params: vec![
                Param::new(
                    Identifier::new("VALUE").expect("valid identifier"),
                    vec![ParamValue::new("uri").expect("valid parameter value")],
                )
                .expect("valid parameter"),
                Param::new(
                    Identifier::new("TYPE").expect("valid identifier"),
                    vec![ParamValue::new(param_value).expect("valid parameter value")],
                )
                .expect("valid parameter"),
            ],
            value: Value::new(format!("tel:{formatted_number}")).expect("valid value"),
        });
    }

    for email_address in &contact.email_addresses {
        contentlines.push(Contentline {
            group: None,
            name: Identifier::new("EMAIL").expect("valid identifier"),
            params: Vec::new(),
            value: Value::new(email_address.to_owned())
                .context("Failed to write email address to contentline")?,
        });
    }

    if let Some(address) = &contact.address {
        contentlines.push(Contentline {
            group: None,
            name: Identifier::new("ADR").expect("valid identifier"),
            params: Vec::new(),
            value: Value::new(format!(
                ";;{street} {number};{locality};;{postal_code};{country}",
                street = address.street,
                number = address.number,
                locality = address.locality,
                postal_code = address.postal_code,
                country = address.country.name
            ))
            .context("Failed to write address to contentline")?,
        });
    }

    if let Some(birthday) = &contact.birthday {
        contentlines.push(Contentline {
            group: None,
            name: Identifier::new("BDAY").expect("valid identifier"),
            params: Vec::new(),
            value: Value::new(
                birthday
                    .to_vcard_string_repr()
                    .context("Failed to write birthday to contentline")?,
            )
            .expect("valid value"),
        });
    }

    contentlines.push(Contentline {
        group: None,
        name: Identifier::new("END").expect("valid identifier"),
        params: Vec::new(),
        value: Value::new("VCARD").expect("valid value"),
    });

    Ok(contentlines)
}
