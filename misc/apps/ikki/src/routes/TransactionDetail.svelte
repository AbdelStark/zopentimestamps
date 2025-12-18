<script lang="ts">
  import {
    ArrowLeft,
    ArrowUpRight,
    ArrowDownLeft,
    Layers,
    RefreshCw,
    Copy,
    Check,
    ExternalLink,
    Clock,
    Hash,
    Wallet,
    FileText,
    Shield,
    CheckCircle2,
    AlertCircle,
    Loader2,
  } from "lucide-svelte";
  import { selectedTransaction, transaction } from "../lib/stores/transaction";
  import { ui } from "../lib/stores/ui";
  import { formatZec, truncateAddress, copyToClipboard } from "../lib/utils/format";

  let copiedField: string | null = null;

  $: tx = $selectedTransaction;

  $: isOutgoing = tx?.tx_type === "sent";
  $: displayAmount = tx ? Math.abs(tx.amount) : 0;

  const labels: Record<string, string> = {
    sent: "Sent",
    received: "Received",
    shielding: "Shielded",
    internal: "Internal",
  };

  const icons: Record<string, typeof ArrowUpRight> = {
    sent: ArrowUpRight,
    received: ArrowDownLeft,
    shielding: Layers,
    internal: RefreshCw,
  };

  const statusConfig: Record<string, { icon: typeof CheckCircle2; label: string; class: string }> = {
    confirmed: { icon: CheckCircle2, label: "Confirmed", class: "confirmed" },
    pending: { icon: Loader2, label: "Pending", class: "pending" },
    failed: { icon: AlertCircle, label: "Failed", class: "failed" },
  };

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString("en-US", {
      weekday: "short",
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  }

  function formatTime(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  async function handleCopy(text: string, field: string) {
    const success = await copyToClipboard(text);
    if (success) {
      copiedField = field;
      ui.showToast("Copied to clipboard", "success");
      setTimeout(() => (copiedField = null), 2000);
    }
  }

  function handleBack() {
    transaction.clear();
    ui.navigate("history");
  }

  function openExplorer() {
    if (tx?.txid) {
      // Testnet explorer URL
      window.open(`https://testnet.zcashblockexplorer.com/transactions/${tx.txid}`, "_blank");
    }
  }

  $: IconComponent = tx ? icons[tx.tx_type] : ArrowDownLeft;
  $: status = tx ? statusConfig[tx.status] : statusConfig.confirmed;
</script>

<div class="detail">
  <header class="detail-header">
    <button class="back-button" onclick={handleBack}>
      <ArrowLeft size={20} strokeWidth={2} />
    </button>
    <h1>Transaction</h1>
    <button class="explorer-button" onclick={openExplorer} title="View in Explorer">
      <ExternalLink size={18} strokeWidth={2} />
    </button>
  </header>

  {#if tx}
    <div class="detail-content">
      <!-- Hero Section -->
      <div class="hero-section">
        <div class="tx-icon" class:outgoing={isOutgoing} class:incoming={!isOutgoing}>
          <svelte:component this={IconComponent} size={24} strokeWidth={1.5} />
        </div>

        <div class="amount-display">
          <span class="amount-sign">{isOutgoing ? "-" : "+"}</span>
          <span class="amount-value">{formatZec(displayAmount)}</span>
          <span class="amount-unit">ZEC</span>
        </div>

        <div class="tx-type-badge">
          <span>{labels[tx.tx_type]}</span>
        </div>
      </div>

      <!-- Status Card -->
      <div class="status-card">
        <div class="status-indicator {status.class}">
          <svelte:component this={status.icon} size={14} strokeWidth={2.5} class={tx.status === "pending" ? "spin" : ""} />
          <span>{status.label}</span>
        </div>
        <div class="status-time">
          <Clock size={12} strokeWidth={2} />
          <span>{formatDate(tx.timestamp)} at {formatTime(tx.timestamp)}</span>
        </div>
      </div>

      <!-- Details Section -->
      <div class="details-section">
        <h2 class="section-title">Details</h2>

        <div class="details-card">
          <!-- Transaction ID -->
          <div class="detail-row">
            <div class="detail-label">
              <Hash size={14} strokeWidth={2} />
              <span>Transaction ID</span>
            </div>
            <button class="detail-value copyable" onclick={() => handleCopy(tx.txid, "txid")}>
              <span class="value-text mono">{truncateAddress(tx.txid, 10)}</span>
              {#if copiedField === "txid"}
                <Check size={14} strokeWidth={2.5} />
              {:else}
                <Copy size={14} strokeWidth={2} />
              {/if}
            </button>
          </div>

          <!-- Address (if available) -->
          {#if tx.address}
            <div class="detail-divider"></div>
            <div class="detail-row">
              <div class="detail-label">
                <Wallet size={14} strokeWidth={2} />
                <span>{isOutgoing ? "To" : "From"}</span>
              </div>
              <button class="detail-value copyable" onclick={() => handleCopy(tx.address || "", "address")}>
                <span class="value-text mono">{truncateAddress(tx.address, 8)}</span>
                {#if copiedField === "address"}
                  <Check size={14} strokeWidth={2.5} />
                {:else}
                  <Copy size={14} strokeWidth={2} />
                {/if}
              </button>
            </div>
          {/if}

          <!-- Confirmations -->
          <div class="detail-divider"></div>
          <div class="detail-row">
            <div class="detail-label">
              <Shield size={14} strokeWidth={2} />
              <span>Confirmations</span>
            </div>
            <div class="detail-value">
              <span class="value-text">{tx.confirmations}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Memo Section (if available) -->
      {#if tx.memo}
        <div class="memo-section">
          <h2 class="section-title">Memo</h2>
          <div class="memo-card">
            <div class="memo-header">
              <FileText size={14} strokeWidth={2} />
              <span>Encrypted Message</span>
            </div>
            <p class="memo-content">{tx.memo}</p>
            <button class="memo-copy" onclick={() => handleCopy(tx.memo || "", "memo")}>
              {#if copiedField === "memo"}
                <Check size={14} strokeWidth={2.5} />
                <span>Copied</span>
              {:else}
                <Copy size={14} strokeWidth={2} />
                <span>Copy</span>
              {/if}
            </button>
          </div>
        </div>
      {/if}

      <!-- Full Transaction ID -->
      <div class="full-txid-section">
        <h2 class="section-title">Full Transaction ID</h2>
        <div class="full-txid-card">
          <p class="full-txid">{tx.txid}</p>
          <button class="txid-copy" onclick={() => handleCopy(tx.txid, "full-txid")}>
            {#if copiedField === "full-txid"}
              <Check size={14} strokeWidth={2.5} />
            {:else}
              <Copy size={14} strokeWidth={2} />
            {/if}
          </button>
        </div>
      </div>
    </div>
  {:else}
    <div class="empty-state">
      <p>Transaction not found</p>
      <button class="back-link" onclick={handleBack}>Go back</button>
    </div>
  {/if}
</div>

<style>
  .detail {
    min-height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
  }

  .detail-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
  }

  .back-button,
  .explorer-button {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: var(--radius-md);
    transition:
      color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
    -webkit-tap-highlight-color: transparent;
  }

  .back-button:hover,
  .explorer-button:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .back-button:active,
  .explorer-button:active {
    transform: scale(0.95);
  }

  .detail-header h1 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-wide);
  }

  .detail-content {
    flex: 1;
    padding: var(--space-6) var(--space-5);
    padding-bottom: calc(var(--nav-height) + var(--space-6));
    max-width: var(--max-width);
    margin: 0 auto;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    animation: fadeIn var(--duration-normal) var(--ease-out);
  }

  /* Hero Section */
  .hero-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-6) 0;
  }

  .tx-icon {
    width: 64px;
    height: 64px;
    border-radius: var(--radius-xl);
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--border);
    animation: scaleIn var(--duration-normal) var(--ease-spring);
  }

  .tx-icon.outgoing {
    background: var(--bg-elevated);
    color: var(--text-secondary);
  }

  .tx-icon.incoming {
    background: var(--receive-muted);
    color: var(--text-primary);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .amount-display {
    display: flex;
    align-items: baseline;
    gap: var(--space-1);
  }

  .amount-sign {
    font-size: var(--text-xl);
    font-weight: var(--font-medium);
    color: var(--text-secondary);
  }

  .amount-value {
    font-size: 2.5rem;
    font-weight: var(--font-bold);
    font-family: var(--font-mono);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tight);
    font-variant-numeric: tabular-nums;
  }

  .amount-unit {
    font-size: var(--text-base);
    font-weight: var(--font-medium);
    color: var(--text-tertiary);
    margin-left: var(--space-1);
  }

  .tx-type-badge {
    display: inline-flex;
    align-items: center;
    padding: 6px 14px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-full);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    color: var(--text-secondary);
    letter-spacing: var(--tracking-wide);
  }

  /* Status Card */
  .status-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    position: relative;
  }

  .status-card::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--gradient-card);
    pointer-events: none;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    letter-spacing: var(--tracking-wide);
    position: relative;
  }

  .status-indicator.confirmed {
    color: var(--text-primary);
  }

  .status-indicator.pending {
    color: var(--text-tertiary);
  }

  .status-indicator.failed {
    color: var(--error);
  }

  .status-indicator :global(.spin) {
    animation: spin 1s linear infinite;
  }

  .status-time {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    letter-spacing: var(--tracking-wide);
    position: relative;
  }

  /* Section Title */
  .section-title {
    font-size: var(--text-2xs);
    font-weight: var(--font-semibold);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: var(--tracking-widest);
    padding-left: var(--space-1);
    margin-bottom: var(--space-2);
  }

  /* Details Card */
  .details-section,
  .memo-section,
  .full-txid-section {
    display: flex;
    flex-direction: column;
  }

  .details-card,
  .memo-card,
  .full-txid-card {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    position: relative;
  }

  .details-card::before,
  .memo-card::before,
  .full-txid-card::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--gradient-card);
    pointer-events: none;
  }

  .detail-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4);
    position: relative;
  }

  .detail-label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    letter-spacing: var(--tracking-wide);
  }

  .detail-value {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    background: none;
    border: none;
    cursor: default;
    padding: 0;
  }

  .detail-value.copyable {
    cursor: pointer;
    padding: 6px 10px;
    margin: -6px -10px;
    border-radius: var(--radius-md);
    transition: background var(--duration-fast) var(--ease-out);
  }

  .detail-value.copyable:hover {
    background: var(--bg-hover);
  }

  .detail-value.copyable:active {
    background: var(--bg-active);
  }

  .value-text {
    font-size: var(--text-xs);
    color: var(--text-primary);
    letter-spacing: var(--tracking-wide);
  }

  .value-text.mono {
    font-family: var(--font-mono);
    letter-spacing: var(--tracking-normal);
  }

  .detail-value :global(svg) {
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .detail-divider {
    height: 1px;
    background: var(--divider);
    margin: 0 var(--space-4);
  }

  /* Memo Card */
  .memo-card {
    padding: var(--space-4);
  }

  .memo-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-2xs);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    margin-bottom: var(--space-3);
    position: relative;
  }

  .memo-content {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    line-height: var(--leading-relaxed);
    word-break: break-word;
    margin-bottom: var(--space-3);
    position: relative;
  }

  .memo-copy {
    display: flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: none;
    font-size: var(--text-2xs);
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 6px 10px;
    margin: -6px -10px;
    border-radius: var(--radius-md);
    transition:
      color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out);
    position: relative;
  }

  .memo-copy:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }

  /* Full TXID */
  .full-txid-card {
    padding: var(--space-4);
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
  }

  .full-txid {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    color: var(--text-secondary);
    word-break: break-all;
    line-height: 1.8;
    position: relative;
  }

  .txid-copy {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
    position: relative;
  }

  .txid-copy:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
    border-color: var(--border-emphasis);
  }

  /* Empty State */
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-4);
    padding: var(--space-8);
    text-align: center;
  }

  .empty-state p {
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }

  .back-link {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  .back-link:hover {
    color: var(--text-primary);
  }
</style>
