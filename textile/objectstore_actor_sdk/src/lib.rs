mod sys;

use fvm_shared::error::ErrorNumber;

pub fn cid_rm(cid: Vec<u8>) -> Result<(), ErrorNumber> {
    unsafe { sys::cid_rm(cid.as_ptr(), cid.len() as u32) }
}
