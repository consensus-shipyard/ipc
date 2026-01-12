import React from 'react';
import { useWallet } from './hooks/useWallet';
import { useCredit } from './hooks/useCredit';
import { useBucket, useFileExplorer } from './hooks/useBucket';
import { useUpload } from './hooks/useUpload';
import { useDownload } from './hooks/useDownload';
import { WalletConnect } from './components/WalletConnect';
import { CreditManager } from './components/CreditManager';
import { BucketManager } from './components/BucketManager';
import { FileExplorer } from './components/FileExplorer';

function App() {
  const wallet = useWallet();
  const credit = useCredit(wallet.signer, wallet.address);
  const bucket = useBucket(wallet.signer, wallet.address);
  const fileExplorer = useFileExplorer(wallet.signer, bucket.bucketAddress);
  const upload = useUpload(wallet.signer, bucket.bucketAddress);
  const download = useDownload();

  return (
    <div className="app">
      <header className="header">
        <h1>IPC Decentralized Dropbox</h1>
        <WalletConnect
          address={wallet.address}
          isConnecting={wallet.isConnecting}
          error={wallet.error}
          onConnect={wallet.connect}
          onDisconnect={wallet.disconnect}
        />
      </header>

      <main className="main">
        {!wallet.isConnected ? (
          <div className="welcome">
            <h2>Welcome to IPC Decentralized Dropbox</h2>
            <p>Connect your wallet to start storing files on the IPC network.</p>
            <button onClick={wallet.connect} className="btn btn-primary btn-large">
              Connect MetaMask
            </button>
          </div>
        ) : !credit.hasCredit ? (
          <div className="setup-step">
            <h2>Step 1: Get Storage Credit</h2>
            <CreditManager
              credit={credit.credit}
              hasCredit={credit.hasCredit}
              isLoading={credit.isLoading}
              isPurchasing={credit.isPurchasing}
              error={credit.error}
              onFetchCredit={credit.fetchCredit}
              onBuyCredit={credit.buyCredit}
            />
          </div>
        ) : !bucket.hasBucket ? (
          <div className="setup-step">
            <h2>Step 2: Create a Storage Bucket</h2>
            <div className="credit-summary">
              <CreditManager
                credit={credit.credit}
                hasCredit={credit.hasCredit}
                isLoading={credit.isLoading}
                isPurchasing={credit.isPurchasing}
                error={credit.error}
                onFetchCredit={credit.fetchCredit}
                onBuyCredit={credit.buyCredit}
              />
            </div>
            <BucketManager
              bucketAddress={bucket.bucketAddress}
              hasBucket={bucket.hasBucket}
              isLoading={bucket.isLoading}
              isCreating={bucket.isCreating}
              error={bucket.error}
              onFetchBuckets={bucket.fetchBuckets}
              onCreateBucket={bucket.createBucket}
            />
          </div>
        ) : (
          <div className="dashboard">
            <div className="sidebar">
              <CreditManager
                credit={credit.credit}
                hasCredit={credit.hasCredit}
                isLoading={credit.isLoading}
                isPurchasing={credit.isPurchasing}
                error={credit.error}
                onFetchCredit={credit.fetchCredit}
                onBuyCredit={credit.buyCredit}
              />
              <BucketManager
                bucketAddress={bucket.bucketAddress}
                hasBucket={bucket.hasBucket}
                isLoading={bucket.isLoading}
                isCreating={bucket.isCreating}
                error={bucket.error}
                onFetchBuckets={bucket.fetchBuckets}
                onCreateBucket={bucket.createBucket}
              />
            </div>
            <div className="content">
              <FileExplorer
                files={fileExplorer.files}
                currentPath={fileExplorer.currentPath}
                isLoading={fileExplorer.isLoading}
                isUploading={upload.isUploading}
                isDeleting={fileExplorer.isDeleting}
                uploadProgress={upload.uploadProgress}
                error={fileExplorer.error}
                uploadError={upload.error}
                deleteError={fileExplorer.deleteError}
                onNavigateToFolder={fileExplorer.navigateToFolder}
                onNavigateUp={fileExplorer.navigateUp}
                onRefresh={fileExplorer.refresh}
                onUpload={upload.uploadFile}
                onDownload={download.downloadFile}
                onDelete={fileExplorer.deleteObject}
                onFetchFiles={fileExplorer.fetchFiles}
              />
            </div>
          </div>
        )}
      </main>

      <footer className="footer">
        <p>Powered by IPC Network</p>
      </footer>
    </div>
  );
}

export default App;
