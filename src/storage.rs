// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! Defines ways of persisting data to disk and retrieving it back.
//! PaxosReplica uses this module's methods to keep its persistent state.

use serde::Serialize;
use tracing::error;

/// Serializes the `vlaue` into a file called `filename`.
pub fn store_in_disk_file<T: ?Sized + Serialize>(filename: &str, value: &T) -> Result<(), ()> {
    let storage = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(filename)
        .map_err(|e| {
            error!("Failed to create or open file: {:?}", e);
        })?;
    bincode::serialize_into(storage, value).map_err(|e| {
        error!("Failed to serialize state: {:?}", e);
    })
}
