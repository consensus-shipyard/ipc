import { useState, useCallback } from 'react';
import { ethers } from 'ethers';
import { getConfig } from '../utils/config';
import { getBucketContract, getBlobsContract, BlobStatus } from '../utils/contracts';
import { base32ToHex } from '../utils/base32';
import { UploadResponse, NodeInfo } from '../types';

export function useUpload(signer: ethers.Signer | null, bucketAddress: string | null) {
  const [isUploading, setIsUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState<string>('');
  const [blobStatus, setBlobStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const pollBlobStatus = useCallback(async (blobHash: string, maxAttempts: number = 60) => {
    const config = getConfig();
    const provider = signer?.provider;
    if (!provider) return;

    const blobsContract = getBlobsContract(config.blobsActor, provider);

    for (let i = 0; i < maxAttempts; i++) {
      try {
        const blob = await blobsContract.getBlob(blobHash);
        const status = Number(blob.status ?? blob[3]);

        if (status === BlobStatus.Resolved) {
          setBlobStatus('Resolved');
          setUploadProgress('Upload complete! Blob resolved.');
          return true;
        } else if (status === BlobStatus.Failed) {
          setBlobStatus('Failed');
          setUploadProgress('Blob resolution failed.');
          return false;
        } else {
          setBlobStatus('Pending');
          setUploadProgress(`Waiting for resolution... (${i + 1}/${maxAttempts})`);
        }
      } catch (err) {
        console.log('Blob not yet registered, waiting...', err);
        setUploadProgress(`Waiting for blob registration... (${i + 1}/${maxAttempts})`);
      }

      // Wait 2 seconds before next poll
      await new Promise(resolve => setTimeout(resolve, 2000));
    }

    setUploadProgress('Timeout waiting for blob resolution');
    return false;
  }, [signer]);

  const uploadFile = useCallback(async (file: File, targetPath: string) => {
    if (!signer || !bucketAddress) {
      setError('Wallet or bucket not connected');
      return false;
    }

    setIsUploading(true);
    setUploadProgress('Preparing upload...');
    setBlobStatus(null);
    setError(null);

    try {
      const config = getConfig();

      // Step 1: Upload to gateway
      setUploadProgress('Uploading to gateway...');
      const formData = new FormData();
      formData.append('size', file.size.toString());
      formData.append('data', file);

      const uploadResponse = await fetch(`${config.objectsListenAddr}/v1/objects`, {
        method: 'POST',
        body: formData,
      });

      if (!uploadResponse.ok) {
        throw new Error(`Upload failed: ${uploadResponse.statusText}`);
      }

      const uploadResult: UploadResponse = await uploadResponse.json();
      console.log('Upload result:', uploadResult);

      // Get node info
      const nodeResponse = await fetch(`${config.objectsListenAddr}/v1/node`);
      const nodeInfo: NodeInfo = await nodeResponse.json();

      // Convert base32 hashes to hex
      const blobHash = base32ToHex(uploadResult.hash);
      const metadataHash = base32ToHex(uploadResult.metadata_hash || uploadResult.metadataHash || '');
      const sourceNode = '0x' + nodeInfo.node_id;

      console.log('Blob hash (hex):', blobHash);
      console.log('Metadata hash (hex):', metadataHash);
      console.log('Source node:', sourceNode);

      // Step 2: Register in bucket
      setUploadProgress('Registering in bucket...');
      const contract = getBucketContract(bucketAddress, signer);

      // Build the full path
      let fullPath = targetPath;
      if (!fullPath.endsWith('/') && fullPath !== '') {
        fullPath += '/';
      }
      fullPath += file.name;

      const tx = await contract.addObject(
        sourceNode,
        fullPath,
        blobHash,
        metadataHash,
        BigInt(file.size)
      );

      setUploadProgress('Waiting for transaction confirmation...');
      await tx.wait();

      // Step 3: Poll for blob status
      setUploadProgress('Checking blob status...');
      await pollBlobStatus(blobHash);

      return true;
    } catch (err: unknown) {
      const error = err as Error;
      console.error('Upload error:', err);
      setError(error.message || 'Upload failed');
      return false;
    } finally {
      setIsUploading(false);
    }
  }, [signer, bucketAddress, pollBlobStatus]);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    isUploading,
    uploadProgress,
    blobStatus,
    error,
    uploadFile,
    clearError,
  };
}
