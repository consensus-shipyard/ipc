import { useState, useCallback, useEffect } from 'react';
import { ethers } from 'ethers';
import { getConfig } from '../utils/config';

declare global {
  interface Window {
    ethereum?: ethers.Eip1193Provider & {
      on: (event: string, callback: (...args: unknown[]) => void) => void;
      removeListener: (event: string, callback: (...args: unknown[]) => void) => void;
    };
  }
}

export interface WalletState {
  address: string | null;
  signer: ethers.Signer | null;
  provider: ethers.BrowserProvider | null;
  isConnecting: boolean;
  error: string | null;
}

export function useWallet() {
  const [state, setState] = useState<WalletState>({
    address: null,
    signer: null,
    provider: null,
    isConnecting: false,
    error: null,
  });

  const connect = useCallback(async () => {
    if (!window.ethereum) {
      setState(s => ({ ...s, error: 'MetaMask not found. Please install MetaMask.' }));
      return;
    }

    setState(s => ({ ...s, isConnecting: true, error: null }));

    try {
      const config = getConfig();
      const provider = new ethers.BrowserProvider(window.ethereum);

      // Request accounts
      await provider.send('eth_requestAccounts', []);

      // Try to switch to the correct network
      try {
        const chainId = await provider.send('eth_chainId', []);
        const targetChainId = '0x' + BigInt(config.chainId).toString(16);

        if (chainId !== targetChainId) {
          try {
            await provider.send('wallet_switchEthereumChain', [{ chainId: targetChainId }]);
          } catch (switchError: unknown) {
            const err = switchError as { code?: number };
            // Chain not added, try to add it
            if (err.code === 4902) {
              await provider.send('wallet_addEthereumChain', [{
                chainId: targetChainId,
                chainName: 'IPC Local',
                rpcUrls: [config.ethRpc],
                nativeCurrency: {
                  name: 'FIL',
                  symbol: 'FIL',
                  decimals: 18,
                },
              }]);
            }
          }
        }
      } catch {
        // Ignore network switch errors
      }

      const signer = await provider.getSigner();
      const address = await signer.getAddress();

      setState({
        address,
        signer,
        provider,
        isConnecting: false,
        error: null,
      });
    } catch (err: unknown) {
      const error = err as Error;
      setState(s => ({
        ...s,
        isConnecting: false,
        error: error.message || 'Failed to connect wallet',
      }));
    }
  }, []);

  const disconnect = useCallback(() => {
    setState({
      address: null,
      signer: null,
      provider: null,
      isConnecting: false,
      error: null,
    });
  }, []);

  // Listen for account changes
  useEffect(() => {
    if (!window.ethereum) return;

    const handleAccountsChanged = (accounts: unknown) => {
      const accs = accounts as string[];
      if (accs.length === 0) {
        disconnect();
      } else if (state.address && accs[0].toLowerCase() !== state.address.toLowerCase()) {
        connect();
      }
    };

    window.ethereum.on('accountsChanged', handleAccountsChanged);
    return () => {
      window.ethereum?.removeListener('accountsChanged', handleAccountsChanged);
    };
  }, [state.address, connect, disconnect]);

  return {
    ...state,
    connect,
    disconnect,
    isConnected: !!state.address,
  };
}
