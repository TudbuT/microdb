use std::io;

use crate::MicroDB;

use super::{ComObj, Path, RawObj};

impl<T, E> RawObj for Result<T, E>
where
    T: RawObj,
    E: RawObj,
{
    fn to_db(self) -> Vec<u8> {
        let mut v = Vec::new();
        match self {
            Ok(x) => {
                v.push(1);
                v.append(&mut x.to_db());
            }
            Err(x) => {
                v.push(0);
                v.append(&mut x.to_db());
            }
        }
        v
    }

    fn from_db(mut x: Vec<u8>) -> Option<Self> {
        if x.is_empty() {
            return None;
        }
        if x[0] == 0 && x.len() == 1 {
            x.remove(0);
            return E::from_db(x).map(|x| Err(x));
        }
        if x[0] == 1 {
            x.remove(0);
            return T::from_db(x).map(|x| Ok(x));
        }
        None
    }
}

impl<T, E> ComObj for Result<T, E>
where
    T: ComObj,
    E: ComObj,
{
    fn to_db<P: Path>(self, path: P, db: &MicroDB) -> Result<(), io::Error> {
        db.set_raw(path.sub_path("type"), self.is_ok())?;
        match self {
            Ok(x) => db.set_com(path.sub_path("data"), x),
            Err(x) => db.set_com(path.sub_path("data"), x),
        }
    }

    fn remove<P: Path>(path: P, db: &MicroDB) -> Result<(), io::Error> {
        db.remove_raw(path.sub_path("type"))?;
        db.remove(path.sub_path("data"))?;
        Ok(())
    }

    fn from_db<P: Path>(path: P, db: &MicroDB) -> Result<Option<Self>, io::Error> {
        if let Some(x) = db.get_raw(path.sub_path("type"))? {
            if x {
                if let Some(data) = db.get_com(path.sub_path("data"))? {
                    // found
                    Ok(Some(Ok(data)))
                } else {
                    // broken
                    Ok(None)
                }
            } else if let Some(data) = db.get_com(path.sub_path("data"))? {
                // found
                Ok(Some(Err(data)))
            } else {
                // broken
                Ok(None)
            }
        } else {
            // broken
            Ok(None)
        }
    }

    fn paths<P: Path>(path: P, db: &MicroDB) -> Result<Vec<String>, io::Error> {
        Ok(vec![path.sub_path("type"), path.sub_path("data")])
    }
}
