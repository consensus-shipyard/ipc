/**
 * Address formatting utilities
 */

/**
 * Format an address from various input formats to a standard hex string
 */
export function formatAddress(address: unknown): string {
  if (!address) return 'N/A'

  // Handle different address formats
  let addressStr = ''

  if (typeof address === 'string') {
    // Already a string, check if it needs 0x prefix
    addressStr = address
  } else if (Array.isArray(address)) {
    // Handle byte arrays - convert to hex string
    if (address.length >= 20 && address.every(b => typeof b === 'number' && b >= 0 && b <= 255)) {
      // This is a 20-byte (or longer) Ethereum address as array of numbers
      // Take only the first 20 bytes for the address
      const addressBytes = address.slice(0, 20)
      addressStr = '0x' + addressBytes.map(b => b.toString(16).padStart(2, '0')).join('')
    } else {
      return 'N/A (invalid array)'
    }
  } else if (typeof address === 'object' && address !== null) {
    // Handle object format
    const obj = address as Record<string, unknown>
    if (obj.route && Array.isArray(obj.route)) {
      // Subnet ID format - extract the address from route
      const lastRoute = obj.route[obj.route.length - 1]
      if (lastRoute && Array.isArray(lastRoute) && lastRoute.length === 20) {
        addressStr = '0x' + lastRoute.map((b: unknown) => {
          if (typeof b === 'number') {
            return b.toString(16).padStart(2, '0')
          }
          return '00'
        }).join('')
      } else {
        return 'N/A (invalid route)'
      }
    } else {
      return 'N/A (invalid object)'
    }
  } else if (typeof address === 'number') {
    return 'N/A (single number)'
  } else {
    return 'N/A (unknown format)'
  }

  // Ensure we have a valid hex address format
  if (addressStr && !addressStr.startsWith('0x') && addressStr.length === 40) {
    addressStr = '0x' + addressStr
  }

  // Validate the address length
  if (addressStr.startsWith('0x') && addressStr.length !== 42) {
    return 'N/A (invalid length)'
  }

  return addressStr
}

/**
 * Format an address for short display (truncated)
 */
export function formatAddressShort(address: unknown): string {
  const fullAddress = formatAddress(address)
  if (fullAddress === 'N/A' || !fullAddress.startsWith('0x')) return fullAddress
  if (fullAddress.length < 14) return fullAddress // Don't truncate short addresses
  return `${fullAddress.slice(0, 8)}...${fullAddress.slice(-6)}`
}

/**
 * Extract subnet actor address from a subnet ID
 */
export function extractSubnetActorAddress(subnetId: string): string {
  if (!subnetId) return 'N/A'

  // Extract the subnet actor address from the subnet ID
  // For IPC subnets, the format is typically /r{chainId}/{actorAddress}
  // The address can be in Filecoin format (t410...) or Ethereum format (0x...)
  try {
    const parts = subnetId.split('/')

    if (parts.length >= 3) {
      // The last part should be the subnet actor address
      const actorAddress = parts[parts.length - 1]

      // Handle Filecoin t410 addresses (delegated/Ethereum-compatible addresses)
      if (actorAddress.startsWith('t410f')) {
        // t410f addresses are Filecoin representations of Ethereum addresses
        // The format is t410f{32-byte-address-in-base32}
        // For display purposes, we'll show the full Filecoin address
        return actorAddress
      }

      // Handle Ethereum addresses (with 0x prefix)
      if (actorAddress.startsWith('0x') && actorAddress.length === 42) {
        return actorAddress
      }

      // Handle raw hex addresses (40 hex chars without 0x)
      if (actorAddress.length === 40 && /^[a-fA-F0-9]+$/.test(actorAddress)) {
        return '0x' + actorAddress
      }

      // Handle other Filecoin address formats (f0, f1, f2, f3, f4)
      if (/^[tf][0-4]/.test(actorAddress)) {
        return actorAddress
      }
    }

    // Fallback: try to extract any address pattern from the raw ID
    // Look for Ethereum addresses
    const ethAddressMatch = subnetId.match(/0x[a-fA-F0-9]{40}/)
    if (ethAddressMatch) {
      return ethAddressMatch[0]
    }

    // Look for Filecoin addresses
    const filAddressMatch = subnetId.match(/[tf][0-4][a-zA-Z0-9]+/)
    if (filAddressMatch) {
      return filAddressMatch[0]
    }

    return 'N/A (unable to parse from subnet ID)'
  } catch (err) {
    console.warn('Error parsing subnet actor address from subnet ID:', err)
    return 'N/A (parse error)'
  }
}
