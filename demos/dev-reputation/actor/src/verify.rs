use libsecp256k1::{recover, Message, RecoveryId, Signature};
use tiny_keccak::{Hasher, Keccak};

fn keccak256(input: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut keccak = Keccak::v256();
    keccak.update(input);
    keccak.finalize(&mut output);
    output
}

pub fn prefixed_hash(document_hash: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"\x19Ethereum Signed Message:\n32".to_vec();
    bytes.extend_from_slice(&document_hash);
    keccak256(&bytes)
}

pub fn recover_address(document_hash: [u8; 32], signature: &[u8]) -> Result<[u8; 20], String> {
    if signature.len() != 65 {
        return Err("invalid signature length".to_string());
    }
    let mut sig = [0u8; 64];
    sig.copy_from_slice(&signature[0..64]);
    let mut v = signature[64];
    if v >= 27 {
        v -= 27;
    }
    if v > 1 {
        return Err("invalid recovery id".to_string());
    }

    let recovery_id = RecoveryId::parse(v).map_err(|_| "invalid recovery id".to_string())?;
    let standard_sig = Signature::parse_standard_slice(&sig).map_err(|_| "invalid signature".to_string())?;
    let msg = Message::parse(&prefixed_hash(document_hash));
    let pubkey = recover(&msg, &standard_sig, &recovery_id).map_err(|_| "recover failed".to_string())?;
    let serialized = pubkey.serialize();
    let hashed = keccak256(&serialized[1..]);
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hashed[12..]);
    Ok(addr)
}
