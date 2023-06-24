use std::io;

use crate::MicroDB;

/// A path which escapes its inner path
#[derive(Clone)]
pub struct Escape<P: Path>(pub P);

impl<T: Path> Path for Escape<T> {
    fn to_db_path(self) -> String {
        self.0.to_db_path().replace('\\', "\\b").replace('/', "\\s")
    }
}

/// A path which unescapes its inner path
#[derive(Clone)]
pub struct Unescape<P: Path>(pub P);

impl<T: Path> Path for Unescape<T> {
    fn to_db_path(self) -> String {
        self.0.to_db_path().replace("\\s", "/").replace("\\b", "\\")
    }
}

/// Anything that can be used as a path in a MicroDB
pub trait Path: Sized + Clone {
    /// Turns the path into a string for storage in the [`crate::FAlloc`]
    fn to_db_path(self) -> String;
    /// Returns a sub-path of this path. Separator is added automatically, but escaping is NOT done.
    fn sub_path<P: Path>(&self, other: P) -> String {
        self.clone().to_db_path() + "/" + &other.to_db_path()
    }
}

/// An object which the DB can represent in bytes
pub trait RawObj: Sized {
    /// Turns the object into a byte-array for storage.
    fn to_db(self) -> Vec<u8>;
    /// Turns a byte-array back into this object. None means failure to decode.
    fn from_db(x: Vec<u8>) -> Option<Self>;
}

/// A composite object, made of other ComObjects and RawObjects.
pub trait ComObj: Sized {
    /// Turns the object into multiple DB-objects, which can be ComObjects or RawObjects.
    /// To write, use the set_raw and set_com methods on the db. Don't forget to always use
    /// [`Path::sub_path`].
    fn to_db<P: Path>(self, path: P, db: &MicroDB) -> Result<(), io::Error>;
    /// Removes the object and its sub-objects from the DB. Don't forget to always use
    /// [`Path::sub_path`].
    fn remove<P: Path>(path: P, db: &MicroDB) -> Result<(), io::Error>;
    /// Turns the DB object back into the original object. None means failure to decode.
    /// Don't forget to always use [`Path::sub_path`].
    fn from_db<P: Path>(path: P, db: &MicroDB) -> Result<Option<Self>, io::Error>;
    /// Returns list of paths in the object. If not implemented manually, uses catch-all
    /// function [`MicroDB::get_paths`]. MUST NOT return indirect sub-paths (sub-paths of
    /// sub-paths)
    fn paths<P: Path>(path: P, db: &MicroDB) -> Result<Vec<String>, io::Error> {
        db.get_paths(Some(path))
    }
}

/// When implemented, automatically implements ComObj for the RawObj.
/// Use with `com_obj!($t);`.
pub trait AutoComObj: RawObj {}

impl<T> ComObj for T
where
    T: AutoComObj,
{
    fn to_db<P: Path>(self, path: P, db: &MicroDB) -> Result<(), io::Error> {
        db.set_raw(path, self)
    }

    fn remove<P: Path>(path: P, db: &MicroDB) -> Result<(), io::Error> {
        db.remove_raw(path)
    }

    fn from_db<P: Path>(path: P, db: &MicroDB) -> Result<Option<Self>, io::Error> {
        db.get_raw(path)
    }

    fn paths<P: Path>(_path: P, _db: &MicroDB) -> Result<Vec<String>, io::Error> {
        Ok(Vec::new())
    }
}

/// Automatically implements ComObj for a RawObj
#[macro_export]
macro_rules! com_obj {
    {$t:ty} => {
        impl $crate::data::traits::AutoComObj for $t {}
    };
}

impl Path for &str {
    fn to_db_path(self) -> String {
        self.to_owned()
    }
}

impl Path for String {
    fn to_db_path(self) -> String {
        self
    }
}
