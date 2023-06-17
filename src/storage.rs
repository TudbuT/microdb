use std::{
    collections::HashMap,
    fs::{self, File},
    hint::black_box,
    io::{self, ErrorKind, Read, Seek, SeekFrom, Write},
    mem, process,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use deborrow::deborrow;

macro_rules! serialize_u64 {
    ($f:ident, $thing:expr) => {
        $f.write_all(&u64::to_be_bytes($thing as u64))
    };
}
macro_rules! deserialize_u64 {
    ($f:ident, $buf64:ident) => {
        (
            $f.read_exact(&mut $buf64)?,
            u64::from_be_bytes($buf64) as usize,
        )
            .1
    };
}

#[derive(Debug)]
struct Allocation {
    full_size: usize,
    locations: Vec<(usize, usize)>, // start, length
}

#[derive(Debug)]
struct AllocationTable {
    filename: String,
    block_size: usize,
    blocks_reserved: usize,
    free: Vec<(usize, usize)>,
    map: HashMap<String, Allocation>,
}

#[derive(Debug)]
struct InnerFAlloc {
    cache_period: u128,
    data: File,
    alloc: AllocationTable,
    cache: HashMap<String, (u128, bool, Vec<u8>)>,
    last_cache_check: u128,
    shutdown: bool,
}

#[derive(Debug)]
pub struct FAlloc {
    inner: Arc<Mutex<InnerFAlloc>>,
}

impl Allocation {
    fn get_data(&self, file: &mut File) -> Result<Vec<u8>, io::Error> {
        let mut bytes = vec![0_u8; self.full_size];
        let mut i = 0;
        for location in &self.locations {
            file.seek(SeekFrom::Start(location.0 as u64))?;
            file.read_exact(&mut bytes[i..(i + location.1).min(self.full_size)])?;
            i += location.1;
        }
        Ok(bytes)
    }
    fn set_data(&self, file: &mut File, mut data: Vec<u8>) -> Result<(), io::Error> {
        data.resize(self.full_size, 0);
        let mut i = 0;
        for location in &self.locations {
            file.seek(SeekFrom::Start(location.0 as u64))?;
            file.write_all(&data[i..(i + location.1).min(self.full_size)])?;
            i += location.1;
        }
        Ok(())
    }
}

impl AllocationTable {
    fn new(file: String) -> Result<Self, io::Error> {
        let mut buf64 = [0_u8; 8];
        let mut f = File::open(&file)?;
        let block_size = deserialize_u64!(f, buf64);
        let blocks_reserved = deserialize_u64!(f, buf64);
        let free_len = deserialize_u64!(f, buf64);
        let map_len = deserialize_u64!(f, buf64);
        let mut free = Vec::new();
        for _ in 0..free_len {
            free.push((deserialize_u64!(f, buf64), deserialize_u64!(f, buf64)));
        }
        let mut map = HashMap::new();
        for _ in 0..map_len {
            let str_len = deserialize_u64!(f, buf64);
            let mut buf = vec![0_u8; str_len];
            f.read_exact(&mut buf)?;
            let str = String::from_utf8(buf).expect("bitflip on drive??");
            let full_size = deserialize_u64!(f, buf64);
            let locs_len = deserialize_u64!(f, buf64);
            let mut locations = Vec::new();
            for _ in 0..locs_len {
                locations.push((deserialize_u64!(f, buf64), deserialize_u64!(f, buf64)));
            }
            map.insert(
                str,
                Allocation {
                    full_size,
                    locations,
                },
            );
        }
        Ok(Self {
            filename: file,
            block_size,
            blocks_reserved,
            free,
            map,
        })
    }

    fn alloc(&mut self, amount: usize, file: &mut File) -> Result<(usize, usize), io::Error> {
        println!("Allocating {amount} bytes:");
        let amount = ((amount - 1) / self.block_size + 1) * self.block_size;
        println!("Real alloc = {amount}.");
        // try to reclaim old space
        if let Some((loc, &x)) = self.free.iter().enumerate().find(|x| x.1 .1 >= amount) {
            println!("Reclaiming.");
            self.free.remove(loc);
            if (x.1 - amount) / self.block_size > 0 {
                self.free.push((
                    // location + amount.round_up_to(block_size)
                    x.0 + ((amount - 1) / self.block_size + 1) * self.block_size,
                    // size - amount.round_down_to(block_size)
                    x.1 - amount / self.block_size * self.block_size,
                ))
            }
            println!("Free remaining: {:?}", self.free);
            println!("Alloc success: {} - {} claimed.", x.0, amount);
            return Ok((x.0, amount));
        }
        println!("Creating new space.");
        // otherwise find new place
        let start = self.blocks_reserved * self.block_size;
        let amount_blocks = amount / self.block_size;
        println!("Allocating {amount_blocks} blocks.");
        file.seek(SeekFrom::Start(start as u64))?;
        file.write_all(&vec![0_u8; amount_blocks * self.block_size])?;
        self.blocks_reserved += amount_blocks;
        println!(
            "Alloc success: {start} - {} claimed.",
            amount_blocks * self.block_size
        );
        Ok((start, amount_blocks * self.block_size))
    }
    fn dealloc(&mut self, alloc: (usize, usize)) {
        println!("Deallocating {} bytes at {}", alloc.1, alloc.0);
        let amount = ((alloc.1 - 1) / self.block_size + 1) * self.block_size;
        println!("Real amount of bytes deallocated = {amount}");
        self.free
            // round size up to block size
            .push((alloc.0, amount));
        println!("Dealloc successful.")
    }

    fn set_allocation_length(
        &mut self,
        allocation: &mut Allocation,
        file: &mut File,
        needed: usize,
    ) -> Result<(), io::Error> {
        if needed == allocation.full_size {
            return Ok(());
        }
        if allocation.full_size == 0 && needed > 0 {
            allocation.locations.push(self.alloc(needed, file)?);
            allocation.full_size = needed;
            return Ok(());
        }
        if needed == 0 {
            for loc in &allocation.locations {
                self.dealloc(*loc);
            }
            allocation.full_size = 0;
            allocation.locations.clear();
            return Ok(());
        }
        let current_blocked_size = (allocation.full_size - 1) / self.block_size + 1;
        // can we change without (de)allocation?
        if (allocation.full_size + needed - 1) / self.block_size + 1 == current_blocked_size {
            allocation.full_size = needed;
            return Ok(());
        }

        if needed > allocation.full_size {
            let change = needed - allocation.full_size;
            let place = self.alloc(change, file)?;
            allocation.locations.push(place);
            allocation.full_size = needed;
        } else {
            while needed < allocation.full_size {
                let change = allocation.full_size - needed;
                if allocation.locations.last().unwrap().1 >= change {
                    // the entire thing can be removed
                    self.dealloc(allocation.locations.pop().unwrap());
                } else {
                    // start .. end - change is the range where data is still needed,
                    // so it follows end - change .. end is the range where it isnt.
                    let last = allocation.locations.last().unwrap();
                    let end = last.0 + last.1;
                    let begin_dealloc = end - change;
                    let begin_dealloc =
                        ((begin_dealloc - 1) / self.block_size + 1) * self.block_size;
                    self.dealloc((begin_dealloc, end - begin_dealloc));
                }
            }
        }
        Ok(())
    }

    fn save(&mut self) -> Result<(), io::Error> {
        println!("Saving {self:?}");
        let mut file = File::create(self.filename.to_owned() + ".tmp")?;
        serialize_u64!(file, self.block_size)?;
        serialize_u64!(file, self.blocks_reserved)?;
        serialize_u64!(file, self.free.len())?;
        serialize_u64!(file, self.map.len())?;
        for item in &self.free {
            serialize_u64!(file, item.0)?;
            serialize_u64!(file, item.1)?;
        }
        for item in &self.map {
            serialize_u64!(file, item.0.as_bytes().len())?;
            file.write_all(item.0.as_bytes())?;
            serialize_u64!(file, item.1.full_size)?;
            serialize_u64!(file, item.1.locations.len())?;
            for location in &item.1.locations {
                serialize_u64!(file, location.0)?;
                serialize_u64!(file, location.1)?;
            }
        }
        fs::rename(self.filename.to_owned() + ".tmp", &self.filename)
    }
}

impl InnerFAlloc {
    fn flush_cache(&mut self) -> Result<u128, io::Error> {
        let time = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis();
        if time - self.last_cache_check > 100 {
            self.last_cache_check = time;
            for item in self.cache.iter_mut() {
                if item.1 .1 && time - item.1 .0 >= self.cache_period {
                    let allocation = unsafe { deborrow(self.alloc.map.get_mut(item.0).unwrap()) };
                    self.alloc.set_allocation_length(
                        allocation,
                        &mut self.data,
                        item.1 .2.len(),
                    )?;
                    allocation.set_data(&mut self.data, item.1 .2.clone())?;
                    item.1 .1 = false;
                    if allocation.full_size == 0 {
                        self.alloc.map.remove(item.0);
                        item.1 .0 = 0;
                        continue;
                    }
                }
            }
        }
        Ok(time)
    }
}

impl FAlloc {
    fn internal_new(
        data: File,
        alloc: AllocationTable,
        cache_period: u128,
    ) -> Result<Self, io::Error> {
        let inner = Arc::new(Mutex::new(InnerFAlloc {
            cache_period,
            data,
            alloc,
            cache: HashMap::new(),
            last_cache_check: 0,
            shutdown: false,
        }));
        let inner_clone = inner.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            let mut recovery = false;
            loop {
                if inner_clone.is_poisoned() {
                    println!(
                        "SEVERE: The database mutex was poisoned. Data may be corrupt. {}", 
                        "Clearing poison and attempting to write to disk one last time before shutting down."
                    );
                    println!(
                        "First, waiting 60 seconds for rest of program to crash if possible..."
                    );
                    thread::sleep(Duration::from_secs(60));
                    println!("Circumventing poison and attempting recovery.");
                    recovery = true;
                }
                let mut inner = inner_clone.lock().unwrap_or_else(|x| x.into_inner());
                if recovery {
                    inner.shutdown = true;
                    if let Err(e) = inner.alloc.save() {
                        println!("The database was unable to write *critical* data to disk. DO NOT END THE PROGRAM. Error: {e:?}. Recovery attempts happen every 10 seconds.");
                        thread::sleep(Duration::from_secs(10));
                        continue;
                    }
                }
                if let Err(e) = inner.flush_cache().and(inner.data.sync_all()) {
                    inner.shutdown = true;
                    recovery = true;
                    println!("The database was unable to write to disk. Depending on where this error happened, your data might be mostly fine. Error: {e:?}. Recovery will be attempted every 30 seconds.");
                    thread::sleep(Duration::from_secs(30));
                    continue;
                }
                if inner.shutdown {
                    inner.shutdown = false;
                    if recovery {
                        println!("Recovery seems to have been successful. HALTING THE PROGRAM IN ORDER TO PREVENT FURTHER DAMAGE.");
                        println!("Poisoning mutex just in case any threads still try to use it.");
                        let inner_clone = inner_clone.clone();
                        thread::spawn(move || {
                            let thing = inner_clone.lock().unwrap();
                            if 1 == 1 {
                                panic!("Poisoning mutex intentionally.");
                            }
                            mem::drop(black_box(thing));
                        });
                        println!("Sleeping for 2 hours, then exiting.");
                        thread::sleep(Duration::from_secs(3600 * 2));
                        process::exit(255);
                    }
                    break;
                }
                let d = inner.cache_period;
                mem::drop(inner);
                thread::sleep(Duration::from_millis((d * 10) as u64));
            }
        });
        Ok(Self { inner })
    }

    pub fn new<S: ToString>(data: S, alloc: S, cache_period: u128) -> Result<Self, io::Error> {
        Self::internal_new(
            File::options()
                .read(true)
                .write(true)
                .truncate(false)
                .create(false)
                .open(data.to_string())?,
            AllocationTable::new(alloc.to_string())?,
            cache_period,
        )
    }

    pub fn create<S: ToString>(
        data: S,
        alloc: S,
        cache_period: u128,
        block_size: usize,
    ) -> Result<Self, io::Error> {
        Self::internal_new(
            File::options()
                .read(true)
                .write(true)
                .create_new(true)
                .open(data.to_string())?,
            AllocationTable {
                filename: alloc.to_string(),
                block_size,
                blocks_reserved: 0,
                free: Vec::new(),
                map: HashMap::new(),
            },
            cache_period,
        )
    }

    pub fn cache_lookup(&self, path: Option<&str>) -> Result<Option<Vec<u8>>, io::Error> {
        let mut this = self.inner.lock().unwrap();
        if this.shutdown {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "The database has shut down. Writes are prohibited. If you didn't do this, some kind of error was encountered that forced the DB to shut down. Recovery will be attempted at regular intervals."));
        }
        let time = this.flush_cache()?;
        if let Some(path) = path {
            Ok(this
                .cache
                .get_mut(path)
                .map(|x| (x.0 = time, x.2.to_owned()).1)
                // empty data can only exist in cache, so we only need this condition in the cache.
                .and_then(|x| if x.is_empty() { None } else { Some(x) }))
        } else {
            Ok(None)
        }
    }

    pub fn get(&self, path: &str) -> Result<Option<Vec<u8>>, io::Error> {
        if let Some(x) = self.cache_lookup(Some(path))? {
            return Ok(Some(x));
        }

        let mut this = self.inner.lock().unwrap();
        if this.shutdown {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "The database has shut down. Writes are prohibited. If you didn't do this, some kind of error was encountered that forced the DB to shut down. Recovery will be attempted at regular intervals."));
        }
        let (cache, alloc, data) = deborrow!(this: cache, alloc, data);
        let time = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis();
        alloc
            .map
            .get(path)
            .map(|x| {
                // get data, cache, and return it
                x.get_data(data).map(|x| {
                    (
                        cache.insert(path.to_owned(), (time, false, x.clone())),
                        Some(x),
                    )
                        .1
                })
            })
            .unwrap_or(Ok(None))
    }

    pub fn set(&self, path: &str, data: Vec<u8>) -> Result<(), io::Error> {
        let mut this = self.inner.lock().unwrap();
        if this.shutdown {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "The database has shut down. Writes are prohibited. If you didn't do this, some kind of error was encountered that forced the DB to shut down. Recovery will be attempted at regular intervals."));
        }
        let (cache, alloc) = deborrow!(this: cache, alloc);
        let time = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis();
        cache.insert(path.to_owned(), (time, true, data));
        if !alloc.map.contains_key(path) {
            alloc.map.insert(
                path.to_owned(),
                Allocation {
                    full_size: 0,
                    locations: Vec::new(),
                },
            );
        }
        Ok(())
    }

    pub fn sync(&self) -> Result<(), io::Error> {
        let mut this = self.inner.lock().unwrap();
        this.last_cache_check = 0;
        for item in this.cache.iter_mut() {
            item.1 .0 = 0;
        }
        this.flush_cache()?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), io::Error> {
        self.sync()?;
        let mut this = self.inner.lock().unwrap();
        this.alloc.save()?;
        this.data.sync_all()
    }

    pub fn shutdown(self) -> Result<(), io::Error> {
        self.save()?;
        self.inner.lock().unwrap().shutdown = true;
        while self.inner.lock().unwrap().shutdown {
            thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::storage::FAlloc;

    #[test]
    fn main() {
        let _ = fs::remove_file("test.dat");
        let _ = fs::remove_file("test.alloc");
        create();
        load();
        delete_val();
        create_new_val();
    }
    fn create() {
        let db = FAlloc::create("test.dat", "test.alloc", 500, 256).unwrap();
        db.set("test", vec![40; 400]).unwrap();
        assert_eq!(db.get("test").unwrap().unwrap(), vec![40_u8; 400]);
        db.sync().unwrap();
        db.set("lol", vec![51; 512]).unwrap();
        assert_eq!(db.get("lol").unwrap().unwrap(), vec![51_u8; 512]);
        db.sync().unwrap();
        db.shutdown().unwrap();
    }
    fn load() {
        let db = FAlloc::new("test.dat", "test.alloc", 500).unwrap();
        assert_eq!(db.get("test").unwrap().unwrap(), vec![40_u8; 400]);
        db.shutdown().unwrap();
    }
    fn delete_val() {
        let db = FAlloc::new("test.dat", "test.alloc", 500).unwrap();
        db.set("test", vec![0; 0]).unwrap();
        assert_eq!(db.get("test").unwrap().unwrap(), vec![0; 0]);
        db.shutdown().unwrap();
    }
    fn create_new_val() {
        let db = FAlloc::new("test.dat", "test.alloc", 500).unwrap();
        db.set("test2", vec![40; 200]).unwrap();
        assert_eq!(db.get("test2").unwrap().unwrap(), vec![40_u8; 200]);
        db.sync().unwrap();
        db.set("lol2", vec![51; 212]).unwrap();
        assert_eq!(db.get("lol2").unwrap().unwrap(), vec![51_u8; 212]);
        db.sync().unwrap();
        db.shutdown().unwrap();
    }
}
