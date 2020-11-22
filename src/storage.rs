// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! Contains the main algorithm for the Paxos consensus protocol.

use serde::Serialize;
use tracing::error;

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
