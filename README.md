This repo implements a hash-based decoder for the nervosnetwork/rvv branch.
The hash function is generated using [minimum perfect hash function](https://github.com/ilanschnell/perfect-hash). 
For a nice intro see [here](http://ilan.schnell-web.net/prog/perfect-hash/).

To build the project:
```
git submodule init && git submodule update
cargo build ## This will generate a file called rvv_decoder.rs at src/

## test case
cargo test

## benchmark v.s if-else-based implementation
cargo bench
```

From the bench result, there is minor performance improvement. Because both the
benchmark and mphf are generated randomly, I have seen improvement ranging from 0 up to 20%.


## TODO
- [ ] Explore nicer and faster hash functions.
- [ ] The mphf is generated from a seed. Find a way to move randomness from the generation process (probably by removing salt).
