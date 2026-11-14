//! The report documents' catalogue as this side reads it (split from
//! pack.rs at its line, release 1.10.0): the typed entries, every
//! family's `pretty`, the churn entry's pairing cap and the erase trail
//! entry's two words, read whole from the package's `document` table.

use super::leak::Leak;
use super::pack::DocEntries;
use std::collections::HashMap;

/// The report documents' catalogue: the entries this side reads,
/// whether each family's `--format json` face prints its document
/// indented (`pretty`, 7.10.0, plan v2.32 step 5 ruling R5) — a
/// catalogue fact, never a table here — the churn entry's pairing
/// cap, which the measurement uses and the console names
/// (`cochangeFileCap`), and the erase trail entry's two words: where
/// the trail lives (`log`, root-relative) and the record schema
/// (`record`) the writer stamps and the reader holds every line to.
pub struct DocCatalogue {
    entries: DocEntries,
    pretty: HashMap<String, bool>,
    pub cochange_file_cap: usize,
    pub trail_log: &'static str,
    pub trail_record: &'static str,
}

impl std::ops::Deref for DocCatalogue {
    type Target = DocEntries;
    fn deref(&self) -> &DocEntries {
        &self.entries
    }
}

impl DocCatalogue {
    /// Whether `family`'s document prints indented; None = an entry
    /// that states no `pretty` (the family is no catalogue entry).
    pub fn pretty(&self, family: &str) -> Option<bool> {
        self.pretty.get(family).copied()
    }
}

/// The catalogue read whole: the typed entries, and every family's
/// `pretty` beside them.
#[derive(serde::Deserialize)]
#[serde(try_from = "serde_json::Value")]
pub struct DocCatalogueOwned {
    entries: <DocEntries as Leak>::Owned,
    pretty: HashMap<String, bool>,
    cochange_file_cap: usize,
    trail_log: String,
    trail_record: String,
}

impl TryFrom<serde_json::Value> for DocCatalogueOwned {
    type Error = serde_json::Error;
    fn try_from(v: serde_json::Value) -> Result<Self, Self::Error> {
        let pretty = v
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(family, entry)| Some((family.clone(), entry.get("pretty")?.as_bool()?)))
            .collect();
        let cap = v["churn"]["cochangeFileCap"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok());
        let Some(cochange_file_cap) = cap else {
            return Err(serde::de::Error::custom(
                "document.churn: no cochangeFileCap",
            ));
        };
        let word = |key: &str| {
            v["erase-trail"][key]
                .as_str()
                .map(String::from)
                .ok_or_else(|| serde::de::Error::custom(format!("document.erase-trail: no {key}")))
        };
        Ok(DocCatalogueOwned {
            trail_log: word("log")?,
            trail_record: word("record")?,
            entries: serde_json::from_value(v)?,
            pretty,
            cochange_file_cap,
        })
    }
}

impl Leak for DocCatalogue {
    type Owned = DocCatalogueOwned;
    fn leak(owned: DocCatalogueOwned) -> Self {
        DocCatalogue {
            entries: DocEntries::leak(owned.entries),
            pretty: owned.pretty,
            cochange_file_cap: owned.cochange_file_cap,
            trail_log: Leak::leak(owned.trail_log),
            trail_record: Leak::leak(owned.trail_record),
        }
    }
}
