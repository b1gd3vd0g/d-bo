//! This module is home to the **repository layer** of the application. The repository layer handles
//! direct interaction with the database.
//!
//! A single repository handles a single database model, and all queries related to it. The
//! **service layer** is responsible for orchestration between multiple repositories.
//!
//! Within the base module, some blanket implementations are handled for all repositories, as well
//! as inserts for repositories of **unconstrained** models, which are not constrained by uniqueness
//! indices (except for their id fields).

#[doc(hidden)]
mod confirmation_token_repo;
pub mod counter_id;
#[doc(hidden)]
mod counters_repo;
#[doc(hidden)]
mod player_repo;
#[doc(hidden)]
mod refresh_token_repo;
#[doc(hidden)]
mod undo_token_repo;

use mongodb::{Collection, bson::doc};

use crate::{
    adapters::mongo::database,
    errors::DBoResult,
    models::{Collectible, ConfirmationToken, Counter, Model, Player, RefreshToken, UndoToken},
};

/// An interface over a database collection which handles all database interactions related to a
/// specific Model.
#[derive(Clone)]
pub struct Repository<T: Model + Send + Sync> {
    /// The MongoDB collection that this repository will handle.
    collection: Collection<T>,
}

impl<T: Model + Send + Sync> Repository<T> {
    /// Create a new Repository
    ///
    /// ### Arguments
    /// - `collection`: The MongoDB collection that this Repository will handle.
    pub fn new(collection: Collection<T>) -> Self {
        Self {
            collection: collection,
        }
    }

    /// Find a document within the repository, referencing it by its unique identifier.
    ///
    /// ### Returns
    /// - `Some(doc)` if the document exists
    /// - `None` if the document does not exist
    ///
    /// ### Errors
    /// - `AdapterError` if the query fails
    pub async fn find_by_id(&self, id: &str) -> DBoResult<Option<T>> {
        Ok(self.collection.find_one(doc! { T::id_field(): id }).await?)
    }

    /// Delete a document within the repository, referencing it by its unique identifier.
    ///
    /// ### Returns
    /// - `Some(doc)` - the deleted document
    /// - `None` - if the document could not be found
    ///
    /// ### Errors
    /// - `AdapterError` if the query fails
    pub async fn delete(&self, id: &str) -> DBoResult<Option<T>> {
        Ok(self
            .collection
            .find_one_and_delete(doc! { T::id_field(): id })
            .await?)
    }
}

/// A struct containing all of the repositories needed by the application.
#[derive(Clone)]
pub struct Repositories {
    /// The repository handling email confirmation tokens.
    confirmation_tokens: Repository<ConfirmationToken>,
    /// The repository handling counters.
    counters: Repository<Counter>,
    /// The repository handling player accounts.
    players: Repository<Player>,
    /// The repository handling player refresh tokens.
    refresh_tokens: Repository<RefreshToken>,
    /// The repository handling player undo tokens.
    undo_tokens: Repository<UndoToken>,
}

impl Repositories {
    /// Create a new Repositories struct, with a guaranteed connection to the database.
    ///
    /// ### Panics
    /// If the database connection could not be established.
    pub async fn new() -> Self {
        let db = database().await;
        Self {
            confirmation_tokens: Repository::<ConfirmationToken>::new(
                db.collection(ConfirmationToken::collection_name()),
            ),
            counters: Repository::<Counter>::new(db.collection(&Counter::collection_name())),
            players: Repository::<Player>::new(db.collection(&Player::collection_name())),
            refresh_tokens: Repository::<RefreshToken>::new(
                db.collection(RefreshToken::collection_name()),
            ),
            undo_tokens: Repository::<UndoToken>::new(
                db.collection(RefreshToken::collection_name()),
            ),
        }
    }

    /// Return the confirmation tokens repository.
    pub fn confirmation_tokens(&self) -> &Repository<ConfirmationToken> {
        &self.confirmation_tokens
    }

    /// Return the counters repository.
    pub fn counters(&self) -> &Repository<Counter> {
        &self.counters
    }

    /// Return the players repository.
    pub fn players(&self) -> &Repository<Player> {
        &self.players
    }

    pub fn refresh_tokens(&self) -> &Repository<RefreshToken> {
        &self.refresh_tokens
    }

    pub fn undo_tokens(&self) -> &Repository<UndoToken> {
        &self.undo_tokens
    }
}
