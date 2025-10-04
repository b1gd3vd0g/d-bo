//! This module contains models that are stored as fields of greater documents within the database.
//! These can either be enum values **or** they can be entire structs.

use serde::{Deserialize, Serialize};

// ///////////////// //
// Player Sub-Models //
// ///////////////// //

/// The player's preferred gender. In the case of Spanish-speaking non-binary players, they can also
/// have a pronoun field of type Gender, which may not agree with their specified Gender. This is
/// important, as the "-e" endings for non-binary people is not universally accepted or recognized.
#[derive(Clone, Deserialize, Serialize)]
pub enum Gender {
    /// Player identifies as male (default to masculine pronouns).
    #[serde(rename = "m")]
    Male,
    /// Player identifies as female (default to feminine pronouns).
    #[serde(rename = "f")]
    Female,
    /// Player identifies as a non-binary gender (default to neutral pronouns for English-speaking
    /// players; for Spanish speaking players of this identity, we ask what pronouns they would
    /// prefer).
    #[serde(rename = "nb")]
    Other,
}

/// The player's preferred language for UX, email correspondence, etc.
#[derive(Clone, Deserialize, Serialize)]
pub enum LanguagePreference {
    /// American English
    #[serde(rename = "en")]
    English,
    /// Latin American Spanish
    #[serde(rename = "es")]
    Spanish,
}

/// Keeps track of a player's gameplay statistics.
#[derive(Clone, Deserialize, Serialize)]
pub struct PlayerStats {
    /// The number of games won. This includes standard wins, shared wins via the **last chance**
    /// house rule, and wins by default (when all other players drop out).
    wins: u64,
    /// The number of games finished, but lost.
    losses: u64,
    /// The number of games from which a player has been forfeit, either by choosing to drop out, or
    /// by taking too long to play their turn.
    dropouts: u64,
}

impl PlayerStats {
    /// The default PlayerStats for a new Player, for which all fields are initialized to 0.
    pub fn default() -> Self {
        Self {
            wins: 0,
            losses: 0,
            dropouts: 0,
        }
    }
}
