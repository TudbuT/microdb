use std::io;

use crate::FAlloc;

use crate::data::*;

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
            .max(10.0)
            .min(10000.0) as usize
    }

    /// Expires the cache and flushes it.
    pub fn sync(&self) -> Result<(), io::Error> {
        self.storage.sync()
    }

    /// Syncs, then saves metadata (allocations).
    pub fn save(&self) -> Result<(), io::Error> {
        self.storage.save()
    }

    /// Gracefully shuts down the DB, saving in the process.
    /// Please use [`shutdown`] instead if possible. This variant
    /// will force a shutdown across all threads without the guarantee that
    /// this is the only thread with access to it.
    pub fn shutdown_here(&self) -> Result<(), io::Error> {
        self.storage.shutdown_here()
    }

    /// Gracefully shuts down the DB, saving in the process.
    pub fn shutdown(self) -> Result<(), io::Error> {
        self.storage.shutdown()
    }

    /// Sets an item in the database at the path.
    /// Here, the item is saved in a single blob at the path.
    pub fn set_raw<T: RawObj, P: Path>(&self, path: P, object: T) -> Result<(), io::Error> {
        let path = path.to_db_path();
        self.storage.delete_substructure(&path)?; // raw objects mustn't have substructure
        self.storage.set(&path, object.to_db())
    }

    /// Sets an item in the database at the path.
    /// Here, the item is a composite item, so multiple blobs on sub-paths
    /// may be created.
    pub fn set_com<T: ComObj, P: Path>(&self, path: P, object: T) -> Result<(), io::Error> {
        self.storage
            .delete_substructure(&path.clone().to_db_path())?; // clean substructure
        T::to_db(object, path, self)
    }

    /// Sets an item in the database at the path.
    /// Here, the item is a composite item, so multiple blobs on sub-paths
    /// may be created.
    ///
    /// # Safety
    ///
    /// This function will not clean up the old substructure. It may create database junk until
    /// the next time that substructure is cleaned by some other function. Use this only if you
    /// know that the types of the previous inhabitant and the new one are the same and that the
    /// types aren't dynamic (like Vec<T> is).
    pub fn set_com_hard<T: ComObj, P: Path>(&self, path: P, object: T) -> Result<(), io::Error> {
        T::to_db(object, path, self)
    }

    /// Gets an item from the database.
    pub fn get_raw<T: RawObj, P: Path>(&self, path: P) -> Result<Option<T>, io::Error> {
        Ok(self.storage.get(&path.to_db_path())?.and_then(T::from_db))
    }

    /// Gets a composite item from the database.
    pub fn get_com<T: ComObj, P: Path>(&self, path: P) -> Result<Option<T>, io::Error> {
        T::from_db(path, self)
    }

    /// Removes any item from the database.
    pub fn remove<P: Path>(&self, path: P) -> Result<(), io::Error> {
        let path = path.to_db_path();
        self.storage.delete_substructure(&path)?;
        self.storage.set(&path, Vec::new())
    }

    /// Removes a single-blob item from the database gracefully.
    pub fn remove_raw<P: Path>(&self, path: P) -> Result<(), io::Error> {
        self.storage.set(&path.to_db_path(), Vec::new())
    }

    /// Removes a composite item from the database gracefully.
    pub fn remove_com<T: ComObj, P: Path>(&self, path: P) -> Result<(), io::Error> {
        T::remove(path, self)
    }
}

#[macro_export]
macro_rules! extract {
    ($val:expr) => {
        if let Some(x) = $val? {
            x
        } else {
            return Ok(None);
        };
    };
}
