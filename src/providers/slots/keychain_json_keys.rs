//! A macOS keychain generic password whose value is a JSON object. Only the named keys are
//! the slot; every other key in the item is written back unchanged.

use anyhow::{Context, Result, bail};
use serde_json::Value;
use zeroize::Zeroizing;

use super::Slot;
use super::json_merge::{merge_json_keys, pick_json_keys};
use crate::vault::macos_security::{add_generic_password, delete_generic_password, find_generic_password};

pub struct KeychainJsonKeysSlot {
    pub service: String,
    pub account: String,
    pub keys: Vec<&'static str>,
}

impl KeychainJsonKeysSlot {
    pub fn new(service: &str, account: &str, keys: &[&'static str]) -> Self {
        Self { service: service.to_string(), account: account.to_string(), keys: keys.to_vec() }
    }

    fn load(&self) -> Result<Option<Zeroizing<Vec<u8>>>> {
        Ok(find_generic_password(&self.service, &self.account)?.map(Zeroizing::new))
    }

    /// `label` is the service name, same as `KeychainItemSlot`. An empty object deletes the item.
    fn store(&self, merged: &[u8]) -> Result<()> {
        let value: Value = serde_json::from_slice(merged).context("merged credential is not JSON")?;
        match value.as_object() {
            Some(map) if map.is_empty() => delete_generic_password(&self.service, &self.account),
            Some(_) => add_generic_password(&self.service, &self.account, &self.service, merged),
            None => bail!("merged credential is not a JSON object"),
        }
    }
}

impl Slot for KeychainJsonKeysSlot {
    fn read(&self) -> Result<Option<Vec<u8>>> {
        let Some(bytes) = self.load()? else { return Ok(None) };
        pick_json_keys(&bytes, &self.keys)
    }

    fn write(&self, data: &[u8]) -> Result<()> {
        let existing = self.load()?;
        let merged = Zeroizing::new(merge_json_keys(existing.as_deref().map(Vec::as_slice), data, &self.keys)?);
        self.store(&merged)
    }

    fn clear(&self) -> Result<()> {
        let existing = self.load()?;
        if existing.is_none() {
            return Ok(());
        }
        let merged = Zeroizing::new(merge_json_keys(existing.as_deref().map(Vec::as_slice), b"{}", &self.keys)?);
        self.store(&merged)
    }

    fn describe(&self) -> String {
        format!("keychain item \"{}\" ({}) → {}", self.service, self.account, self.keys.join(", "))
    }
}
