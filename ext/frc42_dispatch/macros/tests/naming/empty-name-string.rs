use frc42_macros::method_hash;

fn main() {
	// this should panic due to empty string
	let _str_hash = method_hash!("");
}
