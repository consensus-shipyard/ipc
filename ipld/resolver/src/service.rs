use libipld::store::StoreParams;
use libp2p::Swarm;

use crate::behaviour::IpldResolver;

pub struct IpldResolverService<P: StoreParams> {
    swarm: Swarm<IpldResolver<P>>,
}

impl<P: StoreParams> IpldResolverService<P> {
    /// Start the swarm listening for incoming connections and drive the events forward.
    pub async fn run(self) -> anyhow::Result<()> {
        todo!("IPC-37")
    }
}
