<script lang="ts">
  import { onMount } from "svelte";
  import { Loader2 } from "lucide-svelte";
  import { getTransactions, type Transaction } from "../lib/utils/tauri";
  import TransactionItem from "../lib/components/TransactionItem.svelte";

  let transactions: Transaction[] = [];
  let loading = true;
  let error: string | null = null;

  interface GroupedTransactions {
    label: string;
    transactions: Transaction[];
  }

  function groupTransactionsByDate(txs: Transaction[]): GroupedTransactions[] {
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);
    const thisWeek = new Date(today);
    thisWeek.setDate(thisWeek.getDate() - 7);
    const thisMonth = new Date(today);
    thisMonth.setDate(thisMonth.getDate() - 30);

    const groups: { [key: string]: Transaction[] } = {
      Today: [],
      Yesterday: [],
      "This Week": [],
      "This Month": [],
      Earlier: [],
    };

    for (const tx of txs) {
      const txDate = new Date(tx.timestamp * 1000);
      if (txDate >= today) {
        groups["Today"].push(tx);
      } else if (txDate >= yesterday) {
        groups["Yesterday"].push(tx);
      } else if (txDate >= thisWeek) {
        groups["This Week"].push(tx);
      } else if (txDate >= thisMonth) {
        groups["This Month"].push(tx);
      } else {
        groups["Earlier"].push(tx);
      }
    }

    return Object.entries(groups)
      .filter(([_, txs]) => txs.length > 0)
      .map(([label, transactions]) => ({ label, transactions }));
  }

  $: groupedTransactions = groupTransactionsByDate(transactions);

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
    {#if transactions.length > 0}
      <span class="tx-count">{transactions.length}</span>
    {/if}
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
      <div class="transaction-groups">
        {#each groupedTransactions as group}
          <div class="transaction-group">
            <div class="group-header">
              <span class="group-label">{group.label}</span>
              <span class="group-count">{group.transactions.length}</span>
            </div>
            <div class="transaction-list">
              {#each group.transactions as tx}
                <TransactionItem
                  txid={tx.txid}
                  txType={tx.tx_type}
                  amount={tx.amount}
                  timestamp={tx.timestamp}
                  address={tx.address}
                  memo={tx.memo}
                  status={tx.status}
                  confirmations={tx.confirmations}
                />
              {/each}
            </div>
          </div>
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
    animation: fadeIn var(--duration-normal) var(--ease-out);
    background: var(--bg-primary);
  }

  .history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-5);
    padding-bottom: var(--space-3);
  }

  .history-header h1 {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tight);
  }

  .tx-count {
    font-size: var(--text-2xs);
    font-weight: var(--font-medium);
    color: var(--text-tertiary);
    background: var(--bg-elevated);
    padding: 4px 10px;
    border-radius: var(--radius-full);
    border: 1px solid var(--border-subtle);
    letter-spacing: var(--tracking-wide);
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
    font-size: var(--text-xs);
    letter-spacing: var(--tracking-wide);
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
    font-size: var(--text-xs);
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
    opacity: 0.3;
    margin-bottom: var(--space-2);
  }

  .empty-state h3 {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
  }

  .empty-state p {
    color: var(--text-tertiary);
    font-size: var(--text-xs);
  }

  .transaction-groups {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .transaction-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-1);
  }

  .group-label {
    font-size: var(--text-2xs);
    font-weight: var(--font-semibold);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: var(--tracking-widest);
  }

  .group-count {
    font-size: var(--text-2xs);
    color: var(--text-tertiary);
    font-weight: var(--font-medium);
    letter-spacing: var(--tracking-wide);
  }

  .transaction-list {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    overflow: hidden;
    position: relative;
  }

  .transaction-list::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--gradient-card);
    pointer-events: none;
  }

  .transaction-list::after {
    content: '';
    position: absolute;
    top: 0;
    left: 10%;
    right: 10%;
    height: 1px;
    background: linear-gradient(90deg,
      transparent,
      rgba(255, 255, 255, 0.04),
      transparent
    );
  }

  .transaction-list > :global(*:not(:last-child)) {
    border-bottom: 1px solid var(--divider);
  }
</style>
