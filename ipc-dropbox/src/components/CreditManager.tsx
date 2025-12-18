import React, { useEffect, useState } from 'react';
import { ethers } from 'ethers';
import { CreditInfo } from '../types';

interface CreditManagerProps {
  credit: CreditInfo | null;
  hasCredit: boolean;
  isLoading: boolean;
  isPurchasing: boolean;
  error: string | null;
  onFetchCredit: () => void;
  onBuyCredit: (amount: string) => Promise<boolean>;
}

export function CreditManager({
  credit,
  hasCredit,
  isLoading,
  isPurchasing,
  error,
  onFetchCredit,
  onBuyCredit,
}: CreditManagerProps) {
  const [amount, setAmount] = useState('0.1');

  useEffect(() => {
    onFetchCredit();
  }, [onFetchCredit]);

  const formatCredit = (value: bigint) => {
    return ethers.formatEther(value);
  };

  const handleBuyCredit = async () => {
    await onBuyCredit(amount);
  };

  if (isLoading) {
    return <div className="credit-manager loading">Loading credit info...</div>;
  }

  return (
    <div className="credit-manager">
      <h3>Credit Balance</h3>
      {credit && (
        <div className="credit-info">
          <p>
            <strong>Current Credit:</strong> {formatCredit(credit.balance)} FIL
          </p>
          <p>
            <strong>Free Credit:</strong> {formatCredit(credit.freeCredit)} FIL
          </p>
        </div>
      )}

      {!hasCredit && (
        <div className="buy-credit">
          <p className="warning">You need credit to use IPC storage.</p>
          <div className="buy-form">
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              step="0.1"
              min="0.01"
              className="input"
            />
            <span className="unit">FIL</span>
            <button
              onClick={handleBuyCredit}
              disabled={isPurchasing}
              className="btn btn-primary"
            >
              {isPurchasing ? 'Purchasing...' : 'Buy Credit'}
            </button>
          </div>
        </div>
      )}

      {error && <p className="error">{error}</p>}
    </div>
  );
}
