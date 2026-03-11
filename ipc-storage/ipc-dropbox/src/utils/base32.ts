const BASE32_ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';

export function base32ToHex(base32: string): string {
  // Normalize: uppercase and add padding
  let input = base32.toUpperCase();
  const padding = (8 - (input.length % 8)) % 8;
  input = input + '='.repeat(padding);

  // Decode base32
  let bits = '';
  for (const char of input) {
    if (char === '=') break;
    const index = BASE32_ALPHABET.indexOf(char);
    if (index === -1) continue;
    bits += index.toString(2).padStart(5, '0');
  }

  // Convert bits to bytes
  const bytes: number[] = [];
  for (let i = 0; i + 8 <= bits.length; i += 8) {
    bytes.push(parseInt(bits.slice(i, i + 8), 2));
  }

  // Ensure exactly 32 bytes for hash
  while (bytes.length < 32) {
    bytes.push(0);
  }
  if (bytes.length > 32) {
    bytes.length = 32;
  }

  // Convert to hex
  return '0x' + bytes.map(b => b.toString(16).padStart(2, '0')).join('');
}
