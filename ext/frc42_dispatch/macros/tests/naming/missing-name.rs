use frc42_macros::method_hash;

fn main() {
	// should panic because no string or identifier provided
    let _ident_hash = method_hash!();
}
