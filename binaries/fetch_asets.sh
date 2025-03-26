#!/bin/bash
set -e

# Variables
RELEASE_API_URL="https://api.github.com/repos/cometbft/cometbft/releases/tags/v0.37.15"
DOWNLOAD_DIR="cometbft_v0.37.15_assets"

# Check for required commands
for cmd in curl jq tar; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "Error: $cmd is required. Please install it and try again."
        exit 1
    fi
done

# Create download directory if it doesn't exist
mkdir -p "$DOWNLOAD_DIR"

echo "Fetching release information from $RELEASE_API_URL ..."
release_json=$(curl -s "$RELEASE_API_URL")

# Parse JSON to get each asset's name and download URL
assets=$(echo "$release_json" | jq -r '.assets[] | "\(.name) \(.browser_download_url)"')

if [ -z "$assets" ]; then
  echo "No assets found for this release."
  exit 1
fi

# Download and process each asset
echo "$assets" | while read -r asset_name asset_url; do
    echo "Downloading $asset_name from $asset_url ..."
    asset_path="$DOWNLOAD_DIR/$asset_name"
    curl -L -o "$asset_path" "$asset_url"
    
    # Process tar.gz files: extract and move the binary
    if [[ "$asset_name" =~ \.tar\.gz$ ]]; then
        # Remove the .tar.gz extension for the base name
        base_name="${asset_name%.tar.gz}"
        dest_file="$DOWNLOAD_DIR/$base_name"
        
        # Determine the expected binary name
        binary_name="cometbft"
        if [[ "$asset_name" == *windows* ]]; then
            binary_name="cometbft.exe"
        fi
        
        # Create a temporary extraction directory
        temp_extract_dir="$DOWNLOAD_DIR/temp_$base_name"
        mkdir -p "$temp_extract_dir"
        
        echo "Extracting $asset_name ..."
        tar -xzf "$asset_path" -C "$temp_extract_dir"
        
        # Locate the binary file in the temporary directory
        binary_path=$(find "$temp_extract_dir" -type f -name "$binary_name" | head -n 1)
        
        if [ -z "$binary_path" ]; then
            echo "Error: $binary_name file not found in $temp_extract_dir"
        else
            echo "Moving $(basename "$binary_path") to $dest_file ..."
            mv "$binary_path" "$dest_file"
        fi
        
        # Remove the temporary extraction directory
        rm -rf "$temp_extract_dir"
    fi
done

echo "All assets processed."
