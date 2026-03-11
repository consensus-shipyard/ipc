import { Config } from '../types';

export function getConfig(): Config {
  return {
    tendermintRpc: import.meta.env.VITE_TENDERMINT_RPC || 'http://localhost:26657',
    objectsListenAddr: import.meta.env.VITE_OBJECTS_LISTEN_ADDR || 'http://localhost:8080',
    nodeOperationObjectApi: import.meta.env.VITE_NODE_OPERATION_OBJECT_API || 'http://localhost:8081',
    ethRpc: import.meta.env.VITE_ETH_RPC || 'http://localhost:8545',
    blobsActor: import.meta.env.VITE_BLOBS_ACTOR || '0x6d342defae60f6402aee1f804653bbae4e66ae46',
    admActor: import.meta.env.VITE_ADM_ACTOR || '0x7caec36fc8a3a867ca5b80c6acb5e5871d05aa28',
    chainId: parseInt(import.meta.env.VITE_CHAIN_ID || '1023102'),
  };
}
