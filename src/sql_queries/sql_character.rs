use error::{GetCharacterByIdError, GetCharactersError};

use crate::{
    error::SqlxErrorConverter,
    log_location,
    model::{
        enums::character_race::CharacterRace, interfaces::character::Character, scalars::id::Id,
    },
    state::State,
};

use super::{
    sql_android::get_android_by_id, sql_cyborg::get_cyborg_by_id, sql_human::get_human_by_id,
};

pub async fn get_character_by_id(
    state: &State,
    user_id: &Id,
    character_id: &Id,
) -> Result<Option<Character>, GetCharacterByIdError> {
    let user_id_str = user_id.as_string_ref();
    let character_id_str = character_id.as_string_ref();

    let record = sqlx::query!(
        "
            SELECT
                Race
            FROM
                Characters
                    JOIN Users ON Characters.UserId = Users.Id
            WHERE
                Users.Id = ? AND Characters.Id = ?
        ",
        user_id_str,
        character_id_str,
    )
    .fetch_optional(state.database.connection_pool_ref())
    .await
    .to_sqlx_error_result()
    .inspect_err(|e| log::error!("{}, error = {e}", log_location!()))?;

    Ok(match record {
        Some(record) => {
            let race = record.Race.parse()?;

            match race {
                CharacterRace::Android => get_android_by_id(state, user_id, character_id)
                    .await?
                    .map(Character::Android),
                CharacterRace::Cyborg => get_cyborg_by_id(state, user_id, character_id)
                    .await?
                    .map(Character::Cyborg),
                CharacterRace::Human => get_human_by_id(state, user_id, character_id)
                    .await?
                    .map(Character::Human),
            }
        }
        None => None,
    })
}

pub async fn get_characters(
    state: &State,
    user_id: &Id,
) -> Result<Vec<Character>, GetCharactersError> {
    let user_id_str = user_id.as_string_ref();

    let records = sqlx::query!(
        "
            SELECT
                Characters.Id, Race
            FROM
                Characters
                    JOIN Users ON Characters.UserId = Users.Id
            WHERE
                Users.Id = ?
            ORDER BY
                Characters.Id
        ",
        user_id_str,
    )
    .fetch_all(state.database.connection_pool_ref())
    .await
    .to_sqlx_error_result()
    .inspect_err(|e| log::error!("{}, error = {e}", log_location!()))?;

    let mut ret = Vec::new();

    for record in records {
        let character_race = record.Race.parse()?;

        let character = match character_race {
            CharacterRace::Android => get_android_by_id(state, user_id, &record.Id.into())
                .await?
                .map(Character::Android),
            CharacterRace::Cyborg => get_cyborg_by_id(state, user_id, &record.Id.into())
                .await?
                .map(Character::Cyborg),
            CharacterRace::Human => get_human_by_id(state, user_id, &record.Id.into())
                .await?
                .map(Character::Human),
        };

        if let Some(character) = character {
            ret.push(character);
        }
    }

    Ok(ret)
}

pub mod error {
    use crate::{
        error::SqlxError,
        model::enums::character_race::{error::InvalidCharacterRace, CharacterRace},
        sql_queries::{
            sql_android::error::GetAndroidByIdError, sql_cyborg::error::GetCyborgByIdError,
            sql_human::error::GetHumanByIdError,
        },
    };

    #[derive(Clone, Debug, thiserror::Error)]
    #[error("CharacterRaceMismatchError: stored = '{stored}', expected = '{expected}'")]
    pub struct CharacterRaceMismatchError {
        pub stored: CharacterRace,
        pub expected: CharacterRace,
    }

    #[derive(Clone, Debug, thiserror::Error)]
    pub enum GetCharacterByIdError {
        #[error("GetCharacterByIdError: '{0}'")]
        SqlxError(
            #[from]
            #[source]
            SqlxError,
        ),

        #[error("GetCharacterByIdError: '{0}'")]
        InvalidCharacterRace(
            #[from]
            #[source]
            InvalidCharacterRace,
        ),

        #[error("GetCharacterByIdError: '{0}'")]
        GetAndroidByIdError(
            #[from]
            #[source]
            GetAndroidByIdError,
        ),

        #[error("GetCharacterByIdError: '{0}'")]
        GetCyborgByIdError(
            #[from]
            #[source]
            GetCyborgByIdError,
        ),

        #[error("GetCharacterByIdError: '{0}'")]
        GetHumanByIdError(
            #[from]
            #[source]
            GetHumanByIdError,
        ),
    }

    #[derive(Clone, Debug, thiserror::Error)]
    pub enum GetCharactersError {
        #[error("GetCharactersError: '{0}'")]
        SqlxError(
            #[from]
            #[source]
            SqlxError,
        ),

        #[error("GetCharactersError: '{0}'")]
        InvalidCharacterRace(
            #[from]
            #[source]
            InvalidCharacterRace,
        ),

        #[error("GetCharactersError: '{0}'")]
        GetAndroidByIdError(
            #[from]
            #[source]
            GetAndroidByIdError,
        ),

        #[error("GetCharactersError: '{0}'")]
        GetCyborgByIdError(
            #[from]
            #[source]
            GetCyborgByIdError,
        ),

        #[error("GetCharactersError: '{0}'")]
        GetHumanByIdError(
            #[from]
            #[source]
            GetHumanByIdError,
        ),
    }
}
