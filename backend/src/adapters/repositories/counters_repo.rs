//! This module provides unique functionality over the counter repository. The only current function
//! for a Counter is to increment it by 1.

// NOTE: Future enhancements to the application may include functionalities to fetch one or more
// counters from the database, or to decrement a counter. These functions are not currently needed for
// the app, so are not yet included.

use mongodb::{bson::doc, options::ReturnDocument};

use crate::{
    adapters::repositories::{Repository, counter_id::CounterId},
    errors::DBoResult,
    models::{Counter, Identifiable},
};

impl Repository<Counter> {
    /// Increment a Counter by 1. If the counter is not found, it will initialize the counter
    /// to 1.
    ///
    /// ### Returns
    /// The new `count` of the Counter
    ///
    /// ### Errors
    /// `AdapterError` if the query fails
    pub async fn increment_counter(&self, id: CounterId) -> DBoResult<u64> {
        Ok(self
            .collection
            .find_one_and_update(
                doc! { Counter::id_field(): &id.to_string() },
                doc! { "$inc": { "count": 1 } },
            )
            .upsert(true)
            .return_document(ReturnDocument::After)
            .await?
            .unwrap()
            .count())
    }
}
