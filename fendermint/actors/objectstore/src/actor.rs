// Copyright 2024 Textile Inc
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::actor_dispatch;
use fil_actors_runtime::actor_error;
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::ActorDowncast;
use fil_actors_runtime::ActorError;
use fil_actors_runtime::Map;
use fvm_ipld_hamt::BytesKey;
use fvm_shared::error::ExitCode;

use crate::DeleteObjectParams;
use crate::{Method, PutObjectParams, State, BIT_WIDTH, OBJECTSTORE_ACTOR_NAME};

fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

impl Actor {
    fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        // FIXME: (sander) We're setting this up to be a subnet-wide actor for a single repo.
        // FIXME: (sander) In the future, this could be deployed dynamically for multi repo subnets.
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let state = State::new(rt.store()).map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to create empty Hamt")
        })?;

        rt.create(&state)
    }

    fn append_object(rt: &impl Runtime, params: PutObjectParams) -> Result<(), ActorError> {
        // FIXME: (@carsonfarmer) We'll want to validate the caller is the owner of the repo.
        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, rt| {
            // Load the root Hamt
            let mut hamt = Map::<_, Vec<u8>>::load_with_bit_width(&st.root, rt.store(), BIT_WIDTH)
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to load Hamt root")
                })?;

            let key = BytesKey(params.key);

            let new_content = match hamt.get(&key).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to get object")
            })? {
                Some(existing) => {
                    // Append the object to the existing object
                    let mut new_content = existing.clone();
                    new_content.extend(params.content);
                    new_content
                }
                None => params.content,
            };

            // Put the new content into the Hamt
            hamt.set(key, new_content).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to update key")
            })?;

            // Save the new Hamt cid root
            st.root = hamt.flush().map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to save root")
            })?;

            Ok(())
        })?;

        Ok(())
    }

    fn put_object(rt: &impl Runtime, params: PutObjectParams) -> Result<(), ActorError> {
        // FIXME: (@carsonfarmer) We'll want to validate the caller is the owner of the repo.
        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, rt| {
            // Load the root Hamt
            let mut hamt =
                Map::load_with_bit_width(&st.root, rt.store(), BIT_WIDTH).map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to load Hamt root")
                })?;

            // Put the object into the Hamt
            // TODO: We could use set_if_absent here to avoid overwriting existing objects.
            hamt.set(BytesKey(params.key), params.content)
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to update key")
                })?;

            // Save the new Hamt cid root
            st.root = hamt.flush().map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to save root")
            })?;

            Ok(())
        })?;

        Ok(())
    }

    fn delete_object(rt: &impl Runtime, params: DeleteObjectParams) -> Result<(), ActorError> {
        // FIXME: (@carsonfarmer) We'll want to validate the caller is the owner of the repo.
        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, rt| {
            // Load the root Hamt
            let mut hamt = Map::<_, Vec<u8>>::load_with_bit_width(&st.root, rt.store(), BIT_WIDTH)
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to load Hamt root")
                })?;

            // Delete the object from the Hamt
            hamt.delete(&BytesKey(params.key)).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to delete object")
            })?;

            // Save the new Hamt cid root
            st.root = hamt.flush().map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to save root")
            })?;

            Ok(())
        })?;

        Ok(())
    }

    fn get_object(rt: &impl Runtime, key: Vec<u8>) -> Result<Option<Vec<u8>>, ActorError> {
        let st: State = rt.state()?;

        // Load the root Hamt
        let hamt = Map::<_, Vec<u8>>::load_with_bit_width(&st.root, rt.store(), BIT_WIDTH)
            .map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to load Hamt root")
            })?;

        // Get the object from the Hamt
        hamt.get(&BytesKey(key))
            .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to get object"))
            .map(|v| v.map(|inner| inner.to_owned()))
    }

    fn list_objects(rt: &impl Runtime) -> Result<Option<Vec<Vec<u8>>>, ActorError> {
        let st: State = rt.state()?;

        // Load the root Hamt
        let hamt = Map::<_, Vec<u8>>::load_with_bit_width(&st.root, rt.store(), BIT_WIDTH)
            .map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to load Hamt root")
            })?;

        let mut keys = Vec::new();

        // List the keys from each item in the Hamt
        hamt.for_each(|k, _| {
            keys.push(k.0.to_owned());
            Ok(())
        })
        .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to list objects"))?;

        Ok(Some(keys))
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        OBJECTSTORE_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        PutObject => put_object,
        AppendObject => append_object,
        DeleteObject => delete_object,
        GetObject => get_object,
        ListObjects => list_objects,
    }
}
