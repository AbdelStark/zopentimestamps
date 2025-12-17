<script lang="ts">
  import { ArrowLeft, Plus, Download, Copy, CheckCircle, Loader2, Eye, EyeOff } from "lucide-svelte";
  import { wallet } from "../lib/stores/wallet";
  import { ui } from "../lib/stores/ui";
  import { generateSeed, initWallet, loadWallet } from "../lib/utils/tauri";
  import { copyToClipboard } from "../lib/utils/format";
  import Button from "../lib/components/Button.svelte";
  import Input from "../lib/components/Input.svelte";

  type Step = "welcome" | "choice" | "create" | "import" | "confirm" | "loading" | "complete";

  let currentStep: Step = "welcome";
  let seedPhrase = "";
  let inputSeed = "";
  let seedConfirmed = false;
  let showSeed = true;
  let loading = false;
  let error = "";

  async function handleCreateWallet() {
    currentStep = "loading";
    try {
      seedPhrase = await generateSeed();
      currentStep = "create";
    } catch (e) {
      error = String(e);
      currentStep = "choice";
      ui.showToast(`Failed to generate seed: ${e}`, "error");
    }
  }

  async function handleImportWallet() {
    currentStep = "import";
  }

  async function handleConfirmSeed() {
    currentStep = "confirm";
  }

  async function handleSeedConfirmed() {
    seedConfirmed = true;
    currentStep = "loading";
    try {
      const walletInfo = await initWallet(seedPhrase);
      wallet.setInfo({
        address: walletInfo.address,
        balance: walletInfo.balance,
        blockHeight: walletInfo.block_height,
      });
      currentStep = "complete";
    } catch (e) {
      error = String(e);
      currentStep = "confirm";
      ui.showToast(`Failed to create wallet: ${e}`, "error");
    }
  }

  async function handleImportConfirm() {
    if (inputSeed.split(" ").length !== 24) {
      ui.showToast("Please enter a valid 24-word seed phrase", "error");
      return;
    }

    currentStep = "loading";
    try {
      const walletInfo = await loadWallet(inputSeed);
      wallet.setInfo({
        address: walletInfo.address,
        balance: walletInfo.balance,
        blockHeight: walletInfo.block_height,
      });
      currentStep = "complete";
    } catch (e) {
      error = String(e);
      currentStep = "import";
      ui.showToast(`Failed to import wallet: ${e}`, "error");
    }
  }

  async function handleComplete() {
    ui.setOnboardingComplete();
  }

  async function handleCopySeed() {
    const success = await copyToClipboard(seedPhrase);
    if (success) {
      ui.showToast("Seed phrase copied!", "success");
    }
  }

  function goBack() {
    if (currentStep === "choice") {
      currentStep = "welcome";
    } else if (currentStep === "create" || currentStep === "import") {
      currentStep = "choice";
    } else if (currentStep === "confirm") {
      currentStep = "create";
    }
  }

  function handleSeedInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    inputSeed = target.value;
  }
</script>

<div class="onboarding">
  {#if currentStep === "welcome"}
    <!-- Welcome Screen -->
    <div class="onboarding-screen welcome animate-fade-in">
      <div class="welcome-content">
        <div class="logo">
          <span class="logo-text">Ikki</span>
        </div>
        <h1>Welcome to Ikki</h1>
        <p>A beautiful Zcash wallet for everyone. Private, secure, and easy to use.</p>
      </div>
      <div class="onboarding-actions">
        <Button variant="primary" size="lg" fullWidth onclick={() => (currentStep = "choice")}>
          Get Started
        </Button>
      </div>
    </div>

  {:else if currentStep === "choice"}
    <!-- Choice Screen -->
    <div class="onboarding-screen choice animate-fade-in">
      <header class="onboarding-header">
        <button class="back-button" onclick={goBack}>
          <ArrowLeft size={24} />
        </button>
        <h2>Set Up Wallet</h2>
        <div class="header-spacer"></div>
      </header>

      <div class="choice-content">
        <p class="choice-subtitle">How would you like to get started?</p>

        <div class="choice-options">
          <button class="choice-card" onclick={handleCreateWallet}>
            <div class="choice-icon create">
              <Plus size={24} />
            </div>
            <div class="choice-info">
              <span class="choice-title">Create New Wallet</span>
              <span class="choice-description">Generate a new wallet with a fresh seed phrase</span>
            </div>
          </button>

          <button class="choice-card" onclick={handleImportWallet}>
            <div class="choice-icon import">
              <Download size={24} />
            </div>
            <div class="choice-info">
              <span class="choice-title">Import Existing Wallet</span>
              <span class="choice-description">Restore your wallet using a seed phrase</span>
            </div>
          </button>
        </div>
      </div>
    </div>

  {:else if currentStep === "create"}
    <!-- Create Wallet - Show Seed -->
    <div class="onboarding-screen create animate-fade-in">
      <header class="onboarding-header">
        <button class="back-button" onclick={goBack}>
          <ArrowLeft size={24} />
        </button>
        <h2>Recovery Phrase</h2>
        <div class="header-spacer"></div>
      </header>

      <div class="create-content">
        <div class="warning-box">
          <p>Write down these 24 words in order and keep them safe. This is the only way to recover your wallet.</p>
        </div>

        <div class="seed-container">
          <div class="seed-header">
            <span>Your Recovery Phrase</span>
            <button class="toggle-visibility" onclick={() => (showSeed = !showSeed)}>
              {#if showSeed}
                <EyeOff size={18} />
              {:else}
                <Eye size={18} />
              {/if}
            </button>
          </div>
          <div class="seed-grid" class:blurred={!showSeed}>
            {#each seedPhrase.split(" ") as word, i}
              <div class="seed-word">
                <span class="word-number">{i + 1}</span>
                <span class="word-text">{word}</span>
              </div>
            {/each}
          </div>
        </div>

        <button class="copy-button" onclick={handleCopySeed}>
          <Copy size={16} />
          Copy to clipboard
        </button>
      </div>

      <div class="onboarding-actions">
        <Button variant="primary" size="lg" fullWidth onclick={handleConfirmSeed}>
          I've Written It Down
        </Button>
      </div>
    </div>

  {:else if currentStep === "import"}
    <!-- Import Wallet -->
    <div class="onboarding-screen import animate-fade-in">
      <header class="onboarding-header">
        <button class="back-button" onclick={goBack}>
          <ArrowLeft size={24} />
        </button>
        <h2>Import Wallet</h2>
        <div class="header-spacer"></div>
      </header>

      <div class="import-content">
        <p class="import-subtitle">Enter your 24-word recovery phrase to restore your wallet.</p>

        <div class="seed-input-container">
          <textarea
            class="seed-input"
            placeholder="Enter your 24-word seed phrase, separated by spaces..."
            value={inputSeed}
            oninput={handleSeedInput}
            rows={6}
          ></textarea>
          <span class="word-count">{inputSeed.split(" ").filter(w => w.trim()).length} / 24 words</span>
        </div>
      </div>

      <div class="onboarding-actions">
        <Button
          variant="primary"
          size="lg"
          fullWidth
          disabled={inputSeed.split(" ").filter(w => w.trim()).length !== 24}
          onclick={handleImportConfirm}
        >
          Import Wallet
        </Button>
      </div>
    </div>

  {:else if currentStep === "confirm"}
    <!-- Confirm Seed Backup -->
    <div class="onboarding-screen confirm animate-fade-in">
      <header class="onboarding-header">
        <button class="back-button" onclick={goBack}>
          <ArrowLeft size={24} />
        </button>
        <h2>Confirm Backup</h2>
        <div class="header-spacer"></div>
      </header>

      <div class="confirm-content">
        <div class="confirm-icon">
          <CheckCircle size={64} />
        </div>
        <h3>Have you securely stored your recovery phrase?</h3>
        <p>You will need these words to recover your wallet if you lose access to this device.</p>

        <div class="confirm-checklist">
          <label class="checkbox-item">
            <input type="checkbox" bind:checked={seedConfirmed} />
            <span>I understand that if I lose my recovery phrase, I will lose access to my funds forever.</span>
          </label>
        </div>
      </div>

      <div class="onboarding-actions">
        <Button
          variant="primary"
          size="lg"
          fullWidth
          disabled={!seedConfirmed}
          onclick={handleSeedConfirmed}
        >
          Create Wallet
        </Button>
      </div>
    </div>

  {:else if currentStep === "loading"}
    <!-- Loading -->
    <div class="onboarding-screen loading-screen animate-fade-in">
      <div class="loading-content">
        <Loader2 size={48} class="spin" />
        <h2>Setting up your wallet...</h2>
        <p>This may take a moment</p>
      </div>
    </div>

  {:else if currentStep === "complete"}
    <!-- Complete -->
    <div class="onboarding-screen complete animate-fade-in">
      <div class="complete-content">
        <div class="success-icon">
          <CheckCircle size={80} />
        </div>
        <h1>You're All Set!</h1>
        <p>Your wallet has been created successfully. You can now send and receive ZEC.</p>
      </div>
      <div class="onboarding-actions">
        <Button variant="primary" size="lg" fullWidth onclick={handleComplete}>
          Open Wallet
        </Button>
      </div>
    </div>
  {/if}
</div>

<style>
  .onboarding {
    min-height: 100vh;
    background: var(--bg-primary);
    display: flex;
    flex-direction: column;
  }

  .onboarding-screen {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--space-lg);
    max-width: var(--max-width);
    margin: 0 auto;
    width: 100%;
  }

  .onboarding-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-xl);
  }

  .back-button {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    border-radius: var(--radius-md);
    transition: background var(--transition-fast);
  }

  .back-button:hover {
    background: var(--bg-card);
  }

  .onboarding-header h2 {
    font-size: var(--text-h3);
    font-weight: var(--weight-semibold);
  }

  .header-spacer {
    width: 40px;
  }

  .onboarding-actions {
    margin-top: auto;
    padding-top: var(--space-xl);
  }

  /* Welcome */
  .welcome {
    justify-content: center;
    text-align: center;
  }

  .welcome-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-lg);
  }

  .logo-text {
    font-size: 4rem;
    font-weight: var(--weight-bold);
    background: var(--gradient-accent);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .welcome h1 {
    font-size: var(--text-h1);
  }

  .welcome p {
    font-size: var(--text-body);
    color: var(--text-secondary);
    max-width: 280px;
  }

  /* Choice */
  .choice-content {
    flex: 1;
  }

  .choice-subtitle {
    color: var(--text-secondary);
    margin-bottom: var(--space-xl);
  }

  .choice-options {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .choice-card {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-lg);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    cursor: pointer;
    text-align: left;
    transition: all var(--transition-fast);
  }

  .choice-card:hover {
    border-color: var(--accent);
    background: var(--bg-elevated);
  }

  .choice-icon {
    width: 48px;
    height: 48px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .choice-icon.create {
    background: var(--accent-dim);
    color: var(--accent);
  }

  .choice-icon.import {
    background: var(--info-dim);
    color: var(--info);
  }

  .choice-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .choice-title {
    font-size: var(--text-body);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .choice-description {
    font-size: var(--text-small);
    color: var(--text-secondary);
  }

  /* Create */
  .create-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .warning-box {
    padding: var(--space-md);
    background: var(--warning-dim);
    border-radius: var(--radius-md);
  }

  .warning-box p {
    color: var(--warning);
    font-size: var(--text-small);
    margin: 0;
  }

  .seed-container {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .seed-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-md);
    border-bottom: 1px solid var(--border);
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .toggle-visibility {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: var(--space-xs);
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }

  .toggle-visibility:hover {
    color: var(--text-primary);
    background: var(--bg-elevated);
  }

  .seed-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-sm);
    padding: var(--space-md);
    transition: filter var(--transition-fast);
  }

  .seed-grid.blurred {
    filter: blur(8px);
  }

  .seed-word {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm);
    background: var(--bg-primary);
    border-radius: var(--radius-sm);
  }

  .word-number {
    font-size: var(--text-caption);
    color: var(--text-tertiary);
    min-width: 18px;
  }

  .word-text {
    font-size: var(--text-small);
    font-family: var(--font-mono);
    color: var(--text-primary);
  }

  .copy-button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    padding: var(--space-sm);
    background: none;
    border: none;
    color: var(--accent);
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    cursor: pointer;
    transition: opacity var(--transition-fast);
  }

  .copy-button:hover {
    opacity: 0.8;
  }

  /* Import */
  .import-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .import-subtitle {
    color: var(--text-secondary);
  }

  .seed-input-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .seed-input {
    flex: 1;
    min-height: 150px;
    padding: var(--space-md);
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--text-small);
    line-height: 1.6;
    resize: none;
  }

  .seed-input::placeholder {
    color: var(--text-tertiary);
  }

  .seed-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .word-count {
    font-size: var(--text-caption);
    color: var(--text-tertiary);
    text-align: right;
  }

  /* Confirm */
  .confirm-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--space-lg);
    padding-top: var(--space-xl);
  }

  .confirm-icon {
    color: var(--accent);
  }

  .confirm-content h3 {
    font-size: var(--text-h3);
  }

  .confirm-content p {
    color: var(--text-secondary);
    max-width: 300px;
  }

  .confirm-checklist {
    width: 100%;
    text-align: left;
    margin-top: var(--space-lg);
  }

  .checkbox-item {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    cursor: pointer;
    padding: var(--space-md);
    background: var(--bg-card);
    border-radius: var(--radius-md);
  }

  .checkbox-item input {
    margin-top: 4px;
    accent-color: var(--accent);
  }

  .checkbox-item span {
    font-size: var(--text-small);
    color: var(--text-secondary);
    line-height: 1.5;
  }

  /* Loading */
  .loading-screen {
    justify-content: center;
    align-items: center;
  }

  .loading-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-lg);
    text-align: center;
  }

  .loading-content :global(.spin) {
    animation: spin 1s linear infinite;
    color: var(--accent);
  }

  .loading-content h2 {
    font-size: var(--text-h3);
  }

  .loading-content p {
    color: var(--text-secondary);
  }

  /* Complete */
  .complete {
    justify-content: center;
  }

  .complete-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--space-lg);
  }

  .success-icon {
    color: var(--success);
  }

  .complete h1 {
    font-size: var(--text-h1);
  }

  .complete p {
    color: var(--text-secondary);
    max-width: 280px;
  }
</style>
