import { useState, useCallback } from 'react';
import { ethers } from 'ethers';
import { getConfig } from '../utils/config';
import { getBlobsContract } from '../utils/contracts';
import { CreditInfo } from '../types';

export function useCredit(signer: ethers.Signer | null, address: string | null) {
  const [credit, setCredit] = useState<CreditInfo | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isPurchasing, setIsPurchasing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchCredit = useCallback(async () => {
    if (!signer || !address) return;

    setIsLoading(true);
    setError(null);

    try {
      const config = getConfig();
      // Use provider for view calls to avoid MetaMask issues
      const provider = await signer.provider;
      if (!provider) throw new Error('No provider available');
      const contract = getBlobsContract(config.blobsActor, provider);
      const account = await contract.getAccount(address);

      console.log('getAccount raw result:', account);

      // Access by property name or index (ethers v6 returns both)
      const creditFree = account.creditFree ?? account[1];
      const creditCommitted = account.creditCommitted ?? account[2];
      const lastDebitEpoch = account.lastDebitEpoch ?? account[4];

      console.log('Parsed credit:', { creditFree, creditCommitted, lastDebitEpoch });

      setCredit({
        balance: creditFree + creditCommitted,
        freeCredit: creditFree,
        lastDebitEpoch: BigInt(lastDebitEpoch),
      });
    } catch (err: unknown) {
      const error = err as Error;
      console.error('fetchCredit error:', err);
      setError(error.message || 'Failed to fetch credit');
    } finally {
      setIsLoading(false);
    }
  }, [signer, address]);

  const buyCredit = useCallback(async (amountEther: string) => {
    if (!signer) {
      setError('Wallet not connected');
      return false;
    }

    setIsPurchasing(true);
    setError(null);

    try {
      const config = getConfig();
      const contract = getBlobsContract(config.blobsActor, signer);
      const tx = await contract.buyCredit({
        value: ethers.parseEther(amountEther),
      });
      await tx.wait();
      await fetchCredit();
      return true;
    } catch (err: unknown) {
      const error = err as Error;
      setError(error.message || 'Failed to buy credit');
      return false;
    } finally {
      setIsPurchasing(false);
    }
  }, [signer, fetchCredit]);

  const hasCredit = credit && (credit.balance > 0n || credit.freeCredit > 0n);

  return {
    credit,
    isLoading,
    isPurchasing,
    error,
    fetchCredit,
    buyCredit,
    hasCredit,
  };
}
