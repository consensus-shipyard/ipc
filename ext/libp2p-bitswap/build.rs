fn main() {
    #[cfg(feature = "compat")]
    prost_build::compile_protos(&["src/compat/bitswap_pb.proto"], &["src/compat"]).unwrap();
}
