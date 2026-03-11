import React, { useEffect, useRef, useState } from 'react';
import { FileItem } from '../types';

interface FileExplorerProps {
  files: FileItem[];
  currentPath: string;
  isLoading: boolean;
  isUploading: boolean;
  isDeleting: boolean;
  uploadProgress: string;
  error: string | null;
  uploadError: string | null;
  deleteError: string | null;
  onNavigateToFolder: (path: string) => void;
  onNavigateUp: () => void;
  onRefresh: () => void;
  onUpload: (file: File, targetPath: string, dataShards: number, parityShards: number) => Promise<boolean>;
  onDownload: (blobHash: string, fileName: string) => Promise<boolean>;
  onDelete: (key: string) => Promise<boolean>;
  onFetchFiles: (prefix: string) => void;
}

export function FileExplorer({
  files,
  currentPath,
  isLoading,
  isUploading,
  isDeleting,
  uploadProgress,
  error,
  uploadError,
  deleteError,
  onNavigateToFolder,
  onNavigateUp,
  onRefresh,
  onUpload,
  onDownload,
  onDelete,
  onFetchFiles,
}: FileExplorerProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [newFolderName, setNewFolderName] = useState('');
  const [showNewFolderInput, setShowNewFolderInput] = useState(false);
  const [pendingFile, setPendingFile] = useState<File | null>(null);
  const [showUploadConfig, setShowUploadConfig] = useState(false);
  const [dataShards, setDataShards] = useState(4);
  const [parityShards, setParityShards] = useState(2);

  useEffect(() => {
    onFetchFiles(currentPath);
  }, [onFetchFiles, currentPath]);

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setPendingFile(file);
      setShowUploadConfig(true);
    }
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  const handleUploadConfirm = async () => {
    if (!pendingFile) return;
    setShowUploadConfig(false);
    const success = await onUpload(pendingFile, currentPath, dataShards, parityShards);
    if (success) {
      onRefresh();
    }
    setPendingFile(null);
  };

  const handleUploadCancel = () => {
    setShowUploadConfig(false);
    setPendingFile(null);
  };

  const handleCreateFolder = () => {
    if (newFolderName.trim()) {
      const folderPath = currentPath + newFolderName.trim() + '/';
      onNavigateToFolder(folderPath);
      setNewFolderName('');
      setShowNewFolderInput(false);
    }
  };

  const formatSize = (size?: bigint) => {
    if (!size) return '-';
    const bytes = Number(size);
    if (bytes < 1000) return `${bytes} B`;
    if (bytes < 1000 * 1000) return `${(bytes / 1000).toFixed(1)} KB`;
    if (bytes < 1000 * 1000 * 1000) return `${(bytes / (1000 * 1000)).toFixed(1)} MB`;
    return `${(bytes / (1000 * 1000 * 1000)).toFixed(1)} GB`;
  };

  const getBreadcrumbs = () => {
    const parts = currentPath.split('/').filter(Boolean);
    const crumbs = [{ name: 'Home', path: '' }];
    let path = '';
    for (const part of parts) {
      path += part + '/';
      crumbs.push({ name: part, path });
    }
    return crumbs;
  };

  return (
    <div className="file-explorer">
      <div className="explorer-toolbar">
        <div className="breadcrumbs">
          {getBreadcrumbs().map((crumb, index, arr) => (
            <React.Fragment key={crumb.path}>
              <button
                className="breadcrumb"
                onClick={() => onFetchFiles(crumb.path)}
                disabled={index === arr.length - 1}
              >
                {crumb.name}
              </button>
              {index < arr.length - 1 && <span className="separator">/</span>}
            </React.Fragment>
          ))}
        </div>

        <div className="toolbar-actions">
          <button
            onClick={() => onNavigateUp()}
            disabled={!currentPath || isLoading}
            className="btn btn-icon"
            title="Go up"
          >
            ..
          </button>
          <button
            onClick={onRefresh}
            disabled={isLoading}
            className="btn btn-icon"
            title="Refresh"
          >
            Refresh
          </button>
          <button
            onClick={() => setShowNewFolderInput(!showNewFolderInput)}
            className="btn btn-icon"
            title="New folder"
          >
            + Folder
          </button>
          <button
            onClick={() => fileInputRef.current?.click()}
            disabled={isUploading}
            className="btn btn-primary"
          >
            {isUploading ? uploadProgress : 'Upload File'}
          </button>
          <input
            ref={fileInputRef}
            type="file"
            onChange={handleFileSelect}
            style={{ display: 'none' }}
          />
        </div>
      </div>

      {showNewFolderInput && (
        <div className="new-folder-input">
          <input
            type="text"
            value={newFolderName}
            onChange={(e) => setNewFolderName(e.target.value)}
            placeholder="Folder name"
            className="input"
            onKeyDown={(e) => e.key === 'Enter' && handleCreateFolder()}
          />
          <button onClick={handleCreateFolder} className="btn btn-primary">
            Create
          </button>
          <button
            onClick={() => {
              setShowNewFolderInput(false);
              setNewFolderName('');
            }}
            className="btn btn-secondary"
          >
            Cancel
          </button>
        </div>
      )}

      {showUploadConfig && pendingFile && (
        <div className="upload-config">
          <div className="upload-config-header">
            <strong>Upload: {pendingFile.name}</strong>
          </div>
          <div className="upload-config-fields">
            <label className="upload-config-label">
              Data chunks (k):
              <input
                type="number"
                min="1"
                value={dataShards}
                onChange={(e) => setDataShards(parseInt(e.target.value) || 1)}
                className="input input-small"
              />
            </label>
            <label className="upload-config-label">
              Parity chunks (m):
              <input
                type="number"
                min="1"
                value={parityShards}
                onChange={(e) => setParityShards(parseInt(e.target.value) || 1)}
                className="input input-small"
              />
            </label>
          </div>
          <div className="upload-config-actions">
            <button onClick={handleUploadConfirm} className="btn btn-primary">
              Upload
            </button>
            <button onClick={handleUploadCancel} className="btn btn-secondary">
              Cancel
            </button>
          </div>
        </div>
      )}

      {(error || uploadError || deleteError) && (
        <p className="error">{error || uploadError || deleteError}</p>
      )}

      {isLoading ? (
        <div className="loading">Loading files...</div>
      ) : files.length === 0 ? (
        <div className="empty-state">
          <p>This folder is empty</p>
          <p className="hint">Upload a file or create a folder to get started</p>
        </div>
      ) : (
        <div className="file-list">
          <div className="file-header">
            <span className="col-name">Name</span>
            <span className="col-size">Size</span>
            <span className="col-actions">Actions</span>
          </div>
          {files.map((file) => (
            <div key={file.fullPath} className="file-row">
              <span className="col-name">
                {file.isFolder ? (
                  <button
                    className="folder-link"
                    onClick={() => onNavigateToFolder(file.fullPath)}
                  >
                    <span className="icon folder-icon">Folder</span>
                    {file.name}
                  </button>
                ) : (
                  <span className="file-name">
                    <span className="icon file-icon">File</span>
                    {file.name}
                  </span>
                )}
              </span>
              <span className="col-size">{formatSize(file.size)}</span>
              <span className="col-actions">
                {!file.isFolder && file.blobHash && (
                  <>
                    <button
                      onClick={() => onDownload(file.blobHash!, file.name)}
                      className="btn btn-small"
                    >
                      Download
                    </button>
                    <button
                      onClick={() => onDelete(file.fullPath)}
                      className="btn btn-small btn-danger"
                      disabled={isDeleting}
                    >
                      {isDeleting ? 'Deleting...' : 'Delete'}
                    </button>
                  </>
                )}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
