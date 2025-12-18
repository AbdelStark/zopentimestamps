<script lang="ts">
  import { Copy, Check, RefreshCw, Shield } from "lucide-svelte";
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

  $: formattedBalance = formatZec(balance);
  $: [intPart, decPart] = formattedBalance.includes('.')
    ? formattedBalance.split('.')
    : [formattedBalance, '00'];
</script>

<div class="card">
  <div class="card-bg"></div>
  <div class="card-content">
    <div class="card-header">
      <div class="badge">
        <Shield size={10} strokeWidth={2.5} />
        <span>Shielded</span>
      </div>
      {#if syncing}
        <div class="sync-indicator">
          <RefreshCw size={12} class="spinning" />
        </div>
      {/if}
    </div>

    <div class="balance">
      <span class="balance-int">{intPart}</span>
      <span class="balance-dec">.{decPart}</span>
      <span class="balance-unit">ZEC</span>
    </div>

    <button class="address-btn" onclick={handleCopy} class:copied>
      <span class="address-text">{truncateAddress(address, 10)}</span>
      <div class="address-icon">
        {#if copied}
          <Check size={12} strokeWidth={2.5} />
        {:else}
          <Copy size={12} strokeWidth={2} />
        {/if}
      </div>
    </button>
  </div>
</div>

<style>
  .card {
    position: relative;
    border-radius: var(--radius-xl);
    overflow: hidden;
  }

  .card-bg {
    position: absolute;
    inset: 0;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
  }

  .card-bg::before {
    content: '';
    position: absolute;
    inset: 0;
    background: var(--gradient-card);
    border-radius: inherit;
  }

  .card-bg::after {
    content: '';
    position: absolute;
    top: 0;
    left: 10%;
    right: 10%;
    height: 1px;
    background: linear-gradient(90deg,
      transparent,
      rgba(255, 255, 255, 0.08),
      transparent
    );
  }

  .card-content {
    position: relative;
    z-index: 1;
    padding: var(--space-5);
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    background: var(--accent-muted);
    border-radius: var(--radius-full);
    color: var(--text-tertiary);
  }

  .badge span {
    font-size: 10px;
    font-weight: var(--font-medium);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
  }

  .sync-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-full);
    background: var(--bg-elevated);
    color: var(--text-tertiary);
  }

  .sync-indicator :global(.spinning) {
    animation: spin 1.2s linear infinite;
  }

  .balance {
    display: flex;
    align-items: baseline;
    gap: 2px;
    margin-bottom: var(--space-5);
  }

  .balance-int {
    font-size: var(--text-2xl);
    font-weight: var(--font-bold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tighter);
    line-height: var(--leading-none);
    font-variant-numeric: tabular-nums;
  }

  .balance-dec {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
    color: var(--text-tertiary);
    letter-spacing: var(--tracking-tight);
    font-variant-numeric: tabular-nums;
  }

  .balance-unit {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-tertiary);
    margin-left: var(--space-2);
    letter-spacing: var(--tracking-wider);
  }

  .address-btn {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--space-3) var(--space-4);
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: all var(--duration-fast) var(--ease-out);
  }

  .address-btn:hover {
    background: rgba(0, 0, 0, 0.35);
    border-color: var(--border-emphasis);
  }

  .address-btn:active {
    transform: scale(0.99);
  }

  .address-btn.copied {
    border-color: var(--success);
  }

  .address-text {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    letter-spacing: var(--tracking-wide);
  }

  .address-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    transition: color var(--duration-fast) var(--ease-out);
  }

  .address-btn:hover .address-icon {
    color: var(--text-secondary);
  }

  .address-btn.copied .address-icon {
    color: var(--success);
  }
</style>
