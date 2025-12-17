/**
 * Format zatoshis to ZEC display string
 */
export function formatZec(zatoshis: number): string {
  const zec = zatoshis / 100_000_000;
  return zec.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 8,
  });
}

/**
 * Format ZEC with currency symbol
 */
export function formatZecAmount(zatoshis: number): string {
  return `${formatZec(zatoshis)} ZEC`;
}

/**
 * Parse ZEC string to zatoshis
 */
export function parseZec(zecString: string): number {
  const zec = parseFloat(zecString);
  if (isNaN(zec)) return 0;
  return Math.floor(zec * 100_000_000);
}

/**
 * Truncate address for display
 */
export function truncateAddress(address: string, chars: number = 8): string {
  if (address.length <= chars * 2 + 3) return address;
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}

/**
 * Truncate transaction ID
 */
export function truncateTxid(txid: string): string {
  return truncateAddress(txid, 10);
}

/**
 * Format timestamp to relative time
 */
export function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp * 1000;

  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return "Just now";
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;

  return new Date(timestamp * 1000).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

/**
 * Format block height
 */
export function formatBlockHeight(height: number): string {
  return height.toLocaleString("en-US");
}

/**
 * Copy text to clipboard
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}
