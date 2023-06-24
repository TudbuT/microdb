use std::io;

use crate::FAlloc;

use crate::data::*;

pub struct MicroDB {
    storage: FAlloc,
}

impl MicroDB {
    /// Loads a database. Can NOT be used to create one.
    pub fn new<S: ToString>(data: S, alloc: S, cache_period: u128) -> Result<Self, io::Error> {
        Ok(Self {
            storage: FAlloc::new(data, alloc, cache_period)?,
        })
    }

    /// Creates a database. Can NOT be used to load one.
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

    /// Expires the cache and flushes it.
    pub fn sync(&self) -> Result<(), io::Error> {
        self.storage.sync()
    }

    /// Syncs, then saves metadata (allocations).
    pub fn save(&self) -> Result<(), io::Error> {
        self.storage.save()
    }

    /// Gracefully shuts down the DB, saving in the process.
    /// Please use [`Self::shutdown`] instead if possible. This variant
    /// will force a shutdown across all threads without the guarantee that
    /// this is the only thread with access to it.
    pub fn shutdown_here(&self) -> Result<(), io::Error> {
        self.storage.shutdown_here()
    }

    /// Gracefully shuts down the DB, saving in the process.
    pub fn shutdown(self) -> Result<(), io::Error> {
        self.storage.shutdown()
    }

    /// Returns the direct sub-paths of a path, or the direct root paths.
    /// Does NOT return sub-paths of sub-paths.
    pub fn get_paths<P: Path>(&self, path: Option<P>) -> Result<Vec<String>, io::Error> {
        if let Some(path) = path {
            self.storage.paths(Some(&path.to_db_path()))
        } else {
            self.storage.paths(None)
        }
    }

    /// Returns all sub-paths of a path, including indirect ones.
    pub fn get_all_paths<P: Path>(&self, path: Option<P>) -> Result<Vec<String>, io::Error> {
        if let Some(path) = path {
            self.storage.all_paths(Some(&path.to_db_path()))
        } else {
            self.storage.all_paths(None)
        }
    }

    /// Primitively parses the object just enough to know the paths it occupies directly.
    /// Does NOT return sub-paths of sub-paths.
    pub fn get_paths_of<T: ComObj, P: Path>(&self, path: P) -> Result<Vec<String>, io::Error> {
        T::paths(path, self)
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
    /// types aren't dynamic (like [`Vec<T>`] is).
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

/// Convenience macro to extract a value from the database and return Ok(None) if not found.
///
/// Example usage:
/// ```ignore
/// fn from_db<P: Path>(path: P, db: &MicroDB) -> Result<Option<Self>, std::io::Error> {
///     Ok(Some(Self {
///         username: extract!(db.get_raw(path.sub_path("username"))),
///         email_address: extract!(db.get_raw(path.sub_path("email"))),
///         password_hash: extract!(db.get_raw(path.sub_path("pass"))),
///     }))
/// }
/// ```
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
