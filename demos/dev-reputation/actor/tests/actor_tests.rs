use fvm_shared::address::Address;
use ipc_reputation_actor::{
    verify::prefixed_hash, ActorError, ReputationActor, SetScoreParams, EXIT_FORBIDDEN, EXIT_ILLEGAL_ARGUMENT,
};
use libsecp256k1::{sign, Message, PublicKey, SecretKey};
use tiny_keccak::{Hasher, Keccak};

fn keccak256(input: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut keccak = Keccak::v256();
    keccak.update(input);
    keccak.finalize(&mut output);
    output
}

fn secret_key() -> SecretKey {
    SecretKey::parse(&[7u8; 32]).expect("secret key")
}

fn ethereum_address(secret: &SecretKey) -> [u8; 20] {
    let public = PublicKey::from_secret_key(secret);
    let serialized = public.serialize();
    let hash = keccak256(&serialized[1..]);
    let mut out = [0u8; 20];
    out.copy_from_slice(&hash[12..]);
    out
}

fn signed_params(agent_secret: &SecretKey, developer: [u8; 20], score: u8) -> SetScoreParams {
    let document_hash = keccak256(b"demo-doc");
    let prefixed = prefixed_hash(document_hash);
    let message = Message::parse(&prefixed);
    let (sig, rid) = sign(&message, agent_secret);
    let mut signature = vec![0u8; 65];
    signature[..64].copy_from_slice(&sig.serialize());
    signature[64] = rid.serialize() + 27;

    SetScoreParams {
        developer,
        github_handle: "alice".to_string(),
        score,
        tier: "senior".to_string(),
        evidence_cid: "bafy-demo".to_string(),
        period: "2026-Q1".to_string(),
        document_hash,
        agent_address: ethereum_address(agent_secret),
        signature,
    }
}

fn must_err<T>(res: Result<T, ActorError>) -> ActorError {
    match res {
        Ok(_) => panic!("expected error"),
        Err(e) => e,
    }
}

#[test]
fn test_set_score() {
    let secret = secret_key();
    let agent_addr = ethereum_address(&secret);
    let mut actor = ReputationActor::constructor(Address::new_id(1000), agent_addr);

    let developer = [1u8; 20];
    let params = signed_params(&secret, developer, 88);
    actor
        .set_score(agent_addr, params.clone(), 1234, 1710850000)
        .expect("set score should pass");

    let record = actor.get_score(developer).expect("record must exist");
    assert_eq!(record.score, 88);
    assert_eq!(record.evidence_cid, params.evidence_cid);
    assert_eq!(record.agent_address, agent_addr);
}

#[test]
fn test_invalid_signature() {
    let secret = secret_key();
    let agent_addr = ethereum_address(&secret);
    let mut actor = ReputationActor::constructor(Address::new_id(1000), agent_addr);

    let mut params = signed_params(&secret, [2u8; 20], 70);
    params.signature[10] ^= 0xff;
    let err = must_err(actor.set_score(agent_addr, params, 10, 20));
    assert_eq!(err.code, EXIT_ILLEGAL_ARGUMENT);
}

#[test]
fn test_unauthorised_agent() {
    let secret = secret_key();
    let agent_addr = ethereum_address(&secret);
    let mut actor = ReputationActor::constructor(Address::new_id(1000), agent_addr);

    let params = signed_params(&secret, [3u8; 20], 66);
    let unauthorised_caller = [9u8; 20];
    let err = must_err(actor.set_score(unauthorised_caller, params, 10, 20));
    assert_eq!(err.code, EXIT_FORBIDDEN);
}

#[test]
fn test_update_score() {
    let secret = secret_key();
    let agent_addr = ethereum_address(&secret);
    let mut actor = ReputationActor::constructor(Address::new_id(1000), agent_addr);
    let developer = [4u8; 20];

    let params1 = signed_params(&secret, developer, 40);
    actor
        .set_score(agent_addr, params1, 11, 100)
        .expect("first score should set");

    let params2 = signed_params(&secret, developer, 92);
    actor
        .set_score(agent_addr, params2, 12, 200)
        .expect("second score should overwrite");

    let record = actor.get_score(developer).expect("record should exist");
    assert_eq!(record.score, 92);
    assert_eq!(record.block_height, 12);
}
