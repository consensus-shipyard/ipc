use frc42_macros::method_hash;

fn main() {
    let str_hash = method_hash!("Method");
    println!("String hash: {str_hash:x}");

    // this one breaks naming rules and will fail to compile
    //println!("error hash: {}", method_hash!("some_function"));
}
