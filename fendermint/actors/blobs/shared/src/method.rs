// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,

    // EVM Interop
    InvokeContract = frc42_dispatch::method_hash!("InvokeEVM"),

    // User methods
    BuyCredit = frc42_dispatch::method_hash!("BuyCredit"),
    ApproveCredit = frc42_dispatch::method_hash!("ApproveCredit"),
    RevokeCredit = frc42_dispatch::method_hash!("RevokeCredit"),
    SetAccountSponsor = frc42_dispatch::method_hash!("SetAccountSponsor"),
    GetAccount = frc42_dispatch::method_hash!("GetAccount"),
    GetCreditApproval = frc42_dispatch::method_hash!("GetCreditApproval"),
    AddBlob = frc42_dispatch::method_hash!("AddBlob"),
    GetBlob = frc42_dispatch::method_hash!("GetBlob"),
    DeleteBlob = frc42_dispatch::method_hash!("DeleteBlob"),
    OverwriteBlob = frc42_dispatch::method_hash!("OverwriteBlob"),

    // System methods
    GetGasAllowance = frc42_dispatch::method_hash!("GetGasAllowance"),
    UpdateGasAllowance = frc42_dispatch::method_hash!("UpdateGasAllowance"),
    GetBlobStatus = frc42_dispatch::method_hash!("GetBlobStatus"),
    GetAddedBlobs = frc42_dispatch::method_hash!("GetAddedBlobs"),
    GetPendingBlobs = frc42_dispatch::method_hash!("GetPendingBlobs"),
    SetBlobPending = frc42_dispatch::method_hash!("SetBlobPending"),
    FinalizeBlob = frc42_dispatch::method_hash!("FinalizeBlob"),
    DebitAccounts = frc42_dispatch::method_hash!("DebitAccounts"),

    // Admin methods
    SetAccountStatus = frc42_dispatch::method_hash!("SetAccountStatus"),
    TrimBlobExpiries = frc42_dispatch::method_hash!("TrimBlobExpiries"),

    // Metrics methods
    GetStats = frc42_dispatch::method_hash!("GetStats"),
}
