<script lang="ts">
  import { ArrowUpRight, ArrowDownLeft } from "lucide-svelte";
  import { formatZec, truncateAddress, formatRelativeTime } from "../utils/format";

  export let txid: string;
  export let txType: "sent" | "received" | "shielding" | "internal";
  export let amount: number;
  export let timestamp: number;
  export let address: string | null = null;
  export let memo: string | null = null;
  export let status: "pending" | "confirmed" | "failed" = "confirmed";

  const labels = {
    sent: "Sent",
    received: "Received",
    shielding: "Shielded",
    internal: "Internal",
  };

  $: isOutgoing = txType === "sent" || txType === "shielding";
  $: displayAmount = Math.abs(amount);
  $: subtitle = address
    ? truncateAddress(address, 6)
    : memo
      ? memo.length > 24
        ? memo.slice(0, 24) + "..."
        : memo
      : formatRelativeTime(timestamp);

  // Silence unused warning
  $: void txid;
</script>

<button class="transaction-item">
  <div class="tx-icon" class:outgoing={isOutgoing} class:incoming={!isOutgoing}>
    {#if isOutgoing}
      <ArrowUpRight size={16} strokeWidth={2.25} />
    {:else}
      <ArrowDownLeft size={16} strokeWidth={2.25} />
    {/if}
  </div>

  <div class="tx-info">
    <span class="tx-label">{labels[txType]}</span>
    <span class="tx-subtitle">{subtitle}</span>
  </div>

  <div class="tx-amount-section">
    <span class="tx-amount" class:outgoing={isOutgoing} class:incoming={!isOutgoing}>
      {isOutgoing ? "-" : "+"}{formatZec(displayAmount)}
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
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: none;
    border: none;
    cursor: pointer;
    width: 100%;
    text-align: left;
    transition: background var(--transition-fast);
    -webkit-tap-highlight-color: transparent;
  }

  .transaction-item:hover {
    background: var(--bg-hover);
  }

  .transaction-item:active {
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
    transition: all var(--transition-fast);
  }

  .tx-icon.outgoing {
    background: var(--send-dim);
    color: var(--send);
  }

  .tx-icon.incoming {
    background: var(--receive-dim);
    color: var(--receive);
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
    letter-spacing: 0.01em;
  }

  .tx-subtitle {
    font-size: var(--text-caption);
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    letter-spacing: 0.01em;
  }

  .tx-amount-section {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
  }

  .tx-amount {
    font-size: var(--text-body);
    font-weight: var(--weight-semibold);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.01em;
  }

  .tx-amount.outgoing {
    color: var(--text-secondary);
  }

  .tx-amount.incoming {
    color: var(--receive);
  }

  .tx-status {
    font-size: 10px;
    font-weight: var(--weight-semibold);
    padding: 2px 6px;
    border-radius: var(--radius-xs);
    letter-spacing: 0.02em;
    text-transform: uppercase;
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
