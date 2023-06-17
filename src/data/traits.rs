use std::marker::PhantomData;

pub trait Path<T> {
    fn get_type(&self) -> PhantomData<T>;
    fn to_db_path(&self) -> String;
}

pub trait Obj: Sized {
    fn to_db_object(self) -> Vec<u8>;
    fn map(x: Vec<u8>) -> Option<Self>;
}
