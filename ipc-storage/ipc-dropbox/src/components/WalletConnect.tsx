import React from 'react';

interface WalletConnectProps {
  address: string | null;
  isConnecting: boolean;
  error: string | null;
  onConnect: () => void;
  onDisconnect: () => void;
}

export function WalletConnect({
  address,
  isConnecting,
  error,
  onConnect,
  onDisconnect,
}: WalletConnectProps) {
  const shortenAddress = (addr: string) =>
    `${addr.slice(0, 6)}...${addr.slice(-4)}`;

  return (
    <div className="wallet-connect">
      {address ? (
        <div className="wallet-info">
          <span className="wallet-address">{shortenAddress(address)}</span>
          <button onClick={onDisconnect} className="btn btn-secondary">
            Disconnect
          </button>
        </div>
      ) : (
        <button
          onClick={onConnect}
          disabled={isConnecting}
          className="btn btn-primary"
        >
          {isConnecting ? 'Connecting...' : 'Connect Wallet'}
        </button>
      )}
      {error && <p className="error">{error}</p>}
    </div>
  );
}
