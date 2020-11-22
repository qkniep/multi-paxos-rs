// Copyright (C) 2020 Quentin M. Kniep <hello@quentinkniep.com>
// Distributed under terms of the MIT license.

//! Defines ways of persisting data to disk and retrieving it back.
//! PaxosReplica uses this module's methods to keep its persistent state.

use serde::{de::DeserializeOwned, Serialize};
use tracing::error;

/// Serializes the `value` into a file called `filename`.
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

/// Deserializes the previously stored value from the file called `filename`.
pub fn load_from_disk_file<T: DeserializeOwned>(filename: &str) -> Result<T, ()> {
    let storage = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(filename)
        .map_err(|e| {
            error!("Failed to create or open file: {:?}", e);
        })?;
    bincode::deserialize_from(storage).map_err(|e| {
        error!("Failed to serialize state: {:?}", e);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_load() {
        static FILENAME: &str = "store_and_load.XlWG2sQCFyXNjIyq.bin";
        store_in_disk_file(FILENAME, &999).unwrap();
        let num: i32 = load_from_disk_file(FILENAME).unwrap();
        assert_eq!(num, 999);
        std::fs::remove_file(FILENAME).unwrap();
    }

    #[test]
    fn store_and_load_vec() {
        static FILENAME: &str = "store_and_load_vec.TWvJtuzqqbwOVGu5.bin";
        let squares = vec![0, 1, 4, 9, 16, 25, 36, 49, 64, 81];
        store_in_disk_file(FILENAME, &squares).unwrap();
        let squares_loaded: Vec<i32> = load_from_disk_file(FILENAME).unwrap();
        println!("{:?}", squares_loaded);
        assert_eq!(squares_loaded, squares);
        std::fs::remove_file(FILENAME).unwrap();
    }
}
