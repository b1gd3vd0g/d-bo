//! This module provides unique functionality over the counter repository. There are only two
//! functionalities associated with Counters - incrementing them when certain actions are
//! performed, and checking their stored value.

// NOTE: it is possible that in the future I will add a function `decrement_counter()`, but it is
// not currently required for anything I plan to do.

use mongodb::bson::doc;

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
                doc! { "$inc": { "counter": 1 } },
            )
            .upsert(true)
            .await?
            .unwrap()
            .count())
    }

    /// Check the `count` of a Counter.
    ///
    /// ### Returns
    /// The `count` of the Counter, or `0` if it cannot be found
    ///
    /// ### Errors
    /// `AdapterError` if the query fails
    pub async fn check_counter(&self, id: CounterId) -> DBoResult<u64> {
        let counter = self
            .collection
            .find_one(doc! { Counter::id_field(): &id.to_string() })
            .await?;

        Ok(match counter {
            Some(c) => c.count(),
            None => 0,
        })
    }
}
