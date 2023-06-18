use std::{fs, hint::black_box, time::SystemTime};

use microdb::MicroDB;

fn main() {
    let _ = fs::remove_file("example_db.data.mdb");
    let _ = fs::remove_file("example_db.meta.mdb");
    let db = MicroDB::create(
        "example_db.data.mdb",
        "example_db.meta.mdb",
        0,
        // MicroDB::sensible_cache_period(10.0, 0.1, 0.1, 1.0),
        // 1,
        MicroDB::sensible_block_size(500.0, 2.0, 0.0, 1.0),
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
    db.remove_com::<_, Vec<bool>>("test").unwrap();
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
    db.remove_com::<_, Vec<bool>>("test").unwrap();
    db.sync().unwrap();
    db.shutdown().unwrap();
}
