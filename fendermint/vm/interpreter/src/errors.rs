// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Error;
use fendermint_vm_message::signed::SignedMessageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CheckMessageError {
    #[error("illegal message: {0}")]
    IllegalMessage(String),
    #[error("invalid message: {0}")]
    InvalidMessage(String),
    #[error("invalid signature")]
    InvalidSignature(#[from] SignedMessageError),
    #[error("other error: {0}")]
    Other(#[from] Error),
}

#[derive(Error, Debug)]
pub enum ApplyMessageError {
    #[error("invalid message: {0}")]
    InvalidMessage(String),
    #[error("invalid signature")]
    InvalidSignature(#[from] SignedMessageError),
    #[error("other error: {0}")]
    Other(#[from] Error),
}

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("invalid query: {0}")]
    InvalidQuery(String),
    #[error("other error: {0}")]
    Other(#[from] Error),
}

macro_rules! anyhow_wrapper_error {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Error, Debug)]
            pub enum $name {
                #[error("other error: {0}")]
                Other(#[from] Error),
            }
        )*
    }
}

anyhow_wrapper_error!(
    BeginBlockError,
    EndBlockError,
    PrepareMessagesError,
    AttestMessagesError,
);
