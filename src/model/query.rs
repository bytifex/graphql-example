use async_graphql::Object;
use error::QueryMeError;

use crate::{
    log_location,
    sql_queries::sql_user::{error::CannotFindUserById, get_user_by_id},
    state::State,
};

use super::{scalars::id::Id, types::user::User};

pub struct Query {
    pub state: State,
}

#[Object]
impl Query {
    pub async fn me(&self) -> Result<Option<User>, QueryMeError> {
        let user_id: Id = "e30ba9c8-03bf-4ae8-af35-e8366a8fe160".into();
        Ok(Some(
            get_user_by_id(&self.state, user_id.clone())
                .await?
                .ok_or_else(|| CannotFindUserById(user_id.clone()))
                .inspect_err(|e| log::error!("{}, {e}", log_location!()))?,
        ))
    }
}

pub mod error {
    use crate::sql_queries::sql_user::error::{CannotFindUserById, GetUserByIdError};

    #[derive(Debug, thiserror::Error)]
    pub enum QueryMeError {
        #[error("QueryMeError: '{0}'")]
        GetUserByIdError(
            #[from]
            #[source]
            GetUserByIdError,
        ),

        #[error("QueryMeError: '{0}'")]
        CannotFindUserById(
            #[from]
            #[source]
            CannotFindUserById,
        ),
    }
}
