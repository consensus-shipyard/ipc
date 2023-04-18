// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Json serialization of checkpoints

use crate::serialization::SerializeToJson;
use base64::Engine;
use ipc_gateway::checkpoint::{BatchCrossMsgs, CheckData};
use ipc_gateway::BottomUpCheckpoint;
use num_traits::ToPrimitive;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

impl Serialize for SerializeToJson<BottomUpCheckpoint> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let BottomUpCheckpoint { data, sig } = &self.0;

        let data = SerializeToJson(data);
        let sig = base64::engine::general_purpose::STANDARD.encode(sig);

        let mut state = serializer.serialize_struct("BottomUpCheckpoint", 2)?;
        state.serialize_field("data", &data)?;
        state.serialize_field("sig", &sig)?;
        state.end()
    }
}

impl<'a> Serialize for SerializeToJson<&'a CheckData> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let CheckData {
            source,
            proof,
            epoch,
            prev_check,
            children,
            cross_msgs,
        } = self.0;

        let source = source.to_string();
        let proof = base64::engine::general_purpose::STANDARD.encode(proof);
        let prev_check = prev_check.to_string();
        let children = children
            .iter()
            .map(|c| {
                let source = c.source.to_string();
                let checks = c
                    .checks
                    .iter()
                    .map(|cid| cid.to_string())
                    .collect::<Vec<_>>();
                serde_json::json!({
                    "source": source,
                    "checks": checks,
                })
            })
            .collect::<Vec<_>>();
        let cross_msgs = SerializeToJson(cross_msgs);

        let mut state = serializer.serialize_struct("CheckData", 6)?;
        state.serialize_field("source", &source)?;
        state.serialize_field("proof", &proof)?;
        state.serialize_field("epoch", epoch)?;
        state.serialize_field("prev_check", &prev_check)?;
        state.serialize_field("children", &children)?;
        state.serialize_field("cross_msgs", &cross_msgs)?;

        state.end()
    }
}

impl<'a> Serialize for SerializeToJson<&'a BatchCrossMsgs> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let BatchCrossMsgs { cross_msgs, fee } = self.0;

        let mut state = serializer.serialize_struct("BatchCrossMsgs", 2)?;
        state.serialize_field("fee", fee)?;

        if let Some(cross_msgs) = cross_msgs {
            let vs = cross_msgs.iter().map(|c| {
                serde_json::json!({
                    "from": c.msg.from.to_string().unwrap(), // safe to unwrap
                    "to": c.msg.to.to_string().unwrap(), // safe to unwrap
                    "method": c.msg.method,
                    "params": base64::engine::general_purpose::STANDARD.encode(c.msg.params.bytes()),
                    "value": c.msg.value.atto().to_u64().unwrap_or_default(),
                    "nonce": c.msg.nonce,
                })
            })
                .collect::<Vec<_>>();
            state.serialize_field("cross_msgs", &vs)?;
        } else {
            state.serialize_field::<Vec<serde_json::Value>>("cross_msgs", &vec![])?;
        };

        state.end()
    }
}

#[cfg(test)]
mod tests {
    use crate::serialization::SerializeToJson;
    use ipc_gateway::BottomUpCheckpoint;
    use ipc_sdk::subnet_id::ROOTNET_ID;

    #[test]
    fn test_serialization() {
        let cp = BottomUpCheckpoint::new(ROOTNET_ID.clone(), 10);
        let v = serde_json::to_string(&SerializeToJson(cp)).unwrap();
        println!("{v:}");
    }
}
