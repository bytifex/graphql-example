use clap::Parser;

use std::{path::PathBuf, str::FromStr};

use error::CannotParseSchemaSource;

#[derive(Debug, Clone)]
pub enum SchemaSource {
    File(PathBuf),
    SelfSchema,
}

impl FromStr for SchemaSource {
    type Err = CannotParseSchemaSource;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(file_path) = s.strip_prefix("file:") {
            Ok(SchemaSource::File(file_path.into()))
        } else if s == "self-schema" {
            Ok(SchemaSource::SelfSchema)
        } else {
            Err(CannotParseSchemaSource(s.into()))
        }
    }
}

#[derive(Debug, Parser)]
pub struct ServeParams {
    #[arg(
        short('l'),
        long("listener-address"),
        help("Address where the server accepts the connections (e.g., 127.0.0.1:8000)")
    )]
    pub listener_address: String,
}

#[derive(Debug, Parser)]
pub struct DiffSchemaParams {
    #[arg(help("Format: 'file:<filepath>|self-schema'"))]
    pub schema_source_left: SchemaSource,
    #[arg(help("Format: 'file:<filepath>|self-schema'"))]
    pub schema_source_right: SchemaSource,
}

#[derive(Debug, Parser)]
pub enum Commands {
    Serve(ServeParams),
    Sdl,
    DiffSchema(DiffSchemaParams),
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(
        global(true),
        short('p'),
        long("purge-db"),
        help("Purges everything from the database at start")
    )]
    pub purge_db: bool,
}

pub mod error {
    #[derive(Debug, thiserror::Error)]
    #[error("CannotParseSchemaSource: source = '{0}'")]
    pub struct CannotParseSchemaSource(pub String);
}
