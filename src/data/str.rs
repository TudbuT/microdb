use crate::com_obj;

use super::RawObj;

impl RawObj for String {
    fn to_db(self) -> Vec<u8> {
        self.into_bytes()
    }

    fn from_db(x: Vec<u8>) -> Option<Self> {
        String::from_utf8(x).ok()
    }
}
com_obj!(String);
