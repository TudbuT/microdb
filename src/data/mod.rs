pub mod traits;
use std::io;

use crate::FAlloc;

use self::traits::*;

pub struct MicroDB {
    storage: FAlloc,
}

impl MicroDB {
    pub fn new<S: ToString>(data: S, alloc: S, cache_period: u128) -> Result<Self, io::Error> {
        Ok(Self {
            storage: FAlloc::new(data, alloc, cache_period)?,
        })
    }

    pub fn create<S: ToString>(
        data: S,
        alloc: S,
        cache_period: u128,
        block_size: usize,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            storage: FAlloc::create(data, alloc, cache_period, block_size)?,
        })
    }

    /// Gives a sensible cache period so your cache will usually be filled well but not too much.
    /// Keep in mind that spikes up and down will happen and reserve enough RAM for that.
    /// `safety` should be from 0 to 1, where 0 means spikes are no problem, and 1 means to be
    /// especially careful.
    pub fn sensible_cache_period(
        requests_per_second: f64,
        ram_gb: f64,
        average_object_size_mb: f64,
        safety: f64,
    ) -> u128 {
        (100_000.0 * (1.01 - safety) / 1.01 * ram_gb
            / (requests_per_second * average_object_size_mb)) as u128
    }

    /// Gives you a sensible block size for given data requirements.
    /// Storage tightness will drastically affect your result.
    pub fn sensible_block_size(
        object_amount: f64,
        average_object_size_bytes: f64,
        object_size_fluctuation_bytes: f64,
        storage_tightness: f64,
    ) -> usize {
        // more objects = storage has to be more compact (-> bigger blocks)
        let tightness_coefficient =
            ((object_amount / 10_000.0).min(1.2) * storage_tightness).max(0.1);
        ((average_object_size_bytes / 1.2 + object_size_fluctuation_bytes) * tightness_coefficient)
            .max(30.0)
            .min(10000.0) as usize
    }

    pub fn sync(&self) -> Result<(), io::Error> {
        self.storage.sync()
    }

    pub fn save(&self) -> Result<(), io::Error> {
        self.storage.save()
    }

    pub fn shutdown(self) -> Result<(), io::Error> {
        self.storage.shutdown()
    }

    pub fn set<P: Path<T>, T: Obj>(&self, path: P, object: T) {
        self.storage
            .set(&path.to_db_path(), object.to_db_object())
            .unwrap()
    }

    pub fn get<P: Path<T>, T: Obj>(&self, path: P) -> Option<T> {
        self.storage
            .get(&path.to_db_path())
            .unwrap()
            .map(T::map)
            .flatten()
    }
}
