# fvm-utils
This repo contains a series of crates that serve as utils for fvm development and testing.
Here are some simple breakdowns:
- runtime: Contains the runtime wrapper for communicating with `fvm`. It provides some 
handy utility functions such as `transaction` and `verification`.
- primitives: Contains typed version of `fvm` primitives such as `cid` and `hamt`.
- example: Contains a sample user defined actor using `runtime` and `primitive` crate which 
one can deploy to `fvm` as wasm binary.

For more details, please refer to each crate's `README.md`.