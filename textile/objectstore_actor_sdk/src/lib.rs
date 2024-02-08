mod sys;

use fvm_shared::error::ErrorNumber;

pub fn load_car(content: String) -> Result<(), ErrorNumber> {
    unsafe {
        let cid = content.as_bytes();
        sys::load_car(cid.as_ptr(), cid.len() as u32)
    }
}
