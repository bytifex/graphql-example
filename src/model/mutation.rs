use async_graphql::Object;

use crate::{error::UnimplementedError, state::State};

use super::{
    inputs::character_creation_input::CharacterCreationInput, interfaces::character::Character,
    scalars::id::Id, types::user::User,
};

pub struct Mutation {
    pub _state: State,
}

#[Object]
impl Mutation {
    pub async fn set_display_name(
        &self,
        _display_name: String,
    ) -> Result<User, UnimplementedError> {
        Err(UnimplementedError("Mutation::set_display_name".into()))
    }

    pub async fn create_character(
        &self,
        _user_id: Id,
        _character_definition: CharacterCreationInput,
    ) -> Result<Character, UnimplementedError> {
        Err(UnimplementedError("Mutation::create_character".into()))
    }
}
