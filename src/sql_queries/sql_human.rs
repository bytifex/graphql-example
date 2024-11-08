use error::{CannotConvertSqlHumanToHuman, GetHumanByIdError};

use crate::{
    error::SqlxErrorConverter,
    log_location,
    model::{enums::character_race::CharacterRace, scalars::id::Id, types::human::Human},
    state::State,
};

use super::sql_character::error::CharacterRaceMismatchError;

#[derive(sqlx::Type)]
#[allow(non_snake_case)]
pub(super) struct SqlHuman {
    pub Id: Id,
    pub Name: Option<String>,
    pub Nickname: String,
    pub Race: String,
}

pub async fn get_human_by_id(
    state: &State,
    user_id: &Id,
    character_id: &Id,
) -> Result<Option<Human>, GetHumanByIdError> {
    let user_id_str = user_id.as_string_ref();
    let character_id_str = character_id.as_string_ref();

    let record = sqlx::query_as!(
        SqlHuman,
        "
            SELECT
                Characters.Id as Id,
                Characters.Name as Name,
                Characters.Nickname as Nickname,
                Characters.Race as Race
            FROM
                Characters
                    JOIN Humans ON Characters.Id = Humans.Id
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
        .map(|record| record.try_into_human(state))
        .transpose()
        .inspect_err(|e| log::error!("{}, error = {e}", log_location!()))?)
}

impl SqlHuman {
    pub fn try_into_human(self, _state: &State) -> Result<Human, CannotConvertSqlHumanToHuman> {
        const EXPECTED_CHARACTER_RACE: CharacterRace = CharacterRace::Human;

        let character_race = self.Race.parse()?;

        match character_race {
            EXPECTED_CHARACTER_RACE => Ok(Human {
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
    pub enum CannotConvertSqlHumanToHuman {
        #[error("CannotConvertSqlHumanToHuman: '{0}'")]
        SqlxError(
            #[from]
            #[source]
            SqlxError,
        ),

        #[error("CannotConvertSqlHumanToHuman: '{0}'")]
        InvalidCharacterRace(
            #[from]
            #[source]
            InvalidCharacterRace,
        ),

        #[error("CannotConvertSqlHumanToHuman: '{0}'")]
        CharacterRaceMismatchError(
            #[from]
            #[source]
            CharacterRaceMismatchError,
        ),
    }

    #[derive(Clone, Debug, thiserror::Error)]
    pub enum GetHumanByIdError {
        #[error("GetHumanByIdError: '{0}'")]
        SqlxError(
            #[from]
            #[source]
            SqlxError,
        ),

        #[error("GetHumanByIdError: '{0}'")]
        CannotConvertSqlHumanToHuman(
            #[from]
            #[source]
            CannotConvertSqlHumanToHuman,
        ),
    }
}
