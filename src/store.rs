use {
    crate::{json, Contact},
    anyhow::Context,
    std::{fs::File, io::BufReader, path::Path},
};

#[derive(Debug)]
pub struct ContactStore {
    contacts: Vec<Contact>,
}

impl ContactStore {
    /// Initialize a store located at the given path
    pub fn from_path<P: AsRef<Path>>(store_path: P) -> anyhow::Result<Self> {
        let contacts_path = store_path.as_ref().join("contacts.json");
        let contacts_file = File::open(&contacts_path).with_context(|| {
            format!(
                "Failed to open contact store at {}",
                contacts_path.display()
            )
        })?;
        let contacts = json::contacts_from_json(BufReader::new(contacts_file))?;
        Ok(ContactStore { contacts })
    }

    pub fn contacts(&self) -> impl Iterator<Item = &Contact> {
        self.contacts.iter()
    }
}
