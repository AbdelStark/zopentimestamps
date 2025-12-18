<script lang="ts">
  import { ArrowLeft, Plus, Import, Copy, Check, Loader2, Eye, EyeOff } from "lucide-svelte";
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
  let inputBirthday = "";
  let seedConfirmed = false;
  let showSeed = true;
  let error = "";
  let loadingMessage = "Setting up your wallet...";

  async function handleCreateWallet() {
    currentStep = "loading";
    loadingMessage = "Generating seed phrase...";
    try {
      seedPhrase = await generateSeed();
      currentStep = "create";
    } catch (e) {
      error = String(e);
      currentStep = "choice";
      ui.showToast(`Failed to generate seed: ${e}`, "error");
    }
  }

  function handleImportWallet() {
    currentStep = "import";
  }

  function handleConfirmSeed() {
    currentStep = "confirm";
  }

  async function handleSeedConfirmed() {
    seedConfirmed = true;
    currentStep = "loading";
    loadingMessage = "Creating wallet...";
    try {
      // New wallet - use current block height as birthday
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

    currentStep = "loading";
    loadingMessage = "Importing wallet...";
    try {
      const walletInfo = await loadWallet(words.join(" "), birthdayHeight);
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

  function handleComplete() {
    ui.setOnboardingComplete();
  }

  async function handleCopySeed() {
    const success = await copyToClipboard(seedPhrase);
    if (success) {
      ui.showToast("Seed phrase copied", "success");
    }
  }

  function goBack() {
    if (currentStep === "choice") currentStep = "welcome";
    else if (currentStep === "create" || currentStep === "import") currentStep = "choice";
    else if (currentStep === "confirm") currentStep = "create";
  }

  function handleSeedInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    inputSeed = target.value;
  }

  function handleBirthdayInput(e: Event) {
    const target = e.target as HTMLInputElement;
    inputBirthday = target.value;
  }

  $: wordCount = inputSeed.trim().split(/\s+/).filter(w => w.length > 0).length;
</script>

<div class="onboarding">
  {#if currentStep === "welcome"}
    <div class="screen welcome">
      <div class="welcome-content">
        <div class="logo">ikki</div>
        <p class="tagline">Private digital assets</p>
      </div>
      <div class="screen-actions">
        <Button variant="primary" size="lg" fullWidth onclick={() => (currentStep = "choice")}>
          Get Started
        </Button>
      </div>
    </div>

  {:else if currentStep === "choice"}
    <div class="screen">
      <header class="screen-header">
        <button class="back-btn" onclick={goBack}>
          <ArrowLeft size={20} />
        </button>
      </header>

      <div class="screen-content">
        <h1>Set up wallet</h1>
        <p class="subtitle">Choose how you'd like to get started</p>

        <div class="choices">
          <button class="choice-card" onclick={handleCreateWallet}>
            <div class="choice-icon">
              <Plus size={20} strokeWidth={1.5} />
            </div>
            <div class="choice-text">
              <span class="choice-title">Create new wallet</span>
              <span class="choice-desc">Generate a new recovery phrase</span>
            </div>
          </button>

          <button class="choice-card" onclick={handleImportWallet}>
            <div class="choice-icon">
              <Import size={20} strokeWidth={1.5} />
            </div>
            <div class="choice-text">
              <span class="choice-title">Import existing wallet</span>
              <span class="choice-desc">Use your 24-word recovery phrase</span>
            </div>
          </button>
        </div>
      </div>
    </div>

  {:else if currentStep === "create"}
    <div class="screen">
      <header class="screen-header">
        <button class="back-btn" onclick={goBack}>
          <ArrowLeft size={20} />
        </button>
      </header>

      <div class="screen-content">
        <h1>Recovery phrase</h1>
        <p class="subtitle">Write down these 24 words in order. Keep them secure and private.</p>

        <div class="seed-container">
          <div class="seed-header">
            <button class="visibility-toggle" onclick={() => (showSeed = !showSeed)}>
              {#if showSeed}
                <EyeOff size={16} />
              {:else}
                <Eye size={16} />
              {/if}
            </button>
          </div>
          <div class="seed-grid" class:blurred={!showSeed}>
            {#each seedPhrase.split(" ") as word, i}
              <div class="seed-word">
                <span class="word-num">{i + 1}</span>
                <span class="word-text">{word}</span>
              </div>
            {/each}
          </div>
        </div>

        <button class="copy-link" onclick={handleCopySeed}>
          <Copy size={14} />
          Copy to clipboard
        </button>
      </div>

      <div class="screen-actions">
        <Button variant="primary" size="lg" fullWidth onclick={handleConfirmSeed}>
          I've saved my recovery phrase
        </Button>
      </div>
    </div>

  {:else if currentStep === "import"}
    <div class="screen">
      <header class="screen-header">
        <button class="back-btn" onclick={goBack}>
          <ArrowLeft size={20} />
        </button>
      </header>

      <div class="screen-content">
        <h1>Import wallet</h1>
        <p class="subtitle">Enter your 24-word recovery phrase and optional birthday height to restore your wallet.</p>

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
              The block height when your wallet was created. Using the correct birthday speeds up syncing significantly.
            </p>
          </div>
        </div>
      </div>

      <div class="screen-actions">
        <Button
          variant="primary"
          size="lg"
          fullWidth
          disabled={wordCount !== 24}
          onclick={handleImportConfirm}
        >
          Import wallet
        </Button>
      </div>
    </div>

  {:else if currentStep === "confirm"}
    <div class="screen">
      <header class="screen-header">
        <button class="back-btn" onclick={goBack}>
          <ArrowLeft size={20} />
        </button>
      </header>

      <div class="screen-content center">
        <div class="confirm-icon">
          <Check size={32} strokeWidth={1.5} />
        </div>
        <h1>Confirm backup</h1>
        <p class="subtitle">Have you securely stored your recovery phrase?</p>

        <label class="checkbox-row">
          <input type="checkbox" bind:checked={seedConfirmed} />
          <span>I understand that losing my recovery phrase means losing access to my funds.</span>
        </label>
      </div>

      <div class="screen-actions">
        <Button
          variant="primary"
          size="lg"
          fullWidth
          disabled={!seedConfirmed}
          onclick={handleSeedConfirmed}
        >
          Create wallet
        </Button>
      </div>
    </div>

  {:else if currentStep === "loading"}
    <div class="screen center">
      <div class="loading-content">
        <Loader2 size={32} class="spin" />
        <p>{loadingMessage}</p>
      </div>
    </div>

  {:else if currentStep === "complete"}
    <div class="screen center">
      <div class="complete-content">
        <div class="success-icon">
          <Check size={32} strokeWidth={1.5} />
        </div>
        <h1>You're all set</h1>
        <p class="subtitle">Your wallet is ready to use.</p>
      </div>
      <div class="screen-actions">
        <Button variant="primary" size="lg" fullWidth onclick={handleComplete}>
          Open wallet
        </Button>
      </div>
    </div>
  {/if}
</div>

<style>
  .onboarding {
    min-height: 100vh;
    background: var(--bg-primary);
  }

  .screen {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    padding: var(--space-5);
    max-width: var(--max-width);
    margin: 0 auto;
    animation: fadeIn var(--duration-normal) var(--ease-out);
  }

  .screen.center {
    justify-content: center;
    align-items: center;
    text-align: center;
  }

  .screen-header {
    margin-bottom: var(--space-6);
  }

  .back-btn {
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
    margin-left: -8px;
    transition:
      background var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }

  .back-btn:hover {
    background: var(--bg-hover);
  }

  .back-btn:active {
    transform: scale(0.95);
  }

  .screen-content {
    flex: 1;
  }

  .screen-content.center {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
  }

  .screen-content h1 {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
    margin-bottom: var(--space-2);
    letter-spacing: var(--tracking-tight);
  }

  .subtitle {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    margin-bottom: var(--space-6);
    line-height: var(--leading-relaxed);
  }

  .screen-actions {
    padding-top: var(--space-6);
  }

  /* Welcome */
  .welcome {
    justify-content: center;
  }

  .welcome-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }

  .logo {
    font-size: 3rem;
    font-weight: var(--font-bold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tighter);
    margin-bottom: var(--space-2);
  }

  .tagline {
    font-size: var(--text-sm);
    color: var(--text-tertiary);
    margin: 0;
    letter-spacing: var(--tracking-wide);
  }

  /* Choices */
  .choices {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .choice-card {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-5);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    cursor: pointer;
    text-align: left;
    position: relative;
    transition:
      background var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }

  .choice-card::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--gradient-card);
    pointer-events: none;
  }

  .choice-card:hover {
    background: var(--bg-elevated);
    border-color: var(--border-emphasis);
  }

  .choice-card:active {
    transform: scale(0.99);
  }

  .choice-icon {
    width: 44px;
    height: 44px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-primary);
    color: var(--text-secondary);
    flex-shrink: 0;
    position: relative;
  }

  .choice-text {
    display: flex;
    flex-direction: column;
    gap: 4px;
    position: relative;
  }

  .choice-title {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    letter-spacing: var(--tracking-normal);
  }

  .choice-desc {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    letter-spacing: var(--tracking-wide);
  }

  /* Seed Display */
  .seed-container {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    margin-bottom: var(--space-4);
    position: relative;
  }

  .seed-container::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--gradient-card);
    pointer-events: none;
  }

  .seed-header {
    display: flex;
    justify-content: flex-end;
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--border);
    position: relative;
  }

  .visibility-toggle {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: var(--space-1);
    border-radius: var(--radius-sm);
    transition:
      color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out);
  }

  .visibility-toggle:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .seed-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-2);
    padding: var(--space-4);
    position: relative;
    transition: filter var(--duration-fast) var(--ease-out);
  }

  .seed-grid.blurred {
    filter: blur(6px);
    user-select: none;
  }

  .seed-word {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2);
    background: var(--bg-primary);
    border-radius: var(--radius-sm);
  }

  .word-num {
    font-size: var(--text-2xs);
    color: var(--text-tertiary);
    min-width: 16px;
    font-family: var(--font-mono);
  }

  .word-text {
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    color: var(--text-primary);
  }

  .copy-link {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-2);
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: var(--text-xs);
    cursor: pointer;
    transition: color var(--duration-fast) var(--ease-out);
    width: 100%;
    letter-spacing: var(--tracking-wide);
  }

  .copy-link:hover {
    color: var(--text-primary);
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

  /* Confirm */
  .confirm-icon {
    width: 64px;
    height: 64px;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    border: 1px solid var(--border);
    color: var(--text-primary);
    margin-bottom: var(--space-5);
  }

  .checkbox-row {
    display: flex;
    align-items: flex-start;
    gap: var(--space-4);
    padding: var(--space-4);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    margin-top: var(--space-5);
  }

  .checkbox-row input {
    margin-top: 2px;
    accent-color: var(--text-primary);
  }

  .checkbox-row span {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    line-height: var(--leading-relaxed);
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
    letter-spacing: var(--tracking-wide);
  }

  /* Complete */
  .complete-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    margin-bottom: var(--space-6);
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

  .complete-content h1 {
    margin-bottom: var(--space-2);
  }

  .complete-content .subtitle {
    margin-bottom: 0;
  }
</style>
