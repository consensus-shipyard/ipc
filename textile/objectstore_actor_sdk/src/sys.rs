use fvm_sdk::sys::fvm_syscalls;

fvm_syscalls! {
    module = "objectstore";

    pub fn cid_rm(cid: *const u8, cid_len: u32) -> Result<()>;
}
