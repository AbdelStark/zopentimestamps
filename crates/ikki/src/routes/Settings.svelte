<script lang="ts">
  import { RefreshCw, Eye, EyeOff, ExternalLink, Trash2, Loader2, ArrowLeft, Check } from "lucide-svelte";
  import { wallet, isSyncing } from "../lib/stores/wallet";
  import { ui } from "../lib/stores/ui";
  import { syncWallet, resetWallet, loadWallet } from "../lib/utils/tauri";
  import Button from "../lib/components/Button.svelte";
  import Input from "../lib/components/Input.svelte";

  let showSeed = false;
  let showResetFlow = false;
  let resetStep: "confirm" | "import" | "loading" | "complete" = "confirm";
  let inputSeed = "";
  let inputBirthday = "";
  let isResetting = false;
  let loadingMessage = "";

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

  function startResetFlow() {
    showResetFlow = true;
    resetStep = "confirm";
    inputSeed = "";
    inputBirthday = "";
  }

  function cancelReset() {
    showResetFlow = false;
    resetStep = "confirm";
    inputSeed = "";
    inputBirthday = "";
  }

  function confirmReset() {
    resetStep = "import";
  }

  function handleSeedInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    inputSeed = target.value;
  }

  function handleBirthdayInput(e: Event) {
    const target = e.target as HTMLInputElement;
    inputBirthday = target.value;
  }

  async function executeReset() {
    const words = inputSeed.trim().split(/\s+/).filter(w => w.length > 0);
    if (words.length !== 24) {
      ui.showToast("Please enter a valid 24-word seed phrase", "error");
      return;
    }

    // Parse birthday height if provided
    let birthdayHeight: number | undefined;
    if (inputBirthday.trim()) {
      const parsed = parseInt(inputBirthday.trim(), 10);
      if (isNaN(parsed) || parsed < 0) {
        ui.showToast("Invalid birthday height - must be a positive number", "error");
        return;
      }
      birthdayHeight = parsed;
    }

    isResetting = true;
    resetStep = "loading";
    loadingMessage = "Deleting wallet data...";

    try {
      // Reset wallet state and delete data
      await resetWallet();
      wallet.reset();

      loadingMessage = "Importing new wallet...";

      // Import new wallet
      const walletInfo = await loadWallet(words.join(" "), birthdayHeight);
      wallet.setInfo({
        address: walletInfo.address,
        balance: walletInfo.balance,
        blockHeight: walletInfo.block_height,
      });

      resetStep = "complete";
      ui.showToast("Wallet reset successfully", "success");
    } catch (e) {
      ui.showToast(`Reset failed: ${e}`, "error");
      resetStep = "import";
    } finally {
      isResetting = false;
    }
  }

  function finishReset() {
    showResetFlow = false;
    resetStep = "confirm";
    inputSeed = "";
    inputBirthday = "";
  }

  $: wordCount = inputSeed.trim().split(/\s+/).filter(w => w.length > 0).length;
</script>

<div class="settings">
  {#if showResetFlow}
    <!-- Reset Wallet Flow -->
    <div class="reset-flow animate-fade-in">
      {#if resetStep === "confirm"}
        <header class="flow-header">
          <button class="back-btn" onclick={cancelReset}>
            <ArrowLeft size={20} />
          </button>
          <h1>Reset Wallet</h1>
          <div class="header-spacer"></div>
        </header>

        <div class="flow-content">
          <div class="warning-box">
            <Trash2 size={24} />
            <div class="warning-text">
              <h3>Delete all wallet data?</h3>
              <p>This will permanently delete your current wallet data. You'll need to import a seed phrase to restore access.</p>
            </div>
          </div>

          <div class="flow-actions">
            <Button variant="danger" size="lg" fullWidth onclick={confirmReset}>
              Continue with Reset
            </Button>
            <Button variant="ghost" size="lg" fullWidth onclick={cancelReset}>
              Cancel
            </Button>
          </div>
        </div>

      {:else if resetStep === "import"}
        <header class="flow-header">
          <button class="back-btn" onclick={() => (resetStep = "confirm")}>
            <ArrowLeft size={20} />
          </button>
          <h1>Import Wallet</h1>
          <div class="header-spacer"></div>
        </header>

        <div class="flow-content">
          <p class="flow-subtitle">Enter your 24-word recovery phrase and birthday height to restore your wallet.</p>

          <div class="import-form">
            <div class="seed-input-container">
              <label class="input-label">Recovery Phrase</label>
              <textarea
                class="seed-input"
                placeholder="Enter your 24 words separated by spaces..."
                value={inputSeed}
                oninput={handleSeedInput}
                rows={5}
                spellcheck="false"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
              ></textarea>
              <span class="word-counter" class:valid={wordCount === 24}>
                {wordCount}/24 words
              </span>
            </div>

            <div class="birthday-input-container">
              <Input
                type="text"
                inputmode="numeric"
                label="Birthday Height (optional)"
                placeholder="e.g., 2000000"
                value={inputBirthday}
                oninput={handleBirthdayInput}
              />
              <p class="birthday-hint">
                The block height when your wallet was created. Speeds up syncing significantly.
              </p>
            </div>
          </div>

          <div class="flow-actions">
            <Button
              variant="primary"
              size="lg"
              fullWidth
              disabled={wordCount !== 24}
              onclick={executeReset}
            >
              Reset & Import
            </Button>
          </div>
        </div>

      {:else if resetStep === "loading"}
        <div class="flow-content center">
          <div class="loading-content">
            <Loader2 size={32} class="spin" />
            <p>{loadingMessage}</p>
          </div>
        </div>

      {:else if resetStep === "complete"}
        <div class="flow-content center">
          <div class="complete-content">
            <div class="success-icon">
              <Check size={32} strokeWidth={1.5} />
            </div>
            <h2>Wallet Reset</h2>
            <p>Your wallet has been successfully reset and imported.</p>
          </div>
          <div class="flow-actions">
            <Button variant="primary" size="lg" fullWidth onclick={finishReset}>
              Done
            </Button>
          </div>
        </div>
      {/if}
    </div>

  {:else}
    <!-- Normal Settings View -->
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

      <!-- Wallet Section -->
      <section class="settings-section">
        <h2 class="section-title">Wallet</h2>
        <div class="settings-card">
          <button class="setting-item clickable danger" onclick={startResetFlow}>
            <div class="setting-info">
              <span class="setting-label">Reset Wallet</span>
              <span class="setting-description">Delete data and import a new seed</span>
            </div>
            <Trash2 size={16} />
          </button>
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
  {/if}
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

  .setting-item.danger {
    color: var(--error);
  }

  .setting-item.danger .setting-label {
    color: var(--error);
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

  /* Reset Flow */
  .reset-flow {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .flow-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border);
  }

  .flow-header h1 {
    font-size: var(--text-body);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .back-btn {
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
    transition: all var(--transition-fast);
  }

  .back-btn:hover {
    color: var(--text-primary);
    background: var(--bg-card);
  }

  .header-spacer {
    width: 40px;
  }

  .flow-content {
    flex: 1;
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    max-width: var(--max-width);
    margin: 0 auto;
    width: 100%;
  }

  .flow-content.center {
    justify-content: center;
    align-items: center;
    text-align: center;
  }

  .flow-subtitle {
    font-size: var(--text-body);
    color: var(--text-secondary);
    margin-bottom: var(--space-xl);
    line-height: 1.5;
  }

  .flow-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    margin-top: auto;
    padding-top: var(--space-xl);
  }

  .warning-box {
    display: flex;
    gap: var(--space-md);
    padding: var(--space-lg);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--error);
    margin-bottom: var(--space-xl);
  }

  .warning-text h3 {
    font-size: var(--text-body);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    margin-bottom: var(--space-sm);
  }

  .warning-text p {
    font-size: var(--text-small);
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  /* Import Form */
  .import-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
  }

  .input-label {
    display: block;
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    margin-bottom: var(--space-sm);
  }

  .seed-input-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .seed-input {
    width: 100%;
    min-height: 140px;
    padding: var(--space-md);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--text-small);
    line-height: 1.8;
    resize: none;
  }

  .seed-input::placeholder {
    color: var(--text-tertiary);
  }

  .seed-input:focus {
    outline: none;
    border-color: var(--border-focus);
  }

  .word-counter {
    font-size: var(--text-caption);
    color: var(--text-tertiary);
    text-align: right;
  }

  .word-counter.valid {
    color: var(--success);
  }

  .birthday-input-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .birthday-hint {
    font-size: var(--text-caption);
    color: var(--text-tertiary);
    line-height: 1.5;
    margin: 0;
  }

  /* Loading */
  .loading-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-lg);
  }

  .loading-content :global(.spin) {
    animation: spin 1s linear infinite;
    color: var(--text-secondary);
  }

  .loading-content p {
    color: var(--text-secondary);
    margin: 0;
  }

  /* Complete */
  .complete-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    margin-bottom: var(--space-xl);
  }

  .complete-content h2 {
    font-size: var(--text-h3);
    font-weight: var(--weight-semibold);
    margin-bottom: var(--space-sm);
  }

  .complete-content p {
    color: var(--text-secondary);
    font-size: var(--text-body);
  }

  .success-icon {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    border: 1px solid var(--border);
    color: var(--success);
    margin-bottom: var(--space-lg);
  }
</style>
