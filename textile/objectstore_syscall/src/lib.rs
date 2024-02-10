pub mod objectstore_kernel;

use crate::objectstore_kernel::ObjectStoreOps;
use async_std::task;
use fvm::kernel::{IpldBlockOps, Result};
use fvm::syscalls::Context;
use fvm_ipld_car::CarReader;
// use multihash::Code;

pub const SYSCALL_MODULE_NAME: &str = "objectstore";
pub const SYSCALL_FUNCTION_NAME: &str = "load_car";

pub fn load_car(
    context: Context<'_, impl IpldBlockOps + ObjectStoreOps>,
    file_off: u32,
    file_len: u32,
) -> Result<()> {
    let data = context.memory.try_slice(file_off, file_len)?;
    let name = std::str::from_utf8(data).expect("failed to parse file name");

    // Create and link blocks
    task::block_on(async move {
        let file = async_std::fs::File::open(format!("/tmp/{}", name))
            .await
            .unwrap();
        let mut reader = CarReader::new(file).await.unwrap();
        while let Some(block) = reader.next_block().await.unwrap() {
            // let bid = context
            //     .kernel
            //     .block_create(block.cid.codec(), &block.data)
            //     .expect("failed to create block");
            // context
            //     .kernel
            //     .block_link(bid, Code::Blake2b256.into(), 32)
            //     .expect("failed to link block");
            context
                .kernel
                .block_add(block.cid, &block.data)
                .expect("failed to add block");
        }
    });

    Ok(())
}
