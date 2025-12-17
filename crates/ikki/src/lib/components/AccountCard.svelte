<script lang="ts">
  import { Copy, RefreshCw, Shield } from "lucide-svelte";
  import { ui } from "../stores/ui";
  import { formatZec, truncateAddress, copyToClipboard } from "../utils/format";

  export let balance: number = 0;
  export let address: string = "";
  export let syncing: boolean = false;

  let copied = false;

  async function handleCopy() {
    const success = await copyToClipboard(address);
    if (success) {
      copied = true;
      ui.showToast("Address copied", "success");
      setTimeout(() => (copied = false), 2000);
    }
  }
</script>

<div class="account-card">
  <div class="card-glow"></div>
  <div class="card-inner">
    <div class="card-header">
      <div class="label-row">
        <Shield size={12} strokeWidth={2.5} />
        <span class="card-label">Shielded Balance</span>
      </div>
      {#if syncing}
        <div class="sync-badge">
          <RefreshCw size={11} class="spin" />
          <span>Syncing</span>
        </div>
      {/if}
    </div>

    <div class="balance-section">
      <span class="balance-value">{formatZec(balance)}</span>
      <span class="balance-currency">ZEC</span>
    </div>

    <button class="address-row" onclick={handleCopy} class:copied>
      <span class="address-text">{truncateAddress(address, 12)}</span>
      <div class="copy-icon">
        <Copy size={13} strokeWidth={2} />
      </div>
    </button>
  </div>
</div>

<style>
  .account-card {
    position: relative;
    border-radius: var(--radius-xl);
    overflow: hidden;
  }

  .card-glow {
    position: absolute;
    inset: 0;
    background: var(--gradient-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
  }

  .card-glow::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg,
      transparent 0%,
      rgba(255,255,255,0.08) 20%,
      rgba(255,255,255,0.08) 80%,
      transparent 100%
    );
  }

  .card-glow::after {
    content: '';
    position: absolute;
    inset: 0;
    background: var(--gradient-glow);
    pointer-events: none;
  }

  .card-inner {
    position: relative;
    z-index: 1;
    padding: var(--space-5) var(--space-5) var(--space-4);
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }

  .label-row {
    display: flex;
    align-items: center;
    gap: var(--space-1\.5);
    color: var(--text-tertiary);
  }

  .card-label {
    font-size: var(--text-caption);
    font-weight: var(--weight-medium);
    text-transform: uppercase;
    letter-spacing: var(--tracking-widest);
  }

  .sync-badge {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
    font-weight: var(--weight-medium);
    color: var(--text-tertiary);
    padding: 3px 8px;
    background: var(--bg-primary);
    border-radius: var(--radius-full);
    letter-spacing: 0.02em;
  }

  .sync-badge :global(.spin) {
    animation: spin 1s linear infinite;
  }

  .balance-section {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }

  .balance-value {
    font-size: var(--text-display);
    font-weight: var(--weight-bold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tight);
    line-height: var(--leading-none);
    font-variant-numeric: tabular-nums;
  }

  .balance-currency {
    font-size: var(--text-small);
    font-weight: var(--weight-semibold);
    color: var(--text-tertiary);
    letter-spacing: 0.05em;
  }

  .address-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--space-2\.5) var(--space-3);
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .address-row:hover {
    background: rgba(0, 0, 0, 0.4);
    border-color: var(--border-light);
  }

  .address-row:active {
    transform: scale(0.99);
  }

  .address-row.copied {
    border-color: var(--success);
  }

  .address-row.copied .copy-icon {
    color: var(--success);
  }

  .address-text {
    font-family: var(--font-mono);
    font-size: var(--text-small);
    color: var(--text-secondary);
    letter-spacing: 0.02em;
  }

  .copy-icon {
    color: var(--text-tertiary);
    transition: color var(--transition-fast);
  }

  .address-row:hover .copy-icon {
    color: var(--text-secondary);
  }
</style>
