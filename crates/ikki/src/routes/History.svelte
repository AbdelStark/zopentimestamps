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
    <p class="header-subtitle">Your transaction history</p>
  </header>

  <div class="history-content">
    {#if loading}
      <div class="loading-state">
        <Loader2 size={32} class="spin" />
        <p>Loading transactions...</p>
      </div>
    {:else if error}
      <div class="error-state">
        <p class="error-icon">⚠️</p>
        <p class="error-message">{error}</p>
      </div>
    {:else if transactions.length === 0}
      <div class="empty-state">
        <div class="empty-icon">📭</div>
        <h3>No transactions yet</h3>
        <p>Send or receive ZEC to see your activity here</p>
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
  }

  .history-header {
    padding: var(--space-lg);
    padding-bottom: var(--space-md);
  }

  .history-header h1 {
    font-size: var(--text-h2);
    font-weight: var(--weight-bold);
    margin-bottom: var(--space-xs);
  }

  .header-subtitle {
    font-size: var(--text-small);
    color: var(--text-secondary);
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
    color: var(--accent);
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
    font-size: 3rem;
  }

  .error-message {
    color: var(--text-secondary);
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
    font-size: 4rem;
  }

  .empty-state h3 {
    font-size: var(--text-h3);
    font-weight: var(--weight-semibold);
  }

  .empty-state p {
    color: var(--text-secondary);
    font-size: var(--text-small);
  }

  .transaction-list {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }
</style>
