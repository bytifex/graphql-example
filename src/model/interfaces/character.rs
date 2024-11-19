#![allow(clippy::enum_variant_names)]
#![allow(clippy::duplicated_attributes)]

use async_graphql::Interface;

use crate::model::{
    enums::character_race::CharacterRace,
    scalars::id::Id,
    types::{android::Android, cyborg::Cyborg, human::Human},
};

use super::augmented_character::AugmentedCharacter;

#[derive(Interface)]
#[graphql(
    field(name = "id", ty = "&Id", desc = "Id of the character"),
    field(
        name = "name",
        ty = "&Option<String>",
        desc = "Full name of the character",
        deprecation = "This field will be removed",
    ),
    field(name = "nickname", ty = "&String", desc = "Nickname of the character"),
    field(name = "race", ty = "CharacterRace", desc = "Race of the character")
)]
pub enum Character {
    // derived interfaces
    AugmentedCharacter(AugmentedCharacter),

    // types
    Android(Android),
    Cyborg(Cyborg),
    Human(Human),
}
