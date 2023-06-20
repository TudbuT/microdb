# MicroDB

A microsized database for use in programs with too much data for the RAM.

## Completed features

- [x] Disk storage
- [x] Getting, setting, allocation, deallocation
- [x] Caching
- [x] Automatic recovery on error
- [x] Serialization for basic types (numbers, strings, vecs, options, results)
- [x] Easy-to-implement serialization
- [ ] Derivable serialization
- [ ] (maybe) Multi-client support over TCP
- [ ] (maybe) Mirroring operations to backup server (needs TCP)

## How to use it

MicroDB runs where your application does: Saving, cache synchronization, etc all happen simply in another thread of your application.

To get started, create a DB:
```rs
let db = MicroDB::create(
    "example_db.data.mdb",
    "example_db.meta.mdb",
    MicroDB::sensible_cache_period(
        /*requests of unique objects per second*/10.0, 
        /*max ram usage*/0.1, 
        /*average object size in mb*/0.01, 
        /*safety (how important staying within ram spec is)*/1.0),
    MicroDB::sensible_block_size(
        /*object amount*/500.0, 
        /*average object size in bytes*/10_0000.0, 
        /*object size fluctuation in bytes*/0.0, 
        /*storage tightness*/1.0
    ),
)
```
Or load one using ::new and leave out the block_size arg.

And now you're good to go!

# Is it any fast?

Here's a test showing the speed with amount of requests to one value:
```
Setting test --raw--> true
Reading test 10000 times.
Done! Took 1ms: 0.0001ms per read.
```

Here's a test showing the speed with one request per value at 10000 values:
```
Setting horizontal_test/{0..10000} --raw--> true
Reading back all values...
Done! Write took 5570ms: 0.557ms per write; Read took 143ms: 0.0143ms per read.
```

As you can see, the speed is quite negigible, and it actually happens to be a lot faster
than SQL databases like Postgres **for these kinds of dataset sizes**. This DB is not made to
be used on datasets of giant sizes, but it works exceptionally well for smaller datasets.
