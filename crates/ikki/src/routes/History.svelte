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
        <Loader2 size={24} class="spin" />
        <p>Loading transactions</p>
      </div>
    {:else if error}
      <div class="error-state">
        <div class="error-icon">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
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
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
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
    padding: var(--space-lg);
    padding-bottom: var(--space-md);
  }

  .history-header h1 {
    font-size: var(--text-h3);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .history-content {
    flex: 1;
    padding: 0 var(--space-lg) var(--space-lg);
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-3xl) 0;
    gap: var(--space-md);
    color: var(--text-secondary);
  }

  .loading-state :global(.spin) {
    animation: spin 1s linear infinite;
    color: var(--text-tertiary);
  }

  .loading-state p {
    font-size: var(--text-small);
  }

  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-3xl) 0;
    gap: var(--space-md);
    text-align: center;
  }

  .error-icon {
    color: var(--text-tertiary);
  }

  .error-message {
    color: var(--text-secondary);
    font-size: var(--text-small);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-3xl) 0;
    text-align: center;
    gap: var(--space-md);
  }

  .empty-icon {
    color: var(--text-tertiary);
    opacity: 0.5;
    margin-bottom: var(--space-sm);
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
</style>
