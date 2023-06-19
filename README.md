# MicroDB

A microsized database for use in programs with too much data for the RAM, but not necessarily for your
next incredibly successful Discord clone (tho I suppose you could make that work too\*).

\* So it turns out when I compared this against postgres in terms of speed, THIS WON BY MILES. And by miles,
   I mean a factor of about 16 (0.067ms vs 0.0004ms).

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
