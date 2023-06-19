use crate::com_obj;

use super::{Path, RawObj};

macro_rules! impl_obj_num {
    ($($e:expr => $($t:ty),+ ;)+) => { $( $(
        impl RawObj for $t {
            fn to_db(self) -> Vec<u8> {
                Vec::from(self.to_be_bytes())
            }

            fn from_db(x: Vec<u8>) -> Option<Self> {
                if x.len() == $e as usize / 8 {
                    let mut buf = [0_u8; $e as usize / 8];
                    buf.copy_from_slice(x.as_slice());
                    Some(Self::from_be_bytes(buf))
                } else {
                    None
                }
            }
        }
        com_obj!($t);

        impl Path for $t {
            fn to_db_path(self) -> String {
                String::from_utf16(&self.to_be_bytes().map(|x| x as u16)).unwrap()
            }
        }
    )+ )+ };
}

impl_obj_num! {
    Self::BITS => u8, u16, u32, u64, u128, i8, i16, i32, i64, i128;
    32 => f32;
    64 => f64;
}

impl RawObj for bool {
    fn to_db(self) -> Vec<u8> {
        if self {
            1_u8.to_db()
        } else {
            0_u8.to_db()
        }
    }

    fn from_db(x: Vec<u8>) -> Option<Self> {
        u8::from_db(x).and_then(|x| match x {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        })
    }
}
com_obj!(bool);
