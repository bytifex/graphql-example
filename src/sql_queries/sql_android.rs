use error::{CannotConvertSqlAndroidToAndroid, GetAndroidByIdError};

use crate::{
    error::SqlxErrorConverter,
    log_location,
    model::{enums::character_race::CharacterRace, scalars::id::Id, types::android::Android},
    state::State,
};

use super::sql_character::error::CharacterRaceMismatchError;

#[derive(sqlx::Type)]
#[allow(non_snake_case)]
pub(super) struct SqlAndroid {
    pub Id: Id,
    pub Name: Option<String>,
    pub Nickname: String,
    pub Race: String,
}

pub async fn get_android_by_id(
    state: &State,
    user_id: &Id,
    character_id: &Id,
) -> Result<Option<Android>, GetAndroidByIdError> {
    let user_id_str = user_id.as_string_ref();
    let character_id_str = character_id.as_string_ref();

    let record = sqlx::query_as!(
        SqlAndroid,
        "
            SELECT
                Characters.Id as Id,
                Characters.Name as Name,
                Characters.Nickname as Nickname,
                Characters.Race as Race
            FROM
                Characters
                    JOIN Androids ON Characters.Id = Androids.Id
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
        .map(|record| record.try_into_android(state))
        .transpose()
        .inspect_err(|e| log::error!("{}, error = {e}", log_location!()))?)
}

impl SqlAndroid {
    pub fn try_into_android(
        self,
        _state: &State,
    ) -> Result<Android, CannotConvertSqlAndroidToAndroid> {
        const EXPECTED_CHARACTER_RACE: CharacterRace = CharacterRace::Android;

        let character_race = self.Race.parse()?;

        match character_race {
            EXPECTED_CHARACTER_RACE => Ok(Android {
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
    pub enum CannotConvertSqlAndroidToAndroid {
        #[error("CannotConvertSqlAndroidToAndroid: '{0}'")]
        SqlxError(
            #[from]
            #[source]
            SqlxError,
        ),

        #[error("CannotConvertSqlAndroidToAndroid: '{0}'")]
        InvalidCharacterRace(
            #[from]
            #[source]
            InvalidCharacterRace,
        ),

        #[error("CannotConvertSqlAndroidToAndroid: '{0}'")]
        CharacterRaceMismatchError(
            #[from]
            #[source]
            CharacterRaceMismatchError,
        ),
    }

    #[derive(Clone, Debug, thiserror::Error)]
    pub enum GetAndroidByIdError {
        #[error("GetAndroidByIdError: '{0}'")]
        SqlxError(
            #[from]
            #[source]
            SqlxError,
        ),

        #[error("GetAndroidByIdError: '{0}'")]
        CannotConvertSqlAndroidToAndroid(
            #[from]
            #[source]
            CannotConvertSqlAndroidToAndroid,
        ),
    }
}
