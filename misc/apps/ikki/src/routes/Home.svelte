<script lang="ts">
  import { onMount } from "svelte";
  import { wallet, balance, address, isSyncing } from "../lib/stores/wallet";
  import { ui } from "../lib/stores/ui";
  import { syncWallet, getTransactions, type Transaction } from "../lib/utils/tauri";
  import AccountCard from "../lib/components/AccountCard.svelte";
  import ActionButton from "../lib/components/ActionButton.svelte";
  import TransactionItem from "../lib/components/TransactionItem.svelte";

  let recentTransactions: Transaction[] = [];
  let loading = true;

  onMount(async () => {
    try {
      recentTransactions = await getTransactions();
    } catch (e) {
      console.error("Failed to load transactions:", e);
    } finally {
      loading = false;
    }
  });

  async function handleSync() {
    try {
      wallet.setSyncing(true);
      const result = await syncWallet();
      wallet.updateBalance(result.balance);
      // Refresh transactions after sync
      recentTransactions = await getTransactions();
      ui.showToast("Wallet synced", "success");
    } catch (e) {
      ui.showToast(`Sync failed: ${e}`, "error");
    } finally {
      wallet.setSyncing(false);
    }
  }
</script>

<div class="home">
  <div class="home-content">
    <AccountCard
      balance={$balance}
      address={$address}
      syncing={$isSyncing}
    />

    <div class="actions">
      <ActionButton variant="send" onclick={() => ui.navigate("send")} />
      <ActionButton variant="receive" onclick={() => ui.navigate("receive")} />
    </div>

    <section class="recent-section">
      <div class="section-header">
        <h3>Recent Activity</h3>
        {#if recentTransactions.length > 0}
          <button class="see-all" onclick={() => ui.navigate("history")}>
            See all
          </button>
        {/if}
      </div>

      {#if loading}
        <div class="loading-state">
          <div class="skeleton-list">
            {#each [1, 2, 3] as _}
              <div class="skeleton-item">
                <div class="skeleton skeleton-icon"></div>
                <div class="skeleton-content">
                  <div class="skeleton skeleton-title"></div>
                  <div class="skeleton skeleton-subtitle"></div>
                </div>
                <div class="skeleton skeleton-amount"></div>
              </div>
            {/each}
          </div>
        </div>
      {:else if recentTransactions.length === 0}
        <div class="empty-state">
          <div class="empty-icon">
            <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
            </svg>
          </div>
          <p class="empty-title">No transactions yet</p>
          <p class="empty-subtitle">Your activity will appear here</p>
        </div>
      {:else}
        <div class="transaction-list">
          {#each recentTransactions.slice(0, 5) as tx}
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
      {/if}
    </section>
  </div>
</div>

<style>
  .home {
    min-height: 100%;
    padding: var(--space-5);
    padding-bottom: calc(var(--nav-height) + var(--space-4));
    animation: fadeIn var(--duration-normal) var(--ease-out);
  }

  .home-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    max-width: var(--max-width);
    margin: 0 auto;
  }

  .actions {
    display: flex;
    justify-content: center;
    gap: var(--space-10);
  }

  .recent-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-1);
  }

  .section-header h3 {
    font-size: var(--text-2xs);
    font-weight: var(--font-semibold);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: var(--tracking-widest);
  }

  .see-all {
    background: none;
    border: none;
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    cursor: pointer;
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    transition:
      color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out);
    letter-spacing: var(--tracking-wide);
  }

  .see-all:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
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

  .loading-state {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .skeleton-list {
    padding: var(--space-1);
  }

  .skeleton-item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3);
  }

  .skeleton {
    background: linear-gradient(
      90deg,
      var(--bg-elevated) 0%,
      var(--bg-hover) 50%,
      var(--bg-elevated) 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.8s ease-in-out infinite;
    border-radius: var(--radius-sm);
  }

  .skeleton-icon {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }

  .skeleton-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .skeleton-title {
    height: 12px;
    width: 64px;
  }

  .skeleton-subtitle {
    height: 10px;
    width: 88px;
  }

  .skeleton-amount {
    width: 56px;
    height: 12px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-12) var(--space-4);
    text-align: center;
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
  }

  .empty-icon {
    color: var(--text-tertiary);
    margin-bottom: var(--space-4);
    opacity: 0.3;
  }

  .empty-title {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    margin-bottom: var(--space-1);
  }

  .empty-subtitle {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }
</style>
