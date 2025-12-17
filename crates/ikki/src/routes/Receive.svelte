<script lang="ts">
  import { ArrowLeft, Copy, Share2 } from "lucide-svelte";
  import { address } from "../lib/stores/wallet";
  import { ui } from "../lib/stores/ui";
  import { copyToClipboard } from "../lib/utils/format";
  import Button from "../lib/components/Button.svelte";

  async function handleCopy() {
    const success = await copyToClipboard($address);
    if (success) {
      ui.showToast("Address copied", "success");
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
      <ArrowLeft size={20} />
    </button>
    <h1>Receive</h1>
    <div class="header-spacer"></div>
  </header>

  <div class="receive-content">
    <div class="qr-section">
      <div class="qr-container">
        <!-- QR Code placeholder - would need a QR library -->
        <div class="qr-placeholder">
          <svg width="120" height="120" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="7" height="7"/>
            <rect x="14" y="3" width="7" height="7"/>
            <rect x="14" y="14" width="7" height="7"/>
            <rect x="3" y="14" width="7" height="7"/>
          </svg>
        </div>
      </div>
      <span class="address-type">Shielded Address</span>
    </div>

    <div class="address-section">
      <label class="address-label">Your Address</label>
      <div class="address-box">
        <span class="address-text">{$address}</span>
      </div>
    </div>

    <div class="actions">
      <Button variant="primary" size="lg" fullWidth onclick={handleCopy}>
        <Copy size={16} />
        Copy Address
      </Button>
      <Button variant="secondary" size="lg" fullWidth onclick={handleShare}>
        <Share2 size={16} />
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
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border);
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
    transition: all var(--transition-fast);
  }

  .back-button:hover {
    color: var(--text-primary);
    background: var(--bg-card);
  }

  .receive-header h1 {
    font-size: var(--text-body);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    letter-spacing: 0.02em;
  }

  .header-spacer {
    width: 40px;
  }

  .receive-content {
    flex: 1;
    padding: var(--space-xl) var(--space-lg);
    max-width: var(--max-width);
    margin: 0 auto;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
    animation: fadeIn var(--transition-normal) ease-out;
  }

  .qr-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-md);
  }

  .qr-container {
    width: 200px;
    height: 200px;
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    padding: var(--space-lg);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .qr-placeholder {
    color: var(--text-tertiary);
    opacity: 0.5;
  }

  .address-type {
    font-size: var(--text-small);
    color: var(--text-tertiary);
    letter-spacing: 0.02em;
  }

  .address-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .address-label {
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .address-box {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--space-md);
  }

  .address-text {
    font-family: var(--font-mono);
    font-size: var(--text-small);
    color: var(--text-primary);
    word-break: break-all;
    line-height: 1.6;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .info-section {
    margin-top: auto;
    text-align: center;
  }

  .info-text {
    font-size: var(--text-small);
    color: var(--text-tertiary);
    line-height: 1.5;
  }
</style>
