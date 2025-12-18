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
      <ArrowUpRight size={16} strokeWidth={2.5} />
    {:else}
      <ArrowDownLeft size={16} strokeWidth={2.5} />
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
    border-radius: var(--radius-md);
    transition:
      background var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
    -webkit-tap-highlight-color: transparent;
  }

  .transaction-item:hover {
    background: var(--bg-hover);
  }

  .transaction-item:active {
    background: var(--bg-active);
    transform: scale(0.99);
  }

  .tx-icon {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: transform var(--duration-fast) var(--ease-out);
  }

  .transaction-item:hover .tx-icon {
    transform: scale(1.05);
  }

  .tx-icon.outgoing {
    background: var(--send-muted);
    color: var(--send);
  }

  .tx-icon.incoming {
    background: var(--receive-muted);
    color: var(--receive);
  }

  .tx-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .tx-label {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    letter-spacing: var(--tracking-normal);
  }

  .tx-subtitle {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    letter-spacing: var(--tracking-wide);
  }

  .tx-amount-section {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 3px;
  }

  .tx-amount {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    letter-spacing: var(--tracking-tight);
  }

  .tx-amount.outgoing {
    color: var(--text-secondary);
  }

  .tx-amount.incoming {
    color: var(--receive);
  }

  .tx-status {
    font-size: 9px;
    font-weight: var(--font-semibold);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    letter-spacing: var(--tracking-wider);
    text-transform: uppercase;
  }

  .tx-status.pending {
    background: var(--warning-muted);
    color: var(--warning);
  }

  .tx-status.failed {
    background: var(--error-muted);
    color: var(--error);
  }
</style>
