#!/usr/bin/env python3
"""
Script to convert f410 Filecoin address to Ethereum address
and query subnet permission mode.

Requirements:
    pip install web3 base32-crockford

Usage:
    python3 address_converter.py
"""

import base64
import binascii
from web3 import Web3
import json

# Your subnet information
SUBNET_ID = '/r31337/t410f5kakfhdd3amp56oqwpxbupfalxsbjjrqilas5my'
F410_ADDRESS = 't410f5kakfhdd3amp56oqwpxbupfalxsbjjrqilas5my'

def f410_to_eth_address(f410_addr):
    """
    Convert f410 Filecoin address to Ethereum address.

    f410 addresses are delegated addresses that contain a 20-byte Ethereum address
    encoded using base32 (specifically Crockford base32).
    """
    try:
        # Remove the prefix (t410 or f410)
        if f410_addr.startswith('t410'):
            without_prefix = f410_addr[4:]
        elif f410_addr.startswith('f410'):
            without_prefix = f410_addr[4:]
        else:
            raise ValueError("Invalid f410 address format")

        print(f"Address without prefix: {without_prefix}")

        # Try to decode using base32 (Crockford variant)
        try:
            # Try with base32-crockford if available
            import base32_crockford
            decoded_bytes = base32_crockford.decode(without_prefix.upper())
        except ImportError:
            # Fallback to manual base32 decoding
            # Map Crockford base32 characters to standard base32
            crockford_to_standard = str.maketrans(
                '0123456789ABCDEFGHJKMNPQRSTVWXYZ',
                'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567'
            )

            # Convert to uppercase and translate
            standard_b32 = without_prefix.upper().translate(crockford_to_standard)

            # Pad if necessary
            while len(standard_b32) % 8 != 0:
                standard_b32 += '='

            try:
                decoded_bytes = base64.b32decode(standard_b32)
            except Exception as e:
                print(f"Standard base32 decode failed: {e}")
                # Try alternative approach - direct hex interpretation
                return try_hex_interpretation(without_prefix)

        print(f"Decoded bytes length: {len(decoded_bytes)}")
        print(f"Decoded bytes (hex): {decoded_bytes.hex()}")

        # For delegated addresses, we expect 20 bytes (Ethereum address)
        if len(decoded_bytes) >= 20:
            # Take the last 20 bytes as the Ethereum address
            eth_bytes = decoded_bytes[-20:]
            eth_address = '0x' + eth_bytes.hex()
            return eth_address
        else:
            print(f"Warning: Decoded bytes too short ({len(decoded_bytes)} bytes)")
            return None

    except Exception as e:
        print(f"Conversion error: {e}")
        return try_hex_interpretation(without_prefix)

def try_hex_interpretation(address_part):
    """
    Alternative approach: try to interpret as hex-encoded
    """
    try:
        print(f"\nTrying hex interpretation of: {address_part}")

        # Sometimes f410 addresses might be hex-encoded
        if len(address_part) == 40:  # 20 bytes * 2 hex chars
            return '0x' + address_part
        elif len(address_part) > 40:
            # Take last 40 characters
            return '0x' + address_part[-40:]
        else:
            print("Cannot interpret as hex - too short")
            return None

    except Exception as e:
        print(f"Hex interpretation failed: {e}")
        return None

def query_permission_mode(contract_address, rpc_url):
    """
    Query the subnet actor contract for permission mode.
    """
    try:
        # Connect to the network
        w3 = Web3(Web3.HTTPProvider(rpc_url))

        if not w3.is_connected():
            print(f"Failed to connect to {rpc_url}")
            return None

        print(f"Connected to network. Chain ID: {w3.eth.chain_id}")

        # Contract ABI for permissionMode function
        contract_abi = [
            {
                "inputs": [],
                "name": "permissionMode",
                "outputs": [{"internalType": "uint8", "name": "", "type": "uint8"}],
                "stateMutability": "view",
                "type": "function"
            }
        ]

        # Create contract instance
        contract = w3.eth.contract(
            address=Web3.to_checksum_address(contract_address),
            abi=contract_abi
        )

        # Call the permissionMode function
        permission_mode = contract.functions.permissionMode().call()

        # Map the result
        modes = {
            0: 'Collateral',
            1: 'Federated',
            2: 'Static'
        }

        print(f"\nPermission Mode: {modes.get(permission_mode, 'Unknown')} ({permission_mode})")
        return permission_mode

    except Exception as e:
        print(f"Query error: {e}")
        return None

def main():
    print("=== F410 TO ETHEREUM ADDRESS CONVERTER ===")
    print(f"Subnet ID: {SUBNET_ID}")
    print(f"F410 Address: {F410_ADDRESS}")
    print()

    # Convert the address
    eth_address = f410_to_eth_address(F410_ADDRESS)

    if eth_address:
        print(f"\n✅ Converted Ethereum Address: {eth_address}")

        # Ask user for RPC URL
        print(f"\nTo query the permission mode, you need an RPC endpoint for chain 31337.")
        print("Common options:")
        print("- Local node: http://localhost:8545")
        print("- Custom RPC: https://your-rpc-endpoint.com")

        rpc_url = input("\nEnter RPC URL (or press Enter to skip query): ").strip()

        if rpc_url:
            print(f"\nQuerying permission mode at {eth_address}...")
            query_permission_mode(eth_address, rpc_url)
        else:
            print(f"\nTo query manually, use this contract address: {eth_address}")
            print("Function to call: permissionMode()")
            print("Expected return: uint8 (0=Collateral, 1=Federated, 2=Static)")
    else:
        print("\n❌ Failed to convert address")
        print("\nManual steps to try:")
        print("1. Use Filecoin address utilities/libraries")
        print("2. Check if the address is in a different format")
        print("3. Try online Filecoin address converters")

if __name__ == "__main__":
    main()