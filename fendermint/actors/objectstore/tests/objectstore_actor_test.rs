// Copyright 2024 Textile Inc
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::vec;

use fendermint_actor_objectstore::{
    actor::Actor as ObjectstoreActor, Method, PutObjectParams, State,
};
use fil_actors_runtime::{
    test_utils::{expect_abort, MockRuntime, ACCOUNT_ACTOR_CODE_ID, SYSTEM_ACTOR_CODE_ID},
    SYSTEM_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::{address::Address, error::ExitCode, MethodNum};

#[test]
fn construction() {
    fn construct(exit_code: ExitCode) {
        let rt = MockRuntime {
            receiver: Address::new_id(100),
            ..Default::default()
        };
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        if exit_code.is_success() {
            rt.call::<ObjectstoreActor>(Method::Constructor as MethodNum, None)
                .unwrap();

            let state: State = rt.get_state();
            assert_eq!(state.root, Default::default());
            rt.expect_validate_caller_any();
        } else {
            expect_abort(
                exit_code,
                rt.call::<ObjectstoreActor>(1, IpldBlock::serialize_cbor(&false).unwrap()),
            )
        }
        rt.verify();
    }

    construct(ExitCode::OK);
    construct(ExitCode::USR_ILLEGAL_STATE);
}

#[test]
fn put_object() {
    let rt = MockRuntime {
        receiver: Address::new_id(100),
        ..Default::default()
    };
    rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
    rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

    rt.call::<ObjectstoreActor>(Method::Constructor as MethodNum, None)
        .unwrap();

    let state: State = rt.get_state();
    assert_eq!(state.root, Default::default());

    rt.set_caller(*ACCOUNT_ACTOR_CODE_ID, Address::new_id(1234));
    rt.expect_validate_caller_any();

    let ret = rt
        .call::<ObjectstoreActor>(
            Method::PutObject as MethodNum,
            IpldBlock::serialize_cbor(&PutObjectParams {
                key: vec![1, 2, 3],
                content: vec![1, 2, 3],
            })
            .unwrap(),
        )
        .unwrap();
    assert!(ret.is_none());

    assert_eq!(state.root, Default::default());
}
