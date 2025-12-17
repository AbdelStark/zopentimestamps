<script lang="ts">
  import { Copy, Shield, RefreshCw } from "lucide-svelte";
  import { wallet, isSyncing } from "../stores/wallet";
  import { ui } from "../stores/ui";
  import { formatZec, truncateAddress, copyToClipboard } from "../utils/format";

  export let balance: number = 0;
  export let address: string = "";
  export let syncing: boolean = false;

  async function handleCopy() {
    const success = await copyToClipboard(address);
    if (success) {
      ui.showToast("Address copied!", "success");
    }
  }
</script>

<div class="account-card">
  <div class="card-header">
    <div class="network-badge">
      <Shield size={12} />
      <span>Testnet</span>
    </div>
    {#if syncing}
      <div class="sync-indicator">
        <RefreshCw size={14} class="spin" />
        <span>Syncing...</span>
      </div>
    {/if}
  </div>

  <div class="balance-section">
    <span class="balance-label">Total Balance</span>
    <div class="balance-amount">
      <span class="balance-value">{formatZec(balance)}</span>
      <span class="balance-currency">ZEC</span>
    </div>
  </div>

  <div class="address-section">
    <button class="address-button" onclick={handleCopy}>
      <span class="address-text">{truncateAddress(address, 12)}</span>
      <Copy size={14} />
    </button>
  </div>
</div>

<style>
  .account-card {
    background: var(--gradient-card);
    border-radius: var(--radius-xl);
    padding: var(--space-lg);
    box-shadow: var(--shadow-card);
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
    position: relative;
    overflow: hidden;
  }

  .account-card::before {
    content: "";
    position: absolute;
    top: -50%;
    right: -50%;
    width: 100%;
    height: 100%;
    background: radial-gradient(
      circle,
      var(--accent-dim) 0%,
      transparent 60%
    );
    pointer-events: none;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    position: relative;
    z-index: 1;
  }

  .network-badge {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    background: var(--warning-dim);
    color: var(--warning);
    border-radius: var(--radius-full);
    font-size: var(--text-caption);
    font-weight: var(--weight-medium);
  }

  .sync-indicator {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    color: var(--text-secondary);
    font-size: var(--text-caption);
  }

  .sync-indicator :global(.spin) {
    animation: spin 1s linear infinite;
  }

  .balance-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    position: relative;
    z-index: 1;
  }

  .balance-label {
    font-size: var(--text-small);
    color: var(--text-secondary);
  }

  .balance-amount {
    display: flex;
    align-items: baseline;
    gap: var(--space-sm);
  }

  .balance-value {
    font-size: var(--text-display);
    font-weight: var(--weight-bold);
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .balance-currency {
    font-size: var(--text-h3);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .address-section {
    position: relative;
    z-index: 1;
  }

  .address-button {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: var(--text-small);
    font-family: var(--font-mono);
    cursor: pointer;
    transition: all var(--transition-fast);
    width: 100%;
    justify-content: center;
  }

  .address-button:hover {
    background: var(--bg-secondary);
    border-color: var(--border-light);
    color: var(--text-primary);
  }

  .address-text {
    flex: 1;
    text-align: center;
  }
</style>
