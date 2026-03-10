/// Opaque 32-byte node identifier, decoupled from iroh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub [u8; 32]);

/// A single shard with its positional index.
#[derive(Debug, Clone)]
pub struct Shard {
    pub index: usize,
    pub data: Vec<u8>,
}

/// A shard assigned to a specific storage node.
#[derive(Debug, Clone)]
pub struct AssignedShard {
    pub shard: Shard,
    pub node: NodeId,
}

/// One encoded chunk containing all its assigned shards.
#[derive(Debug, Clone)]
pub struct EncodedChunk {
    pub chunk_index: usize,
    pub original_data_len: usize,
    pub shards: Vec<AssignedShard>,
}

/// Metadata describing the encoding parameters, needed for decoding.
#[derive(Debug, Clone)]
pub struct EncodingMetadata {
    pub original_len: usize,
    pub num_chunks: usize,
    pub data_shards: usize,
    pub parity_shards: usize,
}
