import { ethers } from 'ethers';

// ABI for Blobs Actor
export const BLOBS_ABI = [
  'function buyCredit() payable',
  'function getAccount(address addr) view returns (tuple(uint64 capacityUsed, uint256 creditFree, uint256 creditCommitted, address creditSponsor, uint64 lastDebitEpoch, tuple(address addr, tuple(uint256 creditLimit, uint256 gasFeeLimit, uint64 expiry, uint256 creditUsed, uint256 gasFeeUsed) approval)[] approvalsTo, tuple(address addr, tuple(uint256 creditLimit, uint256 gasFeeLimit, uint64 expiry, uint256 creditUsed, uint256 gasFeeUsed) approval)[] approvalsFrom, uint64 maxTtl, uint256 gasAllowance))',
  'function getBlob(bytes32 blobHash) view returns (tuple(uint64 size, bytes32 metadataHash, tuple(string id, int64 expiry)[] subscriptions, uint8 status))',
];

// Blob status enum values
export enum BlobStatus {
  Pending = 0,
  Resolved = 1,
  Failed = 2,
}

// ABI for ADM Actor
export const ADM_ABI = [
  'function createBucket() returns (address)',
  'function listBuckets(address owner) view returns (tuple(uint8 kind, address addr, tuple(string key, string value)[] metadata)[])',
  'event MachineInitialized(uint8 indexed kind, address machineAddress)',
];

// ABI for Bucket Actor
export const BUCKET_ABI = [
  'function addObject(bytes32 source, string key, bytes32 hash, bytes32 recoveryHash, uint64 size, uint16 dataShards, uint16 parityShards)',
  'function getObject(string key) view returns (tuple(bytes32 blobHash, bytes32 recoveryHash, uint64 size, uint64 expiry, tuple(string key, string value)[] metadata))',
  'function deleteObject(string key)',
  'function updateObjectMetadata(string key, tuple(string key, string value)[] metadata)',
  'function queryObjects() view returns (tuple(tuple(string key, tuple(bytes32 blobHash, uint64 size, uint64 expiry, tuple(string key, string value)[] metadata) state)[] objects, string[] commonPrefixes, string nextKey))',
  'function queryObjects(string prefix) view returns (tuple(tuple(string key, tuple(bytes32 blobHash, uint64 size, uint64 expiry, tuple(string key, string value)[] metadata) state)[] objects, string[] commonPrefixes, string nextKey))',
  'function queryObjects(string prefix, string delimiter) view returns (tuple(tuple(string key, tuple(bytes32 blobHash, uint64 size, uint64 expiry, tuple(string key, string value)[] metadata) state)[] objects, string[] commonPrefixes, string nextKey))',
  'function queryObjects(string prefix, string delimiter, string startKey, uint64 limit) view returns (tuple(tuple(string key, tuple(bytes32 blobHash, uint64 size, uint64 expiry, tuple(string key, string value)[] metadata) state)[] objects, string[] commonPrefixes, string nextKey))',
  'function owner() view returns (address)',
];

export function getBlobsContract(address: string, signer: ethers.Signer | ethers.Provider) {
  return new ethers.Contract(address, BLOBS_ABI, signer);
}

export function getAdmContract(address: string, signer: ethers.Signer | ethers.Provider) {
  return new ethers.Contract(address, ADM_ABI, signer);
}

export function getBucketContract(address: string, signer: ethers.Signer | ethers.Provider) {
  return new ethers.Contract(address, BUCKET_ABI, signer);
}

// Event topic for MachineInitialized
export const MACHINE_INITIALIZED_TOPIC = '0x8f7252642373d5f0b89a0c5cd9cd242e5cd5bb1a36aec623756e4f52a8c1ea6e';
