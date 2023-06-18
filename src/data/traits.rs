use std::io;

use crate::MicroDB;

pub trait Path: Sized + Clone {
    fn to_db_path(self) -> String;
    fn sub_path<P: Path>(&self, other: P) -> String {
        self.clone().to_db_path().to_owned() + &other.to_db_path()
    }
}

/// An object which the DB can represent in bytes
pub trait RawObj: Sized {
    fn to_db(self) -> Vec<u8>;
    fn from_db(x: Vec<u8>) -> Option<Self>;
}

/// A composite object, made of other ComObjects and RawObjects.
pub trait ComObj: Sized {
    fn to_db<P: Path>(self, path: P, db: &MicroDB) -> Result<(), io::Error>;
    fn remove<P: Path>(path: P, db: &MicroDB) -> Result<(), io::Error>;
    fn from_db<P: Path>(path: P, db: &MicroDB) -> Result<Option<Self>, io::Error>;
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
}

/// Automatically implements ComObj for a RawObj
#[macro_export]
macro_rules! com_obj {
    {$t:ty} => {
        impl AutoComObj for $t {}
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
