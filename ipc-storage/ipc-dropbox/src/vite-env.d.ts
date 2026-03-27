/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_TENDERMINT_RPC: string;
  readonly VITE_OBJECTS_LISTEN_ADDR: string;
  readonly VITE_NODE_OPERATION_OBJECT_API: string;
  readonly VITE_ETH_RPC: string;
  readonly VITE_BLOBS_ACTOR: string;
  readonly VITE_ADM_ACTOR: string;
  readonly VITE_CHAIN_ID: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
