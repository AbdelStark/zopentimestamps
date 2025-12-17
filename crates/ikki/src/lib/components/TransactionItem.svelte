<script lang="ts">
  import { ArrowUpRight, ArrowDownLeft, Shield, Circle } from "lucide-svelte";
  import { formatZec, truncateAddress, formatRelativeTime } from "../utils/format";

  export let txid: string;
  export let txType: "sent" | "received" | "shielding" | "internal";
  export let amount: number;
  export let timestamp: number;
  export let address: string | null = null;
  export let memo: string | null = null;
  export let status: "pending" | "confirmed" | "failed" = "confirmed";

  const icons = {
    sent: ArrowUpRight,
    received: ArrowDownLeft,
    shielding: Shield,
    internal: Circle,
  };

  const labels = {
    sent: "Sent",
    received: "Received",
    shielding: "Shielding",
    internal: "Internal",
  };

  $: Icon = icons[txType];
  $: isNegative = amount < 0;
  $: displayAmount = Math.abs(amount);
  $: subtitle = address ? truncateAddress(address, 6) : memo ? (memo.length > 24 ? memo.slice(0, 24) + "..." : memo) : formatRelativeTime(timestamp);
</script>

<button class="transaction-item">
  <div class="tx-icon tx-{txType}">
    <Icon size={18} />
  </div>

  <div class="tx-info">
    <span class="tx-label">{labels[txType]}</span>
    <span class="tx-subtitle">{subtitle}</span>
  </div>

  <div class="tx-amount-section">
    <span class="tx-amount" class:negative={isNegative} class:positive={!isNegative}>
      {isNegative ? "-" : "+"}{formatZec(displayAmount)} ZEC
    </span>
    {#if status === "pending"}
      <span class="tx-status pending">Pending</span>
    {:else if status === "failed"}
      <span class="tx-status failed">Failed</span>
    {/if}
  </div>
</button>

<style>
  .transaction-item {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md);
    background: none;
    border: none;
    cursor: pointer;
    width: 100%;
    text-align: left;
    border-radius: var(--radius-md);
    transition: background var(--transition-fast);
  }

  .transaction-item:hover {
    background: var(--bg-elevated);
  }

  .tx-icon {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .tx-icon.tx-sent {
    background: var(--error-dim);
    color: var(--send);
  }

  .tx-icon.tx-received {
    background: var(--success-dim);
    color: var(--receive);
  }

  .tx-icon.tx-shielding {
    background: rgba(191, 90, 242, 0.15);
    color: var(--shielded);
  }

  .tx-icon.tx-internal {
    background: var(--bg-elevated);
    color: var(--text-tertiary);
  }

  .tx-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .tx-label {
    font-size: var(--text-body);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }

  .tx-subtitle {
    font-size: var(--text-small);
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tx-amount-section {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
  }

  .tx-amount {
    font-size: var(--text-body);
    font-weight: var(--weight-medium);
  }

  .tx-amount.negative {
    color: var(--send);
  }

  .tx-amount.positive {
    color: var(--receive);
  }

  .tx-status {
    font-size: var(--text-caption);
    font-weight: var(--weight-medium);
    padding: 2px var(--space-sm);
    border-radius: var(--radius-full);
  }

  .tx-status.pending {
    background: var(--warning-dim);
    color: var(--warning);
  }

  .tx-status.failed {
    background: var(--error-dim);
    color: var(--error);
  }
</style>
