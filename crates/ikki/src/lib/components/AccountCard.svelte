<script lang="ts">
  import { Copy, RefreshCw } from "lucide-svelte";
  import { ui } from "../stores/ui";
  import { formatZec, truncateAddress, copyToClipboard } from "../utils/format";

  export let balance: number = 0;
  export let address: string = "";
  export let syncing: boolean = false;

  async function handleCopy() {
    const success = await copyToClipboard(address);
    if (success) {
      ui.showToast("Address copied", "success");
    }
  }
</script>

<div class="account-card">
  <div class="card-inner">
    <div class="card-header">
      <span class="card-label">Total Balance</span>
      {#if syncing}
        <div class="sync-badge">
          <RefreshCw size={12} class="spin" />
          <span>Syncing</span>
        </div>
      {/if}
    </div>

    <div class="balance-section">
      <span class="balance-value">{formatZec(balance)}</span>
      <span class="balance-currency">ZEC</span>
    </div>

    <button class="address-row" onclick={handleCopy}>
      <span class="address-text">{truncateAddress(address, 10)}</span>
      <Copy size={14} />
    </button>
  </div>
</div>

<style>
  .account-card {
    background: var(--gradient-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-lg);
    position: relative;
    overflow: hidden;
  }

  .account-card::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(255,255,255,0.06), transparent);
  }

  .card-inner {
    position: relative;
    z-index: 1;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-md);
  }

  .card-label {
    font-size: var(--text-caption);
    font-weight: var(--weight-medium);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .sync-badge {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    font-size: var(--text-caption);
    color: var(--text-secondary);
  }

  .sync-badge :global(.spin) {
    animation: spin 1s linear infinite;
  }

  .balance-section {
    display: flex;
    align-items: baseline;
    gap: var(--space-sm);
    margin-bottom: var(--space-lg);
  }

  .balance-value {
    font-size: var(--text-display);
    font-weight: var(--weight-bold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tight);
    line-height: 1;
  }

  .balance-currency {
    font-size: var(--text-body);
    font-weight: var(--weight-medium);
    color: var(--text-tertiary);
  }

  .address-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .address-row:hover {
    background: var(--bg-secondary);
    border-color: var(--border-light);
  }

  .address-text {
    font-family: var(--font-mono);
    font-size: var(--text-small);
    color: var(--text-secondary);
  }

  .address-row :global(svg) {
    color: var(--text-tertiary);
  }
</style>
