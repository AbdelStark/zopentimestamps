<script lang="ts">
  import { ArrowLeft, Copy, Check, Share2, Shield } from "lucide-svelte";
  import { address } from "../lib/stores/wallet";
  import { ui } from "../lib/stores/ui";
  import { copyToClipboard } from "../lib/utils/format";
  import Button from "../lib/components/Button.svelte";

  let copied = false;

  async function handleCopy() {
    const success = await copyToClipboard($address);
    if (success) {
      copied = true;
      ui.showToast("Address copied", "success");
      setTimeout(() => (copied = false), 2000);
    } else {
      ui.showToast("Failed to copy", "error");
    }
  }

  async function handleShare() {
    if (navigator.share) {
      try {
        await navigator.share({
          title: "ZEC Address",
          text: $address,
        });
      } catch (e) {
        // User cancelled or error
      }
    } else {
      handleCopy();
    }
  }

  function handleBack() {
    ui.navigate("home");
  }
</script>

<div class="receive">
  <header class="receive-header">
    <button class="back-button" onclick={handleBack}>
      <ArrowLeft size={20} strokeWidth={2} />
    </button>
    <h1>Receive</h1>
    <div class="header-spacer"></div>
  </header>

  <div class="receive-content">
    <div class="qr-section">
      <div class="qr-container">
        <div class="qr-placeholder">
          <svg width="100" height="100" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="0.75" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="7" height="7"/>
            <rect x="14" y="3" width="7" height="7"/>
            <rect x="14" y="14" width="7" height="7"/>
            <rect x="3" y="14" width="7" height="7"/>
          </svg>
        </div>
      </div>
      <div class="address-type">
        <Shield size={12} strokeWidth={2.5} />
        <span>Shielded Address</span>
      </div>
    </div>

    <div class="address-section">
      <div class="address-box">
        <span class="address-text">{$address}</span>
      </div>
    </div>

    <div class="actions">
      <Button variant="primary" size="lg" fullWidth onclick={handleCopy}>
        {#if copied}
          <Check size={16} strokeWidth={2.5} />
          Copied
        {:else}
          <Copy size={16} strokeWidth={2} />
          Copy Address
        {/if}
      </Button>
      <Button variant="secondary" size="lg" fullWidth onclick={handleShare}>
        <Share2 size={16} strokeWidth={2} />
        Share
      </Button>
    </div>

    <div class="info-section">
      <p class="info-text">
        Only send ZEC to this address. Sending other assets may result in permanent loss.
      </p>
    </div>
  </div>
</div>

<style>
  .receive {
    min-height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
  }

  .receive-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
  }

  .back-button {
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
      background var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
    -webkit-tap-highlight-color: transparent;
  }

  .back-button:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .back-button:active {
    transform: scale(0.95);
  }

  .receive-header h1 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-wide);
  }

  .header-spacer {
    width: 40px;
  }

  .receive-content {
    flex: 1;
    padding: var(--space-6) var(--space-5);
    max-width: var(--max-width);
    margin: 0 auto;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    animation: fadeIn var(--duration-normal) var(--ease-out);
  }

  .qr-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-4);
  }

  .qr-container {
    width: 200px;
    height: 200px;
    background: var(--bg-card);
    border-radius: var(--radius-xl);
    border: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }

  .qr-container::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--gradient-card);
    pointer-events: none;
  }

  .qr-container::after {
    content: '';
    position: absolute;
    top: 0;
    left: 10%;
    right: 10%;
    height: 1px;
    background: linear-gradient(90deg,
      transparent,
      rgba(255, 255, 255, 0.06),
      transparent
    );
  }

  .qr-placeholder {
    color: var(--text-tertiary);
    opacity: 0.25;
    position: relative;
    z-index: 1;
  }

  .address-type {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-2xs);
    font-weight: var(--font-medium);
    color: var(--text-tertiary);
    letter-spacing: var(--tracking-wider);
    text-transform: uppercase;
  }

  .address-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .address-box {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    position: relative;
  }

  .address-box::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--gradient-card);
    pointer-events: none;
  }

  .address-text {
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    color: var(--text-secondary);
    word-break: break-all;
    line-height: 1.8;
    letter-spacing: var(--tracking-wide);
    position: relative;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .info-section {
    margin-top: auto;
    text-align: center;
    padding-top: var(--space-4);
  }

  .info-text {
    font-size: var(--text-2xs);
    color: var(--text-tertiary);
    line-height: var(--leading-relaxed);
    max-width: 280px;
    margin: 0 auto;
    letter-spacing: var(--tracking-wide);
  }
</style>
