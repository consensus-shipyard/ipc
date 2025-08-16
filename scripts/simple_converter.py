#!/usr/bin/env python3
"""
Simple script to convert f410 address to Ethereum address.
No external dependencies required.
"""

import base64

def crockford_base32_decode(data):
    """
    Decode Crockford base32 data.
    Crockford base32 uses: 0123456789ABCDEFGHJKMNPQRSTVWXYZ
    Standard base32 uses:  ABCDEFGHIJKLMNOPQRSTUVWXYZ234567
    """
    # Map Crockford to standard base32
    crockford_alphabet = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
    standard_alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"

    # Create translation table
    translate_table = str.maketrans(crockford_alphabet, standard_alphabet)

    # Convert to uppercase and translate
    data_upper = data.upper()
    standard_data = data_upper.translate(translate_table)

    # Add padding if needed
    while len(standard_data) % 8 != 0:
        standard_data += '='

    try:
        decoded = base64.b32decode(standard_data)
        return decoded
    except Exception as e:
        print(f"Base32 decode error: {e}")
        return None

def f410_to_eth_address(f410_addr):
    """Convert f410 address to Ethereum address."""

    print(f"Converting: {f410_addr}")

    # Remove prefix
    if f410_addr.startswith('t410'):
        without_prefix = f410_addr[4:]
    elif f410_addr.startswith('f410'):
        without_prefix = f410_addr[4:]
    else:
        print("Invalid f410 address format")
        return None

    print(f"Without prefix: {without_prefix}")

    # Try Crockford base32 decoding
    decoded_bytes = crockford_base32_decode(without_prefix)

    if decoded_bytes:
        print(f"Decoded {len(decoded_bytes)} bytes: {decoded_bytes.hex()}")

        # For delegated addresses, we want the last 20 bytes
        if len(decoded_bytes) >= 20:
            eth_bytes = decoded_bytes[-20:]
            eth_address = '0x' + eth_bytes.hex()
            return eth_address
        else:
            print(f"Decoded bytes too short: {len(decoded_bytes)} bytes")

    # Fallback: try to interpret different ways
    print("\nTrying alternative interpretations...")

    # Try interpreting as hex
    try:
        if len(without_prefix) >= 40:
            potential_hex = without_prefix[-40:]  # Last 40 chars
            # Verify it's valid hex
            int(potential_hex, 16)
            return '0x' + potential_hex
    except ValueError:
        pass

    # Try a different base32 approach
    try:
        # Maybe it's standard base32
        padded = without_prefix.upper()
        while len(padded) % 8 != 0:
            padded += '='

        decoded = base64.b32decode(padded)
        if len(decoded) >= 20:
            eth_bytes = decoded[-20:]
            return '0x' + eth_bytes.hex()
    except:
        pass

    return None

def main():
    f410_address = 't410f5kakfhdd3amp56oqwpxbupfalxsbjjrqilas5my'

    print("=== Simple F410 to Ethereum Address Converter ===")
    print(f"Input: {f410_address}")
    print()

    eth_address = f410_to_eth_address(f410_address)

    if eth_address:
        print(f"\n✅ SUCCESS!")
        print(f"Ethereum Address: {eth_address}")
        print(f"\nNow you can query this contract address:")
        print(f"Function: permissionMode()")
        print(f"Expected return: uint8 (0=Collateral, 1=Federated, 2=Static)")
        print(f"\nExample cast command:")
        print(f"cast call {eth_address} \"permissionMode()\" --rpc-url YOUR_RPC_URL")
    else:
        print("\n❌ FAILED to convert address")
        print("\nThe address might need manual conversion using:")
        print("1. Filecoin address utilities")
        print("2. Online base32 decoders")
        print("3. Filecoin explorer tools")

if __name__ == "__main__":
    main()