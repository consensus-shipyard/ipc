use frc42_macros::method_hash;

fn main() {
	// should panic because the name starts with non-capital letter
	let _str_hash = method_hash!("noPlaceForCamelCase");
}
