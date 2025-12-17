<script lang="ts">
  import { onMount } from "svelte";
  import { wallet, balance, address, isSyncing } from "../lib/stores/wallet";
  import { ui } from "../lib/stores/ui";
  import { syncWallet, getBalance } from "../lib/utils/tauri";
  import AccountCard from "../lib/components/AccountCard.svelte";
  import ActionButton from "../lib/components/ActionButton.svelte";
  import TransactionItem from "../lib/components/TransactionItem.svelte";

  // Mock recent transactions (until we implement transaction history)
  let recentTransactions: any[] = [];

  async function handleSync() {
    try {
      wallet.setSyncing(true);
      const result = await syncWallet();
      wallet.updateBalance({
        total: result.balance,
        shielded: result.balance,
        transparent: 0,
      });
      ui.showToast("Wallet synced!", "success");
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

      {#if recentTransactions.length === 0}
        <div class="empty-state">
          <div class="empty-icon">📭</div>
          <p class="empty-title">No transactions yet</p>
          <p class="empty-subtitle">Send or receive ZEC to see your activity</p>
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
    padding: var(--space-lg);
    animation: fadeIn var(--transition-normal) ease-out;
  }

  .home-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
    max-width: var(--max-width);
    margin: 0 auto;
  }

  .actions {
    display: flex;
    justify-content: center;
    gap: var(--space-xl);
  }

  .recent-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-header h3 {
    font-size: var(--text-body);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .see-all {
    background: none;
    border: none;
    color: var(--accent);
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    cursor: pointer;
    padding: var(--space-xs) var(--space-sm);
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
  }

  .see-all:hover {
    background: var(--accent-dim);
  }

  .transaction-list {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-2xl) var(--space-lg);
    text-align: center;
  }

  .empty-icon {
    font-size: 3rem;
    margin-bottom: var(--space-md);
  }

  .empty-title {
    font-size: var(--text-body);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    margin-bottom: var(--space-xs);
  }

  .empty-subtitle {
    font-size: var(--text-small);
    color: var(--text-secondary);
  }
</style>
