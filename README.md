This repo implements a hash-based decoder for the nervosnetwork/rvv branch.
The hash function is generated using [minimum perfect hash function](https://github.com/ilanschnell/perfect-hash). 
For a nice intro see [here](http://ilan.schnell-web.net/prog/perfect-hash/).

To build the project:
```
git submodule init && git submodule update
cargo build ## This will generate files called xxx_decoder.rs at src/

## test case
cargo test

## benchmark v.s if-else-based implementation
cargo bench
```
