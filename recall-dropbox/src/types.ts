export interface Config {
  tendermintRpc: string;
  objectsListenAddr: string;
  nodeOperationObjectApi: string;
  ethRpc: string;
  blobsActor: string;
  admActor: string;
  chainId: number;
}

export interface ObjectMetadata {
  key: string;
  value: string;
}

export interface ObjectState {
  blobHash: string;
  size: bigint;
  expiry: bigint;
  metadata: ObjectMetadata[];
}

export interface ObjectEntry {
  key: string;
  state: ObjectState;
}

export interface QueryResult {
  objects: ObjectEntry[];
  commonPrefixes: string[];
  nextKey: string;
}

export interface UploadResponse {
  hash: string;
  metadata_hash?: string;
  metadataHash?: string;
}

export interface NodeInfo {
  node_id: string;
}

export interface CreditInfo {
  balance: bigint;
  freeCredit: bigint;
  lastDebitEpoch: bigint;
}

export interface FileItem {
  name: string;
  fullPath: string;
  isFolder: boolean;
  size?: bigint;
  expiry?: bigint;
  blobHash?: string;
}
