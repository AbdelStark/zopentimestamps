<script lang="ts">
  import { RefreshCw, Eye, EyeOff, ExternalLink, ChevronRight } from "lucide-svelte";
  import { wallet, isSyncing } from "../lib/stores/wallet";
  import { ui } from "../lib/stores/ui";
  import { syncWallet } from "../lib/utils/tauri";
  import Button from "../lib/components/Button.svelte";

  let showSeed = false;
  let seedPhrase = ""; // Would be loaded from secure storage

  async function handleSync() {
    try {
      wallet.setSyncing(true);
      const result = await syncWallet();
      wallet.updateBalance({
        total: result.balance,
        shielded: result.balance,
        transparent: 0,
      });
      ui.showToast("Wallet synced", "success");
    } catch (e) {
      ui.showToast(`Sync failed: ${e}`, "error");
    } finally {
      wallet.setSyncing(false);
    }
  }

  function toggleSeed() {
    showSeed = !showSeed;
  }
</script>

<div class="settings">
  <header class="settings-header">
    <h1>Settings</h1>
  </header>

  <div class="settings-content">
    <!-- Network Section -->
    <section class="settings-section">
      <h2 class="section-title">Network</h2>
      <div class="settings-card">
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-label">Network</span>
            <span class="setting-value">
              <span class="network-badge">Testnet</span>
            </span>
          </div>
        </div>
        <div class="setting-divider"></div>
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-label">Server</span>
            <span class="setting-value secondary">testnet.zec.rocks:443</span>
          </div>
        </div>
      </div>
    </section>

    <!-- Sync Section -->
    <section class="settings-section">
      <h2 class="section-title">Sync</h2>
      <div class="settings-card">
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-label">Status</span>
            <span class="setting-value">
              {#if $isSyncing}
                <span class="sync-status syncing">
                  <RefreshCw size={12} class="spin" />
                  Syncing
                </span>
              {:else}
                <span class="sync-status">Synced</span>
              {/if}
            </span>
          </div>
        </div>
        <div class="setting-divider"></div>
        <button class="setting-item clickable" onclick={handleSync} disabled={$isSyncing}>
          <div class="setting-info">
            <span class="setting-label">Sync Now</span>
          </div>
          <RefreshCw size={16} class={$isSyncing ? "spin" : ""} />
        </button>
      </div>
    </section>

    <!-- Security Section -->
    <section class="settings-section">
      <h2 class="section-title">Security</h2>
      <div class="settings-card">
        <button class="setting-item clickable" onclick={toggleSeed}>
          <div class="setting-info">
            <span class="setting-label">Recovery Phrase</span>
            <span class="setting-description">View your 24-word seed phrase</span>
          </div>
          {#if showSeed}
            <EyeOff size={16} />
          {:else}
            <Eye size={16} />
          {/if}
        </button>
        {#if showSeed}
          <div class="seed-display">
            <p class="seed-warning">
              Never share your recovery phrase. Anyone with these words can access your funds.
            </p>
            <div class="seed-words">
              <span class="seed-placeholder">Seed phrase display not implemented</span>
            </div>
          </div>
        {/if}
      </div>
    </section>

    <!-- About Section -->
    <section class="settings-section">
      <h2 class="section-title">About</h2>
      <div class="settings-card">
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-label">Version</span>
            <span class="setting-value secondary">0.1.1</span>
          </div>
        </div>
        <div class="setting-divider"></div>
        <a href="https://github.com/AbdelStark/zopentimestamps" target="_blank" rel="noopener" class="setting-item clickable">
          <div class="setting-info">
            <span class="setting-label">Source Code</span>
          </div>
          <ExternalLink size={16} />
        </a>
      </div>
    </section>

    <!-- Warning -->
    <div class="testnet-warning">
      <span>Testnet Mode</span>
    </div>
  </div>
</div>

<style>
  .settings {
    min-height: 100%;
    display: flex;
    flex-direction: column;
    animation: fadeIn var(--transition-normal) ease-out;
    background: var(--bg-primary);
  }

  .settings-header {
    padding: var(--space-lg);
    padding-bottom: var(--space-md);
  }

  .settings-header h1 {
    font-size: var(--text-h3);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .settings-content {
    flex: 1;
    padding: 0 var(--space-lg) var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .section-title {
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    padding-left: var(--space-xs);
  }

  .settings-card {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md);
    background: none;
    border: none;
    width: 100%;
    text-align: left;
    color: var(--text-tertiary);
    text-decoration: none;
  }

  .setting-item.clickable {
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .setting-item.clickable:hover {
    background: var(--bg-elevated);
  }

  .setting-item.clickable:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .setting-label {
    font-size: var(--text-body);
    color: var(--text-primary);
  }

  .setting-description {
    font-size: var(--text-small);
    color: var(--text-secondary);
  }

  .setting-value {
    font-size: var(--text-small);
  }

  .setting-value.secondary {
    color: var(--text-secondary);
  }

  .setting-divider {
    height: 1px;
    background: var(--border);
    margin: 0 var(--space-md);
  }

  .network-badge {
    display: inline-flex;
    align-items: center;
    padding: var(--space-xs) var(--space-sm);
    border-radius: var(--radius-sm);
    font-size: var(--text-caption);
    font-weight: var(--weight-medium);
    background: var(--bg-elevated);
    color: var(--text-secondary);
    border: 1px solid var(--border);
  }

  .sync-status {
    display: inline-flex;
    align-items: center;
    gap: var(--space-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    font-size: var(--text-small);
  }

  .sync-status.syncing {
    color: var(--text-tertiary);
  }

  .sync-status :global(.spin) {
    animation: spin 1s linear infinite;
  }

  .setting-item :global(.spin) {
    animation: spin 1s linear infinite;
  }

  .seed-display {
    padding: var(--space-md);
    border-top: 1px solid var(--border);
  }

  .seed-warning {
    font-size: var(--text-small);
    color: var(--text-secondary);
    margin-bottom: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .seed-words {
    font-family: var(--font-mono);
    font-size: var(--text-small);
    color: var(--text-secondary);
  }

  .testnet-warning {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    padding: var(--space-md);
    background: var(--bg-card);
    color: var(--text-tertiary);
    border-radius: var(--radius-md);
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    margin-top: auto;
    border: 1px solid var(--border);
    letter-spacing: 0.02em;
  }
</style>
