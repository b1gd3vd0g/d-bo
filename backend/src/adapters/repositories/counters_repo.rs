//! This module provides unique functionality over the counter repository

use mongodb::bson::doc;

use crate::{
    adapters::repositories::Repository,
    errors::DBoResult,
    models::{Counter, Identifiable},
};

impl Repository<Counter> {
    //! Ping the database. This is run on startup, and is used to test that the database connection
    //! can be made. The counter is therefore a representation of how many times the app has started
    //! up and connected successfully to the database.
    pub async fn ping(&self) -> DBoResult<u64> {
        Ok(self
            .collection
            .find_one_and_update(
                doc! { Counter::id_field(): "pings" },
                doc! { "$inc": { "counter": 1 } },
            )
            .upsert(true)
            .await?
            .unwrap()
            .counter())
    }
}
