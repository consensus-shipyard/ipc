import React, { useEffect } from 'react';

interface BucketManagerProps {
  bucketAddress: string | null;
  hasBucket: boolean;
  isLoading: boolean;
  isCreating: boolean;
  error: string | null;
  onFetchBuckets: () => Promise<string[]>;
  onCreateBucket: () => Promise<string | null>;
}

export function BucketManager({
  bucketAddress,
  hasBucket,
  isLoading,
  isCreating,
  error,
  onFetchBuckets,
  onCreateBucket,
}: BucketManagerProps) {
  useEffect(() => {
    onFetchBuckets();
  }, [onFetchBuckets]);

  const shortenAddress = (addr: string) =>
    `${addr.slice(0, 10)}...${addr.slice(-8)}`;

  if (isLoading) {
    return <div className="bucket-manager loading">Checking for buckets...</div>;
  }

  return (
    <div className="bucket-manager">
      <h3>Storage Bucket</h3>
      {hasBucket ? (
        <div className="bucket-info">
          <p>
            <strong>Bucket Address:</strong>{' '}
            <code>{shortenAddress(bucketAddress!)}</code>
          </p>
        </div>
      ) : (
        <div className="create-bucket">
          <p className="warning">You need a bucket to store files.</p>
          <button
            onClick={onCreateBucket}
            disabled={isCreating}
            className="btn btn-primary"
          >
            {isCreating ? 'Creating...' : 'Create Bucket'}
          </button>
        </div>
      )}

      {error && <p className="error">{error}</p>}
    </div>
  );
}
