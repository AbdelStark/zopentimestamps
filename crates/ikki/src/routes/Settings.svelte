<script lang="ts">
  import { RefreshCw, Eye, EyeOff, ExternalLink, Trash2, Loader2, ArrowLeft, Check, Users, ChevronRight } from "lucide-svelte";
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
      wallet.updateBalance(result.balance);
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
        balance: {
          total: walletInfo.balance.total,
          shielded: walletInfo.balance.shielded,
          transparent: walletInfo.balance.transparent,
        },
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
    <div class="reset-flow">
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
      <!-- Contacts Section -->
      <section class="settings-section">
        <h2 class="section-title">Contacts</h2>
        <div class="settings-card">
          <button class="setting-item clickable" onclick={() => ui.navigate("contacts")}>
            <div class="setting-info">
              <span class="setting-label">Manage Contacts</span>
              <span class="setting-description">Save addresses for quick sending</span>
            </div>
            <ChevronRight size={16} />
          </button>
        </div>
      </section>

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
    animation: fadeIn var(--duration-normal) var(--ease-out);
    background: var(--bg-primary);
  }

  .settings-header {
    padding: var(--space-5);
    padding-bottom: var(--space-4);
  }

  .settings-header h1 {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tight);
  }

  .settings-content {
    flex: 1;
    padding: 0 var(--space-5) var(--space-5);
    padding-bottom: calc(var(--nav-height) + var(--space-4));
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .section-title {
    font-size: var(--text-2xs);
    font-weight: var(--font-medium);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: var(--tracking-widest);
    padding-left: var(--space-1);
  }

  .settings-card {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    overflow: hidden;
    position: relative;
  }

  .settings-card::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--gradient-card);
    pointer-events: none;
  }

  .setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4);
    background: none;
    border: none;
    width: 100%;
    text-align: left;
    color: var(--text-tertiary);
    text-decoration: none;
    position: relative;
  }

  .setting-item.clickable {
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }

  .setting-item.clickable:hover {
    background: var(--bg-hover);
  }

  .setting-item.clickable:disabled {
    opacity: 0.4;
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
    gap: 3px;
  }

  .setting-label {
    font-size: var(--text-sm);
    color: var(--text-primary);
    letter-spacing: var(--tracking-normal);
  }

  .setting-description {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    letter-spacing: var(--tracking-wide);
  }

  .setting-value {
    font-size: var(--text-xs);
  }

  .setting-value.secondary {
    color: var(--text-secondary);
  }

  .setting-divider {
    height: 1px;
    background: var(--border);
    margin: 0 var(--space-4);
  }

  .network-badge {
    display: inline-flex;
    align-items: center;
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--text-2xs);
    font-weight: var(--font-medium);
    background: var(--bg-elevated);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    letter-spacing: var(--tracking-wide);
  }

  .sync-status {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    font-weight: var(--font-medium);
    color: var(--text-secondary);
    font-size: var(--text-xs);
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
    padding: var(--space-4);
    border-top: 1px solid var(--border);
  }

  .seed-warning {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    margin-bottom: var(--space-4);
    padding: var(--space-2) var(--space-4);
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    line-height: var(--leading-relaxed);
  }

  .seed-words {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .testnet-warning {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-4);
    background: var(--bg-card);
    color: var(--text-tertiary);
    border-radius: var(--radius-md);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    margin-top: auto;
    border: 1px solid var(--border);
    letter-spacing: var(--tracking-wide);
  }

  /* Reset Flow */
  .reset-flow {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    animation: fadeIn var(--duration-normal) var(--ease-out);
  }

  .flow-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
  }

  .flow-header h1 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-wide);
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
    transition:
      color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out);
  }

  .back-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .header-spacer {
    width: 40px;
  }

  .flow-content {
    flex: 1;
    padding: var(--space-5);
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
    font-size: var(--text-sm);
    color: var(--text-secondary);
    margin-bottom: var(--space-6);
    line-height: var(--leading-relaxed);
  }

  .flow-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    margin-top: auto;
    padding-top: var(--space-6);
  }

  .warning-box {
    display: flex;
    gap: var(--space-4);
    padding: var(--space-5);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--error);
    margin-bottom: var(--space-6);
  }

  .warning-text h3 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin-bottom: var(--space-2);
  }

  .warning-text p {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    line-height: var(--leading-relaxed);
    margin: 0;
  }

  /* Import Form */
  .import-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .input-label {
    display: block;
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    color: var(--text-tertiary);
    margin-bottom: var(--space-2);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
  }

  .seed-input-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .seed-input {
    width: 100%;
    min-height: 140px;
    padding: var(--space-4);
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.8;
    resize: none;
    transition:
      border-color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out);
  }

  .seed-input::placeholder {
    color: var(--text-disabled);
  }

  .seed-input:focus {
    outline: none;
    border-color: var(--border-focus);
    background: var(--bg-secondary);
  }

  .word-counter {
    font-size: var(--text-2xs);
    color: var(--text-tertiary);
    text-align: right;
    letter-spacing: var(--tracking-wide);
  }

  .word-counter.valid {
    color: var(--success);
  }

  .birthday-input-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .birthday-hint {
    font-size: var(--text-2xs);
    color: var(--text-tertiary);
    line-height: var(--leading-relaxed);
    margin: 0;
    letter-spacing: var(--tracking-wide);
  }

  /* Loading */
  .loading-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-5);
  }

  .loading-content :global(.spin) {
    animation: spin 1s linear infinite;
    color: var(--text-secondary);
  }

  .loading-content p {
    color: var(--text-secondary);
    font-size: var(--text-xs);
    margin: 0;
  }

  /* Complete */
  .complete-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    margin-bottom: var(--space-6);
  }

  .complete-content h2 {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    margin-bottom: var(--space-2);
    letter-spacing: var(--tracking-tight);
  }

  .complete-content p {
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .success-icon {
    width: 64px;
    height: 64px;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--receive-muted);
    border: 1px solid rgba(52, 211, 153, 0.2);
    color: var(--success);
    margin-bottom: var(--space-5);
    animation: scaleIn var(--duration-normal) var(--ease-spring);
  }
</style>
