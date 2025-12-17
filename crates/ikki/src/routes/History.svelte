<script lang="ts">
  import { onMount } from "svelte";
  import { Loader2 } from "lucide-svelte";
  import { getTransactions, type Transaction } from "../lib/utils/tauri";
  import TransactionItem from "../lib/components/TransactionItem.svelte";

  let transactions: Transaction[] = [];
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    try {
      transactions = await getTransactions();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });
</script>

<div class="history">
  <header class="history-header">
    <h1>Activity</h1>
  </header>

  <div class="history-content">
    {#if loading}
      <div class="loading-state">
        <Loader2 size={22} class="spin" />
        <p>Loading transactions</p>
      </div>
    {:else if error}
      <div class="error-state">
        <div class="error-icon">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
        </div>
        <p class="error-message">{error}</p>
      </div>
    {:else if transactions.length === 0}
      <div class="empty-state">
        <div class="empty-icon">
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
          </svg>
        </div>
        <h3>No transactions yet</h3>
        <p>Your activity will appear here</p>
      </div>
    {:else}
      <div class="transaction-list">
        {#each transactions as tx}
          <TransactionItem
            txid={tx.txid}
            txType={tx.tx_type}
            amount={tx.amount}
            timestamp={tx.timestamp}
            address={tx.address}
            memo={tx.memo}
            status={tx.status}
          />
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .history {
    min-height: 100%;
    display: flex;
    flex-direction: column;
    animation: fadeIn var(--transition-normal) ease-out;
    background: var(--bg-primary);
  }

  .history-header {
    padding: var(--space-5);
    padding-bottom: var(--space-3);
  }

  .history-header h1 {
    font-size: var(--text-h2);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tight);
  }

  .history-content {
    flex: 1;
    padding: 0 var(--space-5) var(--space-5);
    padding-bottom: calc(var(--nav-height) + var(--space-4));
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-12) 0;
    gap: var(--space-3);
    color: var(--text-tertiary);
  }

  .loading-state :global(.spin) {
    animation: spin 1s linear infinite;
  }

  .loading-state p {
    font-size: var(--text-small);
    letter-spacing: 0.01em;
  }

  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-12) 0;
    gap: var(--space-3);
    text-align: center;
  }

  .error-icon {
    color: var(--text-tertiary);
    opacity: 0.5;
  }

  .error-message {
    color: var(--text-secondary);
    font-size: var(--text-small);
    max-width: 280px;
    line-height: var(--leading-relaxed);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-12) var(--space-4);
    text-align: center;
    gap: var(--space-3);
  }

  .empty-icon {
    color: var(--text-tertiary);
    opacity: 0.4;
    margin-bottom: var(--space-2);
  }

  .empty-state h3 {
    font-size: var(--text-body);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }

  .empty-state p {
    color: var(--text-tertiary);
    font-size: var(--text-small);
  }

  .transaction-list {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .transaction-list > :global(*:not(:last-child)) {
    border-bottom: 1px solid var(--divider);
  }
</style>
