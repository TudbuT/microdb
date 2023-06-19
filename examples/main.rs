use std::{fs, hint::black_box, time::SystemTime};

use microdb::{
    data::{ComObj, Escape, Path},
    extract, MicroDB,
};

#[derive(Debug)]
struct User {
    username: String,
    email_address: String,
    password_hash: Vec<u8>,
}

impl ComObj for User {
    fn to_db<P: Path>(self, path: P, db: &MicroDB) -> Result<(), std::io::Error> {
        db.set_raw(path.sub_path("username"), self.username)?;
        db.set_raw(path.sub_path("email"), self.email_address)?;
        db.set_raw(path.sub_path("pass"), self.password_hash)?;
        Ok(())
    }

    fn remove<P: Path>(path: P, db: &MicroDB) -> Result<(), std::io::Error> {
        db.remove_raw(path.sub_path("username"))?;
        db.remove_raw(path.sub_path("email"))?;
        db.remove_raw(path.sub_path("pass"))?;
        Ok(())
    }

    fn from_db<P: Path>(path: P, db: &MicroDB) -> Result<Option<Self>, std::io::Error> {
        Ok(Some(Self {
            username: extract!(db.get_raw(path.sub_path("username"))),
            email_address: extract!(db.get_raw(path.sub_path("email"))),
            password_hash: extract!(db.get_raw(path.sub_path("pass"))),
        }))
    }
}

fn main() {
    let _ = fs::remove_file("example_db.data.mdb");
    let _ = fs::remove_file("example_db.meta.mdb");
    let db = MicroDB::create(
        "example_db.data.mdb",
        "example_db.meta.mdb",
        dbg!(MicroDB::sensible_cache_period(10.0, 0.01, 0.1, 1.0)),
        dbg!(MicroDB::sensible_block_size(500.0, 10_000.0, 0.0, 1.0)),
    )
    .unwrap();
    println!("\nSetting test --raw--> vec![true; 500]");
    db.set_raw("test", vec![true; 500]).unwrap();
    let v: Vec<bool> = db.get_raw("test").unwrap().unwrap();
    assert_eq!(v, vec![true; 500]);
    let time = SystemTime::now();
    println!("Reading test 10000 times.");
    for _ in 0..10000 {
        black_box::<Vec<bool>>(db.get_raw("test").unwrap().unwrap());
    }
    let elapsed = time.elapsed().unwrap().as_millis();
    println!(
        "Done! Took {}ms: {}ms per read.",
        elapsed,
        elapsed as f64 / 10000.0
    );
    println!("\nSetting test --raw--> vec![true; 5]");
    db.set_raw("test", vec![true; 5]).unwrap();
    let v: Vec<bool> = db.get_raw("test").unwrap().unwrap();
    assert_eq!(v, vec![true; 5]);
    let time = SystemTime::now();
    println!("Reading test 10000 times.");
    for _ in 0..10000 {
        black_box::<Vec<bool>>(db.get_raw("test").unwrap().unwrap());
    }
    let elapsed = time.elapsed().unwrap().as_millis();
    println!(
        "Done! Took {}ms: {}ms per read.",
        elapsed,
        elapsed as f64 / 10000.0
    );
    println!("\nSetting test --com--> vec![true; 500]");
    db.set_com("test", vec![true; 500]).unwrap();
    let v: Vec<bool> = db.get_com("test").unwrap().unwrap();
    assert_eq!(v, vec![true; 500]);
    let time = SystemTime::now();
    println!("Reading test 10000 times.");
    for _ in 0..10000 {
        black_box::<Vec<bool>>(db.get_com("test").unwrap().unwrap());
    }
    let elapsed = time.elapsed().unwrap().as_millis();
    println!(
        "Done! Took {}ms: {}ms per read.",
        elapsed,
        elapsed as f64 / 10000.0
    );
    println!("\nSetting test --com--> vec![true; 5]");
    db.remove_com::<Vec<bool>, _>("test").unwrap();
    db.set_com("test", vec![true; 5]).unwrap();
    let v: Vec<bool> = db.get_com("test").unwrap().unwrap();
    assert_eq!(v, vec![true; 5]);
    let time = SystemTime::now();
    println!("Reading test 10000 times.");
    for _ in 0..10000 {
        black_box::<Vec<bool>>(db.get_com("test").unwrap().unwrap());
    }
    let elapsed = time.elapsed().unwrap().as_millis();
    println!(
        "Done! Took {}ms: {}ms per read.",
        elapsed,
        elapsed as f64 / 10000.0
    );
    db.remove("test").unwrap();

    println!("\n\n-- benchmarks done --\n\n");

    // there is a slash and stuff in this username!
    let username = "TudbuT \\/\\ Daniella".to_owned();
    println!(
        "Escaped username: {}",
        Escape(username.clone()).to_db_path()
    );
    db.set_com(
        "users".sub_path(Escape(username.clone())),
        User {
            username: username.clone(),
            email_address: "rust-microdb@mail.tudbut.de".to_owned(),
            password_hash: vec![0; 32],
        },
    )
    .unwrap();
    let user = db
        .get_com::<User, _>("users".sub_path(Escape(username.clone())))
        .unwrap()
        .unwrap();
    println!("User is now {:?}", user);
    assert_eq!(user.username, username);
    assert_eq!(user.email_address, "rust-microdb@mail.tudbut.de".to_owned());
    assert_eq!(user.password_hash, vec![0; 32]);
    println!("That is correct.");

    db.sync().unwrap();
    db.shutdown().unwrap();
}
