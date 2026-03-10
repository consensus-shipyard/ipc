#!/usr/bin/env python3
"""
Calculate the EVM chain ID for an IPC subnet.

This mimics the Rust implementation in ipc/api/src/subnet_id.rs:
```rust
pub fn chain_id(&self) -> u64 {
    if self.is_root() {
        return self.root_id();
    }
    let mut hasher = FnvHasher::default();
    hasher.write(self.to_string().as_bytes());
    hasher.finish() % MAX_CHAIN_ID
}
```

The FNV-1a hash algorithm is used to generate a deterministic chain ID
from the subnet ID string.
"""

import sys

# FNV-1a hash algorithm constants
FNV_OFFSET_BASIS = 0xcbf29ce484222325
FNV_PRIME = 0x100000001b3

# Maximum chain ID (same as in Rust implementation)
MAX_CHAIN_ID = (1 << 32) - 1  # 2^32 - 1


def fnv1a_hash(data: bytes) -> int:
    """
    Compute FNV-1a 64-bit hash of the input data.

    FNV-1a algorithm:
    1. Start with offset basis
    2. For each byte: XOR with byte, then multiply by FNV prime
    """
    hash_value = FNV_OFFSET_BASIS

    for byte in data:
        hash_value ^= byte
        hash_value = (hash_value * FNV_PRIME) & 0xffffffffffffffff  # Keep it 64-bit

    return hash_value


def calculate_chain_id(subnet_id: str) -> int:
    """
    Calculate the EVM chain ID for a subnet.

    Args:
        subnet_id: The subnet ID string (e.g., "/r31337/t410fwwa...")

    Returns:
        The calculated chain ID as an integer
    """
    # Check if it's a root network (only /r<number>)
    if subnet_id.startswith('/r') and subnet_id.count('/') == 1:
        # Root network - extract the number
        return int(subnet_id[2:])

    # For child subnets, hash the full subnet ID
    subnet_bytes = subnet_id.encode('utf-8')
    hash_value = fnv1a_hash(subnet_bytes)

    # Take modulo MAX_CHAIN_ID to fit in valid range
    chain_id = hash_value % MAX_CHAIN_ID

    return chain_id


def main():
    if len(sys.argv) != 2:
        print("Usage: calculate_chain_id.py <subnet-id>", file=sys.stderr)
        print("Example: calculate_chain_id.py /r31337/t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia", file=sys.stderr)
        sys.exit(1)

    subnet_id = sys.argv[1]
    chain_id = calculate_chain_id(subnet_id)

    # Output only the chain ID (for use in scripts)
    print(chain_id)


if __name__ == '__main__':
    main()
