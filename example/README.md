# fil-actor-example
Sample fvm actor based on `fvm-utils`. To create your own user defined actor, define 
`Actor` that implements `ActorCode` trait. At the same time, you can create the wasm 
entrypoint with the following short cut:
```rust
#[no_mangle]
pub fn invoke(param: u32) -> u32 {
    runtime::fvm::trampoline::<Actor>(param)
}
```
See `src/lib.rs` for more details.

## Build
To compile:
```shell
cargo build
```
You should be able to see the `fil_actor_example.compact.wasm` compiled generated.

Set up a local fvm according to this [tutorial](https://lotus.filecoin.io/lotus/developers/local-network/).

## Test
To trigger unit tests, perform the following:
```shell
cargo test
```