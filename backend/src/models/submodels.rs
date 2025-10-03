//! This module contains models that are stored as fields of greater documents within the database.
//! These can either be enum values **or** they can be entire structs.

use serde::{Deserialize, Serialize};

// ///////////////// //
// Player Sub-Models //
// ///////////////// //

/// The player's preferred gender.
#[derive(Clone, Deserialize, Serialize)]
pub enum Gender {
    /// Player identifies as male (default to masculine pronouns).
    #[serde(rename = "male")]
    Male,
    /// Player identifies as female (default to feminine pronouns).
    #[serde(rename = "female")]
    Female,
    /// Player identifies as a non-binary gender (default to neutral pronouns for English-speaking
    /// players; for Spanish speaking players of this identity, we ask what pronouns they would
    /// prefer).
    #[serde(rename = "other")]
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

/// How we should refer to this player in language; especially important for gendered language
/// in Spanish translation.
#[derive(Clone, Deserialize, Serialize)]
pub enum Pronoun {
    /// "¡Bienvenido, estimado jugador!"
    #[serde(rename = "masculine")]
    Masculine,
    /// "¡Bienvenida, estimada jugadora!"
    #[serde(rename = "feminine")]
    Feminine,
    /// "¡Bienvenide, estimade jugador!"
    #[serde(rename = "neutral")]
    Neutral,
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
    pub fn default() -> Self {
        Self {
            wins: 0,
            losses: 0,
            dropouts: 0,
        }
    }
}
