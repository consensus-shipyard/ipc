use frc42_macros::method_hash;

fn main() {
	// should panic because the name contains illegal chars
	let _str_hash = method_hash!("Bad!Method!Name!");
}
