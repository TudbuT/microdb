use super::{ComObj, Path, RawObj};
use crate::{extract, MicroDB};
use ident_concat::ident;
use std::io;

// agony.
macro_rules! impl_tuple {
    ($($tvarn:ident),*) => {
        impl<$($tvarn: RawObj),*> RawObj for ($($tvarn,)*) {
            #[allow(unused_variables, unused_mut)]
            fn to_db(self) -> Vec<u8> {
                let ($(ident!(v $tvarn),)*) = self;
                let mut v = Vec::new();
                $(
                    let mut data = RawObj::to_db(ident!(v $tvarn));
                    v.append(&mut RawObj::to_db((data.len() as u64)));
                    v.append(&mut data);
                )*
                v
            }

            #[allow(unused_assignments, unused_variables, unused_mut)]
            fn from_db(mut x: Vec<u8>) -> Option<Self> {
                Some((
                    $(
                        {
                            let len = <u64 as RawObj>::from_db(x[0..8].to_vec())? as usize;
                            x = x[8..].to_vec();
                            let value = <$tvarn as RawObj>::from_db(x[0..len].to_vec())?;
                            x = x[len..].to_vec();
                            value
                        },
                    )*
                ))
            }
        }


        impl<$($tvarn: ComObj),*> ComObj for ($($tvarn,)*) {
            #[allow(unused_variables, unused_mut)]
            fn to_db<P: Path>(self, path: P, db: &MicroDB) -> Result<(), io::Error> {
                let ($(ident!(v $tvarn),)*) = self;
                let mut i = 0_u64;
                $(
                    db.set_com(path.sub_path(i), ident!(v $tvarn))?;
                    i += 1;
                )*
                db.set_raw_hard(path, i)
            }

            fn remove<P: Path>(path: P, db: &MicroDB) -> Result<(), io::Error> {
                <Vec<()> as ComObj>::remove::<P>(path, db) // tuples have identical layout to vecs
            }

            #[allow(unused_variables, unused_mut)]
            fn from_db<P: Path>(path: P, db: &MicroDB) -> Result<Option<Self>, io::Error> {
                let mut i = 0_u64;
                let tup = (
                    $(
                        {
                            let value = extract!(db.get_com::<$tvarn, _>(path.sub_path(i)));
                            i += 1;
                            value
                        },
                    )*
                );
                if matches!(db.get_raw(path), Ok(Some(x)) if i == x) {
                    Ok(Some(tup))
                } else {
                    Ok(None)
                }
            }

            fn paths<P: Path>(path: P, db: &MicroDB) -> Result<Vec<String>, io::Error> {
                <Vec<()> as ComObj>::paths::<P>(path, db) // tuples have identical layout to vecs
            }
        }
    };
}

impl_tuple!();
impl_tuple!(A);
impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);
impl_tuple!(A, B, C, D, E);
impl_tuple!(A, B, C, D, E, F);
impl_tuple!(A, B, C, D, E, F, G);
impl_tuple!(A, B, C, D, E, F, G, H);
impl_tuple!(A, B, C, D, E, F, G, H, I);
impl_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, Q); // P cannot be used, skipping it
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, Q, R);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, Q, R, S);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, Q, R, S, T);

// code so unreadable, im actually gonna have to make tests

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::MicroDB;

    #[test]
    fn test_0() {
        let db = MicroDB::create("tuples.test0.dmdb", "tuples.test0.mmdb", 100, 100).unwrap();
        db.set_raw("0tuple", ()).unwrap();
        assert_eq!(db.get_raw("0tuple").unwrap() as Option<()>, None);
        db.shutdown().unwrap();
        fs::remove_file("tuples.test0.dmdb").unwrap();
        fs::remove_file("tuples.test0.mmdb").unwrap();
    }
    #[test]
    fn test_5() {
        let db = MicroDB::create("tuples.test5.dmdb", "tuples.test5.mmdb", 100, 100).unwrap();
        db.set_raw("5tuple", (1u8, 2u16, 3u8, 4u32, 5u128)).unwrap();
        assert_eq!(
            db.get_raw("5tuple").unwrap(),
            Some((1u8, 2u16, 3u8, 4u32, 5u128))
        );
        db.shutdown().unwrap();
        fs::remove_file("tuples.test5.dmdb").unwrap();
        fs::remove_file("tuples.test5.mmdb").unwrap();
    }
    #[test]
    fn test_10() {
        let db = MicroDB::create("tuples.test10.dmdb", "tuples.test10.mmdb", 100, 100).unwrap();
        db.set_raw(
            "10tuple",
            (
                1u8,
                "hii".to_owned(),
                3u8,
                4u32,
                5u128,
                "6".to_owned(),
                "7".to_owned(),
                8u8,
                9i32,
                10i128,
            ),
        )
        .unwrap();
        assert_eq!(
            db.get_raw("10tuple").unwrap(),
            Some((
                1u8,
                "hii".to_owned(),
                3u8,
                4u32,
                5u128,
                "6".to_owned(),
                "7".to_owned(),
                8u8,
                9i32,
                10i128
            ))
        );
        db.shutdown().unwrap();
        fs::remove_file("tuples.test10.dmdb").unwrap();
        fs::remove_file("tuples.test10.mmdb").unwrap();
    }
    #[test]
    fn test_10_com() {
        let db = MicroDB::create("tuples.test10c.dmdb", "tuples.test10c.mmdb", 100, 100).unwrap();
        db.set_com(
            "10tuple",
            (
                1u8,
                "hii".to_owned(),
                3u8,
                4u32,
                5u128,
                "6".to_owned(),
                "7".to_owned(),
                8u8,
                9i32,
                10i128,
            ),
        )
        .unwrap();
        assert_eq!(
            db.get_com("10tuple").unwrap(),
            Some((
                1u8,
                "hii".to_owned(),
                3u8,
                4u32,
                5u128,
                "6".to_owned(),
                "7".to_owned(),
                8u8,
                9i32,
                10i128
            ))
        );
        db.shutdown().unwrap();
        fs::remove_file("tuples.test10c.dmdb").unwrap();
        fs::remove_file("tuples.test10c.mmdb").unwrap();
    }
}
