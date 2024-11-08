use error::{CannotConvertSqlCyborgToCyborg, GetCyborgByIdError};

use crate::{
    error::SqlxErrorConverter,
    log_location,
    model::{enums::character_race::CharacterRace, scalars::id::Id, types::cyborg::Cyborg},
    state::State,
};

use super::sql_character::error::CharacterRaceMismatchError;

#[derive(sqlx::Type)]
#[allow(non_snake_case)]
pub(super) struct SqlCyborg {
    pub Id: Id,
    pub Name: Option<String>,
    pub Nickname: String,
    pub Race: String,
}

pub async fn get_cyborg_by_id(
    state: &State,
    user_id: &Id,
    character_id: &Id,
) -> Result<Option<Cyborg>, GetCyborgByIdError> {
    let user_id_str = user_id.as_string_ref();
    let character_id_str = character_id.as_string_ref();

    let record = sqlx::query_as!(
        SqlCyborg,
        "
            SELECT
                Characters.Id as Id,
                Characters.Name as Name,
                Characters.Nickname as Nickname,
                Characters.Race as Race
            FROM
                Characters
                    JOIN Cyborgs ON Characters.Id = Cyborgs.Id
            WHERE
                Characters.UserId = ? AND Characters.Id = ?
        ",
        user_id_str,
        character_id_str,
    )
    .fetch_optional(state.database.connection_pool_ref())
    .await
    .to_sqlx_error_result()
    .inspect_err(|e| log::error!("{}, error = {e}", log_location!()))?;

    Ok(record
        .map(|record| record.try_into_cyborg(state))
        .transpose()
        .inspect_err(|e| log::error!("{}, error = {e}", log_location!()))?)
}

impl SqlCyborg {
    pub fn try_into_cyborg(self, _state: &State) -> Result<Cyborg, CannotConvertSqlCyborgToCyborg> {
        const EXPECTED_CHARACTER_RACE: CharacterRace = CharacterRace::Cyborg;

        let character_race = self.Race.parse()?;

        match character_race {
            EXPECTED_CHARACTER_RACE => Ok(Cyborg {
                id: self.Id,
                name: self.Name,
                nickname: self.Nickname,
                race: character_race,
            }),
            _ => Err(CharacterRaceMismatchError {
                stored: character_race,
                expected: EXPECTED_CHARACTER_RACE,
            }
            .into()),
        }
    }
}

pub mod error {
    use crate::{
        error::SqlxError, model::enums::character_race::error::InvalidCharacterRace,
        sql_queries::sql_character::error::CharacterRaceMismatchError,
    };

    #[derive(Clone, Debug, thiserror::Error)]
    pub enum CannotConvertSqlCyborgToCyborg {
        #[error("CannotConvertSqlCyborgToCyborg: '{0}'")]
        SqlxError(
            #[from]
            #[source]
            SqlxError,
        ),

        #[error("CannotConvertSqlCyborgToCyborg: '{0}'")]
        InvalidCharacterRace(
            #[from]
            #[source]
            InvalidCharacterRace,
        ),

        #[error("CannotConvertSqlCyborgToCyborg: '{0}'")]
        CharacterRaceMismatchError(
            #[from]
            #[source]
            CharacterRaceMismatchError,
        ),
    }

    #[derive(Clone, Debug, thiserror::Error)]
    pub enum GetCyborgByIdError {
        #[error("GetCyborgByIdError: '{0}'")]
        SqlxError(
            #[from]
            #[source]
            SqlxError,
        ),

        #[error("GetCyborgByIdError: '{0}'")]
        CannotConvertSqlCyborgToCyborg(
            #[from]
            #[source]
            CannotConvertSqlCyborgToCyborg,
        ),
    }
}
