use async_graphql::{Description, Object};

use crate::{
    model::{interfaces::character::Character, scalars::id::Id},
    sql_queries::sql_character::{
        error::{GetCharacterByIdError, GetCharactersError},
        get_character_by_id, get_characters,
    },
    state::State,
};

/// User of the application
#[derive(Description)]
pub struct User {
    pub state: State,

    pub id: Id,
    pub nickname: String,
    pub email_address: Option<String>,
}

#[Object(use_type_description)]
impl User {
    /// Nick name of the user
    #[graphql(deprecation = "Use `nickName` instead")]
    pub async fn display_name(&self) -> &String {
        &self.nickname
    }

    /// Nick name of the user
    pub async fn nick_name(&self) -> &String {
        &self.nickname
    }

    /// Email address of the user
    pub async fn email_address(&self) -> &Option<String> {
        &self.email_address
    }

    /// Id of the user
    pub async fn id(&self) -> &Id {
        &self.id
    }

    /// Character of the user with the given id
    pub async fn character_by_id(
        &self,
        id: Id,
    ) -> Result<Option<Character>, GetCharacterByIdError> {
        get_character_by_id(&self.state, &self.id, &id).await
    }

    /// Characters belonging to the user
    pub async fn characters(&self) -> Result<Vec<Character>, GetCharactersError> {
        get_characters(&self.state, &self.id).await
    }
}
