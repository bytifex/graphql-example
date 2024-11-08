use std::path::Path;

use crate::{database::Database, error::DatabaseOpenError};

#[derive(Clone)]
pub struct State {
    pub database: Database,
}

impl State {
    pub async fn new(db_folder_path: impl AsRef<Path>) -> Result<Self, DatabaseOpenError> {
        Ok(Self {
            database: Database::open(db_folder_path).await?,
        })
    }
}
