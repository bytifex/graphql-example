use error::GetUserByIdError;

use crate::{
    error::SqlxErrorConverter,
    log_location,
    model::{scalars::id::Id, types::user::User},
    state::State,
};

#[derive(sqlx::Type)]
#[allow(non_snake_case)]
pub(super) struct SqlUser {
    pub EmailAddress: Option<String>,
    pub DisplayName: String,
}

pub async fn get_user_by_id(state: &State, id: Id) -> Result<Option<User>, GetUserByIdError> {
    let id_str = id.as_string_ref();

    let record = sqlx::query_as!(
        SqlUser,
        "
            SELECT
                DisplayName, EmailAddress
            FROM
                Users
            WHERE
                Id = ?
        ",
        id_str,
    )
    .fetch_optional(state.database.connection_pool_ref())
    .await
    .to_sqlx_error_result()
    .inspect_err(|e| log::error!("{}, error = {e}", log_location!()))?;

    Ok(match record {
        Some(record) => Some(User {
            state: state.clone(),
            id: id.clone(),
            display_name: record.DisplayName,
            email_address: record.EmailAddress,
        }),
        None => None,
    })
}

pub mod error {
    use crate::{error::SqlxError, model::scalars::id::Id};

    #[derive(Clone, Debug, thiserror::Error)]
    #[error("CannotFindUserById: user_id = '{0:?}'")]
    pub struct CannotFindUserById(pub Id);

    #[derive(Clone, Debug, thiserror::Error)]
    pub enum GetUserByIdError {
        #[error("GetUserByIdError: '{0}'")]
        SqlxError(
            #[from]
            #[source]
            SqlxError,
        ),
    }
}
