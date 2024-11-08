use std::sync::Arc;

pub trait SqlxErrorConverter<T> {
    fn to_sqlx_error_result(self) -> Result<T, SqlxError>;
}

impl<T> SqlxErrorConverter<T> for Result<T, sqlx::Error> {
    fn to_sqlx_error_result(self) -> Result<T, SqlxError> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => Err(SqlxError(Arc::new(e))),
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("SqlxError: '{0}'")]
pub struct SqlxError(pub Arc<sqlx::Error>);

impl From<sqlx::Error> for SqlxError {
    fn from(value: sqlx::Error) -> Self {
        Self(Arc::new(value))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseOpenError {
    #[error("DatabaseOpenError: '{0}'")]
    SqlxError(
        #[from]
        #[source]
        sqlx::Error,
    ),

    #[error("DatabaseOpenError: '{0}")]
    SqlxMigrateError(
        #[from]
        #[source]
        sqlx::migrate::MigrateError,
    ),
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("UnimplementedError: '{0}'")]
pub struct UnimplementedError(pub String);
