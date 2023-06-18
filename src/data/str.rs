use super::RawObj;

impl RawObj for String {
    fn to_db(self) -> Vec<u8> {
        self.to_owned().into_bytes()
    }

    fn from_db(x: Vec<u8>) -> Option<Self> {
        String::from_utf8(x).ok()
    }
}
