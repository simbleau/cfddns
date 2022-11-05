use crate::inventory::DEFAULT_INVENTORY_PATH;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use tokio::fs;

/// The model for DNS record inventory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory(Option<HashMap<String, InventoryZone>>);

/// The model for a zone with records.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryZone(Option<HashSet<InventoryRecord>>);

/// The model for a DNS record.
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct InventoryRecord(String);

impl Inventory {
    /// Build a new inventory.
    pub fn new() -> Self {
        Self(None)
    }
    /// Read inventory from a target path.
    pub async fn from_file(path: Option<PathBuf>) -> Result<Self> {
        let mut inventory_path = path.unwrap_or(DEFAULT_INVENTORY_PATH.into());
        if !inventory_path.is_absolute() {
            inventory_path =
                inventory_path.canonicalize().with_context(|| {
                    format!(
                        "error getting canonical path to inventory file {:?}",
                        &inventory_path
                    )
                })?;
        }
        anyhow::ensure!(inventory_path.exists(), "inventory was not found");
        let cfg_bytes = fs::read(&inventory_path)
            .await
            .context("error reading inventory file")?;
        let cfg = serde_yaml::from_slice(&cfg_bytes)
            .context("error reading inventory file contents as YAML data")?;
        Ok(cfg)
    }

    /// Insert a record into the inventory.
    pub fn insert<S>(&mut self, zone_id: S, record_id: S)
    where
        S: Into<String>,
    {
        let zone_id = zone_id.into();
        let record_id = record_id.into();

        // Magic that inserts the record
        self.0
            .get_or_insert(HashMap::new())
            .entry(zone_id)
            .or_insert_with(|| InventoryZone(None))
            .0
            .get_or_insert(HashSet::new())
            .insert(InventoryRecord(record_id));
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

/// An iterator over the zone and corresponding records
pub struct InventoryIter {
    items: Vec<(String, Vec<String>)>,
    curr: usize,
}

impl Iterator for InventoryIter {
    /// A tuple containing the zone ID and a list of child record IDs
    type Item = (String, Vec<String>);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.curr;
        if current < self.items.len() {
            self.curr += 1;
            let (zone, records) = &self.items[current];
            Some((zone.clone(), records.clone()))
        } else {
            None
        }
    }
}

impl IntoIterator for Inventory {
    /// A tuple containing the zone ID and a list of child record IDs
    type Item = (String, Vec<String>);
    type IntoIter = InventoryIter;

    fn into_iter(self) -> Self::IntoIter {
        let mut items = HashMap::new();
        if let Some(map) = self.0 {
            for (key, value) in map {
                items.entry(key.to_owned()).or_insert(Vec::new());
                if let Some(record_set) = value.0 {
                    for record in record_set {
                        items.get_mut(&key).unwrap().push(record.0.clone());
                    }
                }
            }
        }
        InventoryIter {
            items: Vec::from_iter(items),
            curr: 0,
        }
    }
}
