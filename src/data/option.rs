use crate::MicroDB;

use super::{ComObj, Path, RawObj};

impl<T> RawObj for Option<T>
where
    T: RawObj,
{
    fn to_db(self) -> Vec<u8> {
        let mut v = Vec::new();
        match self {
            Some(x) => {
                v.push(1);
                v.append(&mut x.to_db());
            }
            None => {
                v.push(0);
            }
        }
        v
    }

    fn from_db(mut x: Vec<u8>) -> Option<Self> {
        if x.is_empty() {
            return None;
        }
        if x[0] == 0 && x.len() == 1 {
            return Some(None);
        }
        if x[0] == 1 {
            x.remove(0);
            return T::from_db(x).map(|x| Some(x));
        }
        None
    }
}

impl<T> ComObj for Option<T>
where
    T: ComObj,
{
    fn to_db<P: Path>(self, path: P, db: &MicroDB) -> Result<(), std::io::Error> {
        db.set_raw(path.sub_path("/type"), self.is_some())?;
        if let Some(x) = self {
            db.set_com(path.sub_path("/data"), x)
        } else {
            db.remove_raw(path.sub_path("/data"))
        }
    }

    fn remove<P: Path>(path: P, db: &MicroDB) -> Result<(), std::io::Error> {
        db.remove_raw(path.sub_path("/type"))?;
        db.remove_raw(path.sub_path("/data"))?;
        Ok(())
    }

    fn from_db<P: Path>(path: P, db: &MicroDB) -> Result<Option<Self>, std::io::Error> {
        if let Some(x) = db.get_raw(path.sub_path("/type"))? {
            if x {
                if let Some(data) = db.get_com(path.sub_path("/data"))? {
                    // found
                    Ok(Some(Some(data)))
                } else {
                    // broken
                    Ok(None)
                }
            } else {
                // found but empty
                Ok(Some(None))
            }
        } else {
            // broken
            Ok(None)
        }
    }
}
