import { useState, useCallback } from 'react';
import { getConfig } from '../utils/config';

export function useDownload() {
  const [isDownloading, setIsDownloading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const downloadFile = useCallback(async (blobHash: string, fileName: string) => {
    console.log('downloadFile called:', { blobHash, fileName });
    setIsDownloading(true);
    setError(null);

    try {
      const config = getConfig();

      // Remove 0x prefix if present
      const hash = blobHash.startsWith('0x') ? blobHash.slice(2) : blobHash;
      console.log('Fetching from:', `${config.objectsListenAddr}/v1/blobs/${hash}`);

      const response = await fetch(`${config.objectsListenAddr}/v1/blobs/${hash}`);

      if (!response.ok) {
        throw new Error(`Download failed: ${response.statusText}`);
      }

      const blob = await response.blob();

      // Create download link
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      return true;
    } catch (err: unknown) {
      const error = err as Error;
      setError(error.message || 'Download failed');
      return false;
    } finally {
      setIsDownloading(false);
    }
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    isDownloading,
    error,
    downloadFile,
    clearError,
  };
}
