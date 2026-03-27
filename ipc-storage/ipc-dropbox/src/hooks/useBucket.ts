import { useState, useCallback } from 'react';
import { ethers } from 'ethers';
import { getConfig } from '../utils/config';
import { getAdmContract, getBucketContract, MACHINE_INITIALIZED_TOPIC } from '../utils/contracts';
import { QueryResult, ObjectEntry, FileItem } from '../types';

export function useBucket(signer: ethers.Signer | null, address: string | null) {
  const [bucketAddress, setBucketAddress] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchBuckets = useCallback(async () => {
    if (!signer || !address) return [];

    setIsLoading(true);
    setError(null);

    try {
      const config = getConfig();
      // Use provider for view calls to avoid MetaMask issues
      const provider = await signer.provider;
      if (!provider) throw new Error('No provider available');
      const contract = getAdmContract(config.admActor, provider);
      // listBuckets returns array of (kind, addr, metadata[])
      const machines = await contract.listBuckets(address);

      console.log('listBuckets raw result:', machines);

      // ethers.js v6 returns tuples as arrays, access by index
      // Machine = [kind, addr, metadata[]]
      const buckets: string[] = [];
      for (const m of machines) {
        // Access as array: m[0] = kind, m[1] = addr, m[2] = metadata
        const kind = typeof m.kind !== 'undefined' ? m.kind : m[0];
        const addr = typeof m.addr !== 'undefined' ? m.addr : m[1];
        console.log('Machine:', { kind, addr });
        if (Number(kind) === 0) {
          buckets.push(addr);
        }
      }

      console.log('Filtered buckets:', buckets);

      if (buckets.length > 0) {
        setBucketAddress(buckets[0]); // Use the first bucket
      }

      return buckets;
    } catch (err: unknown) {
      const error = err as Error;
      console.error('fetchBuckets error:', err);
      setError(error.message || 'Failed to fetch buckets');
      return [];
    } finally {
      setIsLoading(false);
    }
  }, [signer, address]);

  const createBucket = useCallback(async () => {
    if (!signer) {
      setError('Wallet not connected');
      return null;
    }

    setIsCreating(true);
    setError(null);

    try {
      const config = getConfig();
      const contract = getAdmContract(config.admActor, signer);
      const tx = await contract.createBucket();
      const receipt = await tx.wait();

      // Extract bucket address from MachineInitialized event
      let newBucketAddress: string | null = null;
      for (const log of receipt.logs) {
        if (log.topics[0] === MACHINE_INITIALIZED_TOPIC) {
          // The address is in the data field (last 20 bytes of 32-byte word)
          const data = log.data;
          newBucketAddress = '0x' + data.slice(26, 66);
          break;
        }
      }

      if (newBucketAddress) {
        setBucketAddress(newBucketAddress);
      }

      return newBucketAddress;
    } catch (err: unknown) {
      const error = err as Error;
      setError(error.message || 'Failed to create bucket');
      return null;
    } finally {
      setIsCreating(false);
    }
  }, [signer]);

  const selectBucket = useCallback((address: string) => {
    setBucketAddress(address);
  }, []);

  return {
    bucketAddress,
    isLoading,
    isCreating,
    error,
    fetchBuckets,
    createBucket,
    selectBucket,
    hasBucket: !!bucketAddress,
  };
}

export function useFileExplorer(signer: ethers.Signer | null, bucketAddress: string | null) {
  const [files, setFiles] = useState<FileItem[]>([]);
  const [currentPath, setCurrentPath] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchFiles = useCallback(async (prefix: string = '') => {
    if (!signer || !bucketAddress) return;

    setIsLoading(true);
    setError(null);

    try {
      // Use provider for view calls to avoid MetaMask issues
      const provider = await signer.provider;
      if (!provider) throw new Error('No provider available');
      const contract = getBucketContract(bucketAddress, provider);

      let result: QueryResult;
      if (prefix) {
        result = await contract['queryObjects(string,string)'](prefix, '/');
      } else {
        result = await contract['queryObjects(string,string)']('', '/');
      }

      const fileItems: FileItem[] = [];

      // Add folders from commonPrefixes
      for (const folderPath of result.commonPrefixes) {
        const name = folderPath.slice(prefix.length).replace(/\/$/, '');
        if (name) {
          fileItems.push({
            name,
            fullPath: folderPath,
            isFolder: true,
          });
        }
      }

      // Add files from objects
      console.log('queryObjects result:', result);
      console.log('objects:', result.objects);
      for (const obj of result.objects) {
        console.log('Raw object:', obj);
        const objEntry = obj as unknown as ObjectEntry;
        const key = objEntry.key || (obj as unknown as { 0: string })[0];
        const state = objEntry.state || (obj as unknown as { 1: { 0: string; 1: bigint; 2: bigint } })[1];

        console.log('Parsed object:', { key, state });

        const name = key.slice(prefix.length);
        if (name && !name.includes('/')) {
          const fileItem = {
            name,
            fullPath: key,
            isFolder: false,
            size: state.size ?? (state as unknown as { 1: bigint })[1],
            expiry: state.expiry ?? (state as unknown as { 2: bigint })[2],
            blobHash: state.blobHash ?? (state as unknown as { 0: string })[0],
          };
          console.log('FileItem:', fileItem);
          fileItems.push(fileItem);
        }
      }

      console.log('Final fileItems:', fileItems);
      setFiles(fileItems);
      setCurrentPath(prefix);
    } catch (err: unknown) {
      const error = err as Error;
      console.error('fetchFiles error:', err);
      setError(error.message || 'Failed to fetch files');
    } finally {
      setIsLoading(false);
    }
  }, [signer, bucketAddress]);

  const navigateToFolder = useCallback((folderPath: string) => {
    fetchFiles(folderPath);
  }, [fetchFiles]);

  const navigateUp = useCallback(() => {
    if (!currentPath) return;
    const parts = currentPath.split('/').filter(Boolean);
    parts.pop();
    const newPath = parts.length > 0 ? parts.join('/') + '/' : '';
    fetchFiles(newPath);
  }, [currentPath, fetchFiles]);

  const refresh = useCallback(() => {
    fetchFiles(currentPath);
  }, [fetchFiles, currentPath]);

  const [isDeleting, setIsDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  const deleteObject = useCallback(async (key: string) => {
    if (!signer || !bucketAddress) {
      setDeleteError('Wallet or bucket not connected');
      return false;
    }

    setIsDeleting(true);
    setDeleteError(null);

    try {
      const contract = getBucketContract(bucketAddress, signer);
      const tx = await contract.deleteObject(key);
      await tx.wait();

      // Refresh the file list after deletion
      await fetchFiles(currentPath);
      return true;
    } catch (err: unknown) {
      const error = err as Error;
      console.error('deleteObject error:', err);
      setDeleteError(error.message || 'Failed to delete object');
      return false;
    } finally {
      setIsDeleting(false);
    }
  }, [signer, bucketAddress, fetchFiles, currentPath]);

  return {
    files,
    currentPath,
    isLoading,
    error,
    fetchFiles,
    navigateToFolder,
    navigateUp,
    refresh,
    deleteObject,
    isDeleting,
    deleteError,
  };
}
