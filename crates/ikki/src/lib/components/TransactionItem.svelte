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
  $: subtitle = address ? truncateAddress(address, 6) : memo ? (memo.length > 20 ? memo.slice(0, 20) + "..." : memo) : formatRelativeTime(timestamp);

  // Silence unused warning
  $: void txid;
</script>

<button class="transaction-item">
  <div class="tx-icon" class:outgoing={isOutgoing}>
    {#if isOutgoing}
      <ArrowUpRight size={16} strokeWidth={2} />
    {:else}
      <ArrowDownLeft size={16} strokeWidth={2} />
    {/if}
  </div>

  <div class="tx-info">
    <span class="tx-label">{labels[txType]}</span>
    <span class="tx-subtitle">{subtitle}</span>
  </div>

  <div class="tx-amount-section">
    <span class="tx-amount" class:outgoing={isOutgoing}>
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
    gap: var(--space-md);
    padding: var(--space-md);
    background: none;
    border: none;
    cursor: pointer;
    width: 100%;
    text-align: left;
    transition: background var(--transition-fast);
  }

  .transaction-item:hover {
    background: var(--bg-hover);
  }

  .tx-icon {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    background: var(--bg-elevated);
    color: var(--text-secondary);
  }

  .tx-icon.outgoing {
    color: var(--text-secondary);
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
    color: var(--text-tertiary);
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
    font-family: var(--font-mono);
    color: var(--text-primary);
  }

  .tx-amount.outgoing {
    color: var(--text-secondary);
  }

  .tx-status {
    font-size: var(--text-caption);
    font-weight: var(--weight-medium);
    padding: 2px var(--space-sm);
    border-radius: var(--radius-xs);
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
