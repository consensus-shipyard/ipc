// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fs;
use crate::keys::*;
use anyhow::{anyhow, Context};
use fendermint_app_options::key::KeyShowPeerIdArgs;
use fendermint_crypto::SecretKey;
use fendermint_vm_actor_interface::eam::EthAddress;
use fvm_shared::address::Address;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use serde_json::json;
use std::path::Path;
use tendermint_config::NodeKey;

use crate::{
    cmd,
    options::key::{
        AddPeer, KeyAddressArgs, KeyArgs, KeyCommands, KeyFromEthArgs, KeyGenArgs, KeyIntoEthArgs,
        KeyIntoTendermintArgs,
    },
};

cmd! {
    KeyArgs(self) {
        match &self.command {
            KeyCommands::Gen(args) => args.exec(()).await,
            KeyCommands::IntoTendermint(args) => args.exec(()).await,
            KeyCommands::AddPeer(args) => args.exec(()).await,
            KeyCommands::Address(args) => args.exec(()).await,
            KeyCommands::FromEth(args) => args.exec(()).await,
            KeyCommands::IntoEth(args) => args.exec(()).await,
            KeyCommands::ShowPeerId(args) => args.exec(()).await,
        }
    }
}

cmd! {
    KeyFromEthArgs(self) {
        let sk = read_secret_key_hex(&self.secret_key)?;
        let pk = sk.public_key();

        export(&self.out_dir, &self.name, "sk", &secret_to_b64(&sk))?;
        export(&self.out_dir, &self.name, "pk", &public_to_b64(&pk))?;

        Ok(())
    }
}

cmd! {
    KeyIntoEthArgs(self) {
        let sk = read_secret_key(&self.secret_key)?;
        let pk = sk.public_key();

        export(&self.out_dir, &self.name, "sk", &hex::encode(sk.serialize()))?;
        export(&self.out_dir, &self.name, "pk", &hex::encode(pk.serialize()))?;
        export(&self.out_dir, &self.name, "addr", &hex::encode(EthAddress::from(pk).0))?;

        Ok(())
    }
}

cmd! {
  KeyGenArgs(self) {
    let mut rng = ChaCha20Rng::from_entropy();
    let sk = SecretKey::random(&mut rng);
    let pk = sk.public_key();

    export(&self.out_dir, &self.name, "sk", &secret_to_b64(&sk))?;
    export(&self.out_dir, &self.name, "pk", &public_to_b64(&pk))?;

    Ok(())
  }
}

cmd! {
  KeyIntoTendermintArgs(self) {
    let sk = read_secret_key(&self.secret_key)?;
    let pk = sk.public_key();
    let vk = tendermint::crypto::default::ecdsa_secp256k1::VerifyingKey::from_sec1_bytes(&pk.serialize())
      .map_err(|e| anyhow!("failed to convert public key: {e}"))?;
    let pub_key = tendermint::PublicKey::Secp256k1(vk);
    let address = tendermint::account::Id::from(pub_key);

    // tendermint-rs doesn't seem to handle Secp256k1 private keys;
    // if it did, we could use tendermint_config::PrivateValidatorKey
    // to encode the data structure. Tendermint should be okay with it
    // though, as long as we match the expected keys in the JSON.
    let priv_validator_key = json! ({
        "address": address,
        "pub_key": pub_key,
        "priv_key": {
            "type": "tendermint/PrivKeySecp256k1",
            "value": secret_to_b64(&sk)
        }
    });
    let json = serde_json::to_string_pretty(&priv_validator_key)?;

    fs::write(&self.out, json)?;

    Ok(())
  }
}

cmd! {
    AddPeer(self) {
        let node_key = NodeKey::load_json_file(&self.node_key_file).context("failed to read node key file")?;
        let peer_id = format!("{}@{}", node_key.node_id(), self.network_addr);
        let mut peers = fs::read_to_string(&self.local_peers_file).unwrap_or_default();

        if peers.is_empty()  {
            peers.push_str(&peer_id);
        } else {
            peers.push(',');
            peers.push_str(peer_id.as_str());
        }

        fs::write(&self.local_peers_file, peers).context("failed to write to the peers file")?;
        Ok(())
  }
}

cmd! {
    KeyAddressArgs(self) {
        let pk = read_public_key(&self.public_key)?;
        let addr = Address::new_secp256k1(&pk.serialize())?;
        println!("{}", addr);
        Ok(())
    }
}

cmd! {
    KeyShowPeerIdArgs(self) {
        let pk = read_public_key(&self.public_key)?;
        // Just using this type because it does the conversion we need.
        let vk = ipc_ipld_resolver::ValidatorKey::from(pk);
        let pk: libp2p::identity::PublicKey = vk.into();
        let id = pk.to_peer_id();
        println!("{}", id);
        Ok(())
    }
}

fn export(output_dir: &Path, name: &str, ext: &str, b64: &str) -> anyhow::Result<()> {
    let output_path = output_dir.join(format!("{name}.{ext}"));
    fs::write(output_path, b64)?;
    Ok(())
}
