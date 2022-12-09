use std::sync::Arc;

use super::entry::{CatalogEntry, DataTable};
use super::{CatalogError, CatalogSet, TableCatalogEntry};
use crate::main_entry::ClientContext;

/// The Catalog object represents the catalog of the database.
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    /// The catalog set holding the schemas
    schemas: CatalogSet,
    /// The catalog version, incremented whenever anything changes in the catalog
    catalog_version: usize,
}

impl Catalog {
    pub fn create_schema(&mut self, name: String) -> Result<(), CatalogError> {
        self.catalog_version += 1;
        let entry = CatalogEntry::default_schema_catalog_entry(self.catalog_version, name.clone());
        self.schemas.create_entry(name, entry)
    }

    pub fn create_table(
        client_context: Arc<ClientContext>,
        schema: String,
        table: String,
        data_table: DataTable,
    ) -> Result<(), CatalogError> {
        let mut catalog = match client_context.db.catalog.try_write() {
            Ok(c) => c,
            Err(_) => return Err(CatalogError::CatalogLockedError),
        };
        if let CatalogEntry::SchemaCatalogEntry(mut entry) =
            catalog.schemas.get_entry(schema.clone())?
        {
            catalog.catalog_version += 1;
            entry.create_table(catalog.catalog_version, table, data_table)?;
            catalog
                .schemas
                .replace_entry(schema, CatalogEntry::SchemaCatalogEntry(entry))?;
            return Ok(());
        }
        Err(CatalogError::CatalogEntryTypeNotMatch)
    }

    pub fn get_table(
        client_context: Arc<ClientContext>,
        schema: String,
        table: String,
    ) -> Result<TableCatalogEntry, CatalogError> {
        let catalog = match client_context.db.catalog.try_read() {
            Ok(c) => c,
            Err(_) => return Err(CatalogError::CatalogLockedError),
        };
        if let CatalogEntry::SchemaCatalogEntry(entry) = catalog.schemas.get_entry(schema)? {
            return entry.get_table(table);
        }
        Err(CatalogError::CatalogEntryTypeNotMatch)
    }
}