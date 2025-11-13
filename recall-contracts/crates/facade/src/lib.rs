// Copyright 2025 Recall Contributors
// SPDX-License-Identifier: Apache-2.0, MIT

#![allow(dead_code)]

pub use alloy_primitives as primitives;

pub mod types;

#[cfg(feature = "blob-reader")]
mod blobreader_facade;
#[cfg(feature = "blob-reader")]
pub mod blob_reader {
    pub type Events = crate::blobreader_facade::iblobreaderfacade::IBlobReaderFacade::IBlobReaderFacadeEvents;
    pub type ReadRequestClosed = crate::blobreader_facade::iblobreaderfacade::IBlobReaderFacade::ReadRequestClosed;
    pub type ReadRequestOpened = crate::blobreader_facade::iblobreaderfacade::IBlobReaderFacade::ReadRequestOpened;
    pub type ReadRequestPending = crate::blobreader_facade::iblobreaderfacade::IBlobReaderFacade::ReadRequestPending;
}

#[cfg(feature = "blobs")]
mod blobs_facade;
#[cfg(feature = "blobs")]
pub mod blobs {
    pub type Events = crate::blobs_facade::iblobsfacade::IBlobsFacade::IBlobsFacadeEvents;
    pub type BlobAdded = crate::blobs_facade::iblobsfacade::IBlobsFacade::BlobAdded;
    pub type BlobDeleted = crate::blobs_facade::iblobsfacade::IBlobsFacade::BlobDeleted;
    pub type BlobFinalized = crate::blobs_facade::iblobsfacade::IBlobsFacade::BlobFinalized;
    pub type BlobPending = crate::blobs_facade::iblobsfacade::IBlobsFacade::BlobPending;

    pub type Calls = crate::blobs_facade::iblobsfacade::IBlobsFacade::IBlobsFacadeCalls;
    #[allow(non_camel_case_types)]
    pub type addBlobCall = crate::blobs_facade::iblobsfacade::IBlobsFacade::addBlobCall;
    #[allow(non_camel_case_types)]
    pub type deleteBlobCall = crate::blobs_facade::iblobsfacade::IBlobsFacade::deleteBlobCall;
    #[allow(non_camel_case_types)]
    pub type getBlobCall = crate::blobs_facade::iblobsfacade::IBlobsFacade::getBlobCall;
    #[allow(non_camel_case_types)]
    pub type getStatsCall = crate::blobs_facade::iblobsfacade::IBlobsFacade::getStatsCall;
    #[allow(non_camel_case_types)]
    pub type overwriteBlobCall = crate::blobs_facade::iblobsfacade::IBlobsFacade::overwriteBlobCall;
    #[allow(non_camel_case_types)]
    pub type trimBlobExpiriesCall = crate::blobs_facade::iblobsfacade::IBlobsFacade::trimBlobExpiriesCall;

    pub type Subscription = crate::blobs_facade::iblobsfacade::IBlobsFacade::Subscription;
    pub type Blob = crate::blobs_facade::iblobsfacade::IBlobsFacade::Blob;
    pub type SubnetStats = crate::blobs_facade::iblobsfacade::IBlobsFacade::SubnetStats;
    pub type TrimBlobExpiries = crate::blobs_facade::iblobsfacade::IBlobsFacade::TrimBlobExpiries;
}

#[cfg(feature = "bucket")]
mod bucket_facade;
#[cfg(feature = "bucket")]
pub mod bucket {
    pub type Events = crate::bucket_facade::ibucketfacade::IBucketFacade::IBucketFacadeEvents;
    pub type ObjectAdded = crate::bucket_facade::ibucketfacade::IBucketFacade::ObjectAdded;
    pub type ObjectDeleted = crate::bucket_facade::ibucketfacade::IBucketFacade::ObjectDeleted;
    pub type ObjectMetadataUpdated = crate::bucket_facade::ibucketfacade::IBucketFacade::ObjectMetadataUpdated;

    pub type Calls = crate::bucket_facade::ibucketfacade::IBucketFacade::IBucketFacadeCalls;
    #[allow(non_camel_case_types)]
    pub type addObject_0Call = crate::bucket_facade::ibucketfacade::IBucketFacade::addObject_0Call;
    #[allow(non_camel_case_types)]
    pub type addObject_1Call = crate::bucket_facade::ibucketfacade::IBucketFacade::addObject_1Call;
    #[allow(non_camel_case_types)]
    pub type deleteObjectCall = crate::bucket_facade::ibucketfacade::IBucketFacade::deleteObjectCall;
    #[allow(non_camel_case_types)]
    pub type getObjectCall = crate::bucket_facade::ibucketfacade::IBucketFacade::getObjectCall;
    #[allow(non_camel_case_types)]
    pub type queryObjects_0Call = crate::bucket_facade::ibucketfacade::IBucketFacade::queryObjects_0Call;
    #[allow(non_camel_case_types)]
    pub type queryObjects_1Call = crate::bucket_facade::ibucketfacade::IBucketFacade::queryObjects_1Call;
    #[allow(non_camel_case_types)]
    pub type queryObjects_2Call = crate::bucket_facade::ibucketfacade::IBucketFacade::queryObjects_2Call;
    #[allow(non_camel_case_types)]
    pub type queryObjects_3Call = crate::bucket_facade::ibucketfacade::IBucketFacade::queryObjects_3Call;
    #[allow(non_camel_case_types)]
    pub type queryObjects_4Call = crate::bucket_facade::ibucketfacade::IBucketFacade::queryObjects_4Call;
    #[allow(non_camel_case_types)]
    pub type updateObjectMetadataCall = crate::bucket_facade::ibucketfacade::IBucketFacade::updateObjectMetadataCall;

    pub type ObjectValue = crate::bucket_facade::ibucketfacade::IBucketFacade::ObjectValue;
    pub type KeyValue = crate::bucket_facade::ibucketfacade::IBucketFacade::KeyValue;
    pub type Query = crate::bucket_facade::ibucketfacade::IBucketFacade::Query;
    pub type Object = crate::bucket_facade::ibucketfacade::IBucketFacade::Object;
    pub type ObjectState = crate::bucket_facade::ibucketfacade::IBucketFacade::ObjectState;
}

#[cfg(feature = "config")]
mod config_facade;
#[cfg(feature = "config")]
pub mod config {
    pub type Events = crate::config_facade::iconfigfacade::IConfigFacade::IConfigFacadeEvents;
    pub type ConfigAdminSet = crate::config_facade::iconfigfacade::IConfigFacade::ConfigAdminSet;
    pub type ConfigSet = crate::config_facade::iconfigfacade::IConfigFacade::ConfigSet;
}

#[cfg(feature = "credit")]
mod credit_facade;
#[cfg(feature = "credit")]
pub mod credit {
    pub type Events = crate::credit_facade::icreditfacade::ICreditFacade::ICreditFacadeEvents;
    pub type CreditApproved = crate::credit_facade::icreditfacade::ICreditFacade::CreditApproved;
    pub type CreditDebited = crate::credit_facade::icreditfacade::ICreditFacade::CreditDebited;
    pub type CreditPurchased = crate::credit_facade::icreditfacade::ICreditFacade::CreditPurchased;
    pub type CreditRevoked = crate::credit_facade::icreditfacade::ICreditFacade::CreditRevoked;

    pub type Calls = crate::credit_facade::icreditfacade::ICreditFacade::ICreditFacadeCalls;
    #[allow(non_camel_case_types)]
    pub type buyCredit_0Call = crate::credit_facade::icreditfacade::ICreditFacade::buyCredit_0Call;
    #[allow(non_camel_case_types)]
    pub type buyCredit_1Call = crate::credit_facade::icreditfacade::ICreditFacade::buyCredit_1Call;
    #[allow(non_camel_case_types)]
    pub type approveCredit_0Call = crate::credit_facade::icreditfacade::ICreditFacade::approveCredit_0Call;
    #[allow(non_camel_case_types)]
    pub type approveCredit_1Call = crate::credit_facade::icreditfacade::ICreditFacade::approveCredit_1Call;
    #[allow(non_camel_case_types)]
    pub type approveCredit_2Call = crate::credit_facade::icreditfacade::ICreditFacade::approveCredit_2Call;
    #[allow(non_camel_case_types)]
    pub type revokeCredit_0Call = crate::credit_facade::icreditfacade::ICreditFacade::revokeCredit_0Call;
    #[allow(non_camel_case_types)]
    pub type revokeCredit_1Call = crate::credit_facade::icreditfacade::ICreditFacade::revokeCredit_1Call;
    #[allow(non_camel_case_types)]
    pub type setAccountSponsorCall = crate::credit_facade::icreditfacade::ICreditFacade::setAccountSponsorCall;
    #[allow(non_camel_case_types)]
    pub type getAccountCall = crate::credit_facade::icreditfacade::ICreditFacade::getAccountCall;
    #[allow(non_camel_case_types)]
    pub type getCreditApprovalCall = crate::credit_facade::icreditfacade::ICreditFacade::getCreditApprovalCall;
    #[allow(non_camel_case_types)]
    pub type setAccountStatusCall = crate::credit_facade::icreditfacade::ICreditFacade::setAccountStatusCall;

    pub type Account = crate::credit_facade::icreditfacade::ICreditFacade::Account;
    pub type Approval = crate::credit_facade::icreditfacade::ICreditFacade::Approval;
    pub type CreditApproval = crate::credit_facade::icreditfacade::ICreditFacade::CreditApproval;
    pub type TtlStatus = crate::credit_facade::icreditfacade::ICreditFacade::TtlStatus;
}

#[cfg(feature = "gas")]
mod gas_facade;
#[cfg(feature = "gas")]
pub mod gas {
    pub type Events = crate::gas_facade::igasfacade::IGasFacade::IGasFacadeEvents;
    pub type GasSponsorSet = crate::gas_facade::igasfacade::IGasFacade::GasSponsorSet;
    pub type GasSponsorUnset = crate::gas_facade::igasfacade::IGasFacade::GasSponsorUnset;
}

#[cfg(feature = "machine")]
mod machine_facade;
#[cfg(feature = "machine")]
pub mod machine {
    pub type Events = crate::machine_facade::imachinefacade::IMachineFacade::IMachineFacadeEvents;
    pub type MachineCreated = crate::machine_facade::imachinefacade::IMachineFacade::MachineCreated;
    pub type MachineInitialized = crate::machine_facade::imachinefacade::IMachineFacade::MachineInitialized;

    pub type Calls = crate::machine_facade::imachinefacade::IMachineFacade::IMachineFacadeCalls;
    #[allow(non_camel_case_types)]
    pub type createBucket_0Call = crate::machine_facade::imachinefacade::IMachineFacade::createBucket_0Call;
    #[allow(non_camel_case_types)]
    pub type createBucket_1Call = crate::machine_facade::imachinefacade::IMachineFacade::createBucket_1Call;
    #[allow(non_camel_case_types)]
    pub type createBucket_2Call = crate::machine_facade::imachinefacade::IMachineFacade::createBucket_2Call;
    #[allow(non_camel_case_types)]
    pub type listBuckets_0Call = crate::machine_facade::imachinefacade::IMachineFacade::listBuckets_0Call;
    #[allow(non_camel_case_types)]
    pub type listBuckets_1Call = crate::machine_facade::imachinefacade::IMachineFacade::listBuckets_1Call;

    pub type Machine = crate::machine_facade::imachinefacade::IMachineFacade::Machine;
    pub type Kind = crate::machine_facade::imachinefacade::IMachineFacade::Kind;
    pub type KeyValue = crate::machine_facade::imachinefacade::IMachineFacade::KeyValue;
}

#[cfg(feature = "timehub")]
mod timehub_facade;
#[cfg(feature = "timehub")]
pub mod timehub {
    pub type Events = crate::timehub_facade::itimehubfacade::ITimehubFacade::ITimehubFacadeEvents;
    pub type EventPushed = crate::timehub_facade::itimehubfacade::ITimehubFacade::EventPushed;

    pub type Calls = crate::timehub_facade::itimehubfacade::ITimehubFacade::ITimehubFacadeCalls;
    #[allow(non_camel_case_types)]
    pub type pushCall = crate::timehub_facade::itimehubfacade::ITimehubFacade::pushCall;
    #[allow(non_camel_case_types)]
    pub type getLeafAtCall = crate::timehub_facade::itimehubfacade::ITimehubFacade::getLeafAtCall;
    #[allow(non_camel_case_types)]
    pub type getRootCall = crate::timehub_facade::itimehubfacade::ITimehubFacade::getRootCall;
    #[allow(non_camel_case_types)]
    pub type getPeaksCall = crate::timehub_facade::itimehubfacade::ITimehubFacade::getPeaksCall;
    #[allow(non_camel_case_types)]
    pub type getCountCall = crate::timehub_facade::itimehubfacade::ITimehubFacade::getCountCall;
}
