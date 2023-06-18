use crate::MicroDB;

use super::{ComObj, Path, RawObj};

impl<T> RawObj for Vec<T>
where
    T: RawObj,
{
    fn to_db(self) -> Vec<u8> {
        let mut v = Vec::new();
        for item in self {
            let mut item = item.to_db();
            v.append(&mut RawObj::to_db(item.len() as u64));
            v.append(&mut item);
        }
        v
    }

    fn from_db(x: Vec<u8>) -> Option<Self> {
        let mut v = Vec::new();
        let mut x = &x[..];
        loop {
            if x.is_empty() {
                break;
            }
            if x.len() < 8 {
                return None;
            }
            let bytes = <u64 as RawObj>::from_db(Vec::from_iter(x[0..8].iter().copied()))? as usize;
            x = &x[8..];
            if x.len() < bytes {
                return None;
            }
            v.push(T::from_db(Vec::from_iter(x[..bytes].iter().copied()))?);
            x = &x[bytes..];
        }
        Some(v)
    }
}

impl<T> ComObj for Vec<T>
where
    T: ComObj,
{
    fn to_db<P: Path>(self, path: P, db: &MicroDB) -> Result<(), std::io::Error> {
        db.set_raw(path.clone(), self.len() as u64)?;
        for (i, item) in self.into_iter().enumerate() {
            db.set_com(path.sub_path(i as u64), item)?;
        }
        Ok(())
    }

    fn remove<P: Path>(path: P, db: &MicroDB) -> Result<(), std::io::Error> {
        let Some(len): Option<u64> = db.get_raw(path.clone())? else { return Ok(()) };
        for i in 0..len {
            db.remove_raw(path.sub_path(i as u64))?;
        }
        Ok(())
    }

    fn from_db<P: Path>(path: P, db: &MicroDB) -> Result<Option<Self>, std::io::Error> {
        let Some(len): Option<u64> = db.get_raw(path.clone())? else { return Ok(None) };
        let mut v = Vec::new();
        for i in 0..len {
            let Some(value) = db.get_com(path.sub_path(i as u64))? else { return Ok(None) };
            v.push(value);
        }
        Ok(Some(v))
    }
}
