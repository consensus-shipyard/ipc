use fvm_sdk::sys::fvm_syscalls;

fvm_syscalls! {
    module = "objectstore";

    pub fn load_car(content: *const u8, content_len: u32) -> Result<()>;
}
