// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::io::Write;

use erasure_encoding::{
    decode_chunks, encode_and_assign, ChunkRecoveryInput, ErasureError, NodeId, ReedSolomonEncoder,
    RotatingAssigner,
};
use rand::Rng;

fn make_nodes(n: usize) -> Vec<NodeId> {
    (0..n)
        .map(|i| {
            let mut id = [0u8; 32];
            id[0] = i as u8;
            NodeId(id)
        })
        .collect()
}

/// Full end-to-end with mmap: write 100 MB random file, encode via mmap (zero-copy),
/// simulate shard losses, decode, verify via blake3 hash.
#[test]
fn end_to_end_mmap_large_file() {
    let k = 15; // data shards
    let m = 8; // parity shards
    let nodes = make_nodes(30);
    let file_size = 100 * 1024 * 1024; // 100 MB

    // 1. Write random data to a temp file.
    let mut tmpfile = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let mut rng = rand::thread_rng();
    let mut buf = vec![0u8; 1024 * 1024]; // write 1 MB at a time
    let mut hasher = blake3::Hasher::new();
    let mut written = 0;
    while written < file_size {
        let chunk = (file_size - written).min(buf.len());
        rng.fill(&mut buf[..chunk]);
        tmpfile.write_all(&buf[..chunk]).unwrap();
        hasher.update(&buf[..chunk]);
        written += chunk;
    }
    tmpfile.flush().unwrap();
    let original_hash = hasher.finalize();

    // 2. Memory-map the file (zero-copy, OS pages in/out on demand).
    let mmap = unsafe { memmap2::Mmap::map(tmpfile.as_file()).expect("mmap failed") };
    assert_eq!(mmap.len(), file_size);

    // 3. Encode and assign shards to nodes.
    let (metadata, chunk_iter) =
        encode_and_assign::<ReedSolomonEncoder, _>(&mmap, k, m, &nodes, RotatingAssigner::new())
            .expect("encoding should succeed");

    assert_eq!(metadata.original_len, file_size);
    assert_eq!(metadata.data_shards, k);
    assert_eq!(metadata.parity_shards, m);

    let encoded_chunks: Vec<_> = chunk_iter
        .collect::<Result<Vec<_>, _>>()
        .expect("all chunks should encode");

    // Each chunk should have k + m shards.
    for chunk in &encoded_chunks {
        assert_eq!(chunk.shards.len(), k + m);
    }

    // 4. Simulate losing shards: keep 8 original data shards + 7 parity shards = 15 total.
    //    Drop original data shards 0..7 (7 shards) and parity shard at index k (1 shard).
    //    That leaves 8 originals (indices 8..14) + 7 parities (indices 16..22) = 15 >= k.
    let drop_indices: Vec<usize> = (0..7).chain(std::iter::once(k)).collect();
    let mut recovery_inputs: Vec<ChunkRecoveryInput> = encoded_chunks
        .iter()
        .map(|ec| {
            let surviving_shards = ec
                .shards
                .iter()
                .filter(|a| !drop_indices.contains(&a.shard.index))
                .map(|a| a.shard.clone())
                .collect();
            ChunkRecoveryInput {
                chunk_index: ec.chunk_index,
                original_data_len: ec.original_data_len,
                shards: surviving_shards,
                num_data_shards: k,
                num_parity_shards: m,
            }
        })
        .collect();

    // Verify we kept the right number: 23 total - 8 dropped = 15 surviving.
    for input in &recovery_inputs {
        assert_eq!(input.shards.len(), k + m - drop_indices.len());
    }

    // 5. Decode and recover.
    let recovered =
        decode_chunks::<ReedSolomonEncoder>(&mut recovery_inputs, metadata.original_len)
            .expect("decoding should succeed");

    // 6. Verify via blake3 hash.
    assert_eq!(recovered.len(), file_size);
    let recovered_hash = blake3::hash(&recovered);
    assert_eq!(recovered_hash, original_hash, "hash mismatch after decode");
}

/// Empty nodes should be rejected early.
#[test]
fn empty_nodes_rejected() {
    let data = vec![1u8; 100];
    let result =
        encode_and_assign::<ReedSolomonEncoder, _>(&data, 2, 1, &[], RotatingAssigner::new());
    assert!(matches!(result, Err(ErasureError::NotEnoughNodes { .. })));
}
