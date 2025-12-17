<script lang="ts">
  import { ArrowLeft, Copy, Shield, Share2 } from "lucide-svelte";
  import { address } from "../lib/stores/wallet";
  import { ui } from "../lib/stores/ui";
  import { copyToClipboard, truncateAddress } from "../lib/utils/format";
  import Button from "../lib/components/Button.svelte";

  async function handleCopy() {
    const success = await copyToClipboard($address);
    if (success) {
      ui.showToast("Address copied!", "success");
    } else {
      ui.showToast("Failed to copy", "error");
    }
  }

  async function handleShare() {
    if (navigator.share) {
      try {
        await navigator.share({
          title: "My Zcash Address",
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
      <ArrowLeft size={24} />
    </button>
    <h1>Receive ZEC</h1>
    <div class="header-spacer"></div>
  </header>

  <div class="receive-content">
    <div class="qr-section">
      <div class="qr-container">
        <!-- QR Code placeholder - would need a QR library -->
        <div class="qr-placeholder">
          <div class="qr-pattern"></div>
        </div>
      </div>
      <div class="shielded-badge">
        <Shield size={14} />
        <span>Shielded Address</span>
      </div>
    </div>

    <div class="address-section">
      <label class="address-label">Your Address</label>
      <div class="address-box">
        <span class="address-text">{$address}</span>
      </div>
    </div>

    <div class="actions">
      <Button variant="primary" size="lg" fullWidth onclick={handleCopy}>
        <Copy size={18} />
        Copy Address
      </Button>
      <Button variant="secondary" size="lg" fullWidth onclick={handleShare}>
        <Share2 size={18} />
        Share
      </Button>
    </div>

    <div class="info-section">
      <p class="info-text">
        Share this address to receive ZEC. Only send Zcash (ZEC) to this address.
      </p>
    </div>
  </div>
</div>

<style>
  .receive {
    min-height: 100%;
    display: flex;
    flex-direction: column;
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
    color: var(--text-primary);
    cursor: pointer;
    border-radius: var(--radius-md);
    transition: background var(--transition-fast);
  }

  .back-button:hover {
    background: var(--bg-card);
  }

  .receive-header h1 {
    font-size: var(--text-h3);
    font-weight: var(--weight-semibold);
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
    background: white;
    border-radius: var(--radius-lg);
    padding: var(--space-md);
    box-shadow: var(--shadow-md);
  }

  .qr-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(45deg, #f0f0f0 25%, transparent 25%),
      linear-gradient(-45deg, #f0f0f0 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #f0f0f0 75%),
      linear-gradient(-45deg, transparent 75%, #f0f0f0 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
    border-radius: var(--radius-sm);
  }

  .shielded-badge {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-md);
    background: rgba(191, 90, 242, 0.15);
    color: var(--shielded);
    border-radius: var(--radius-full);
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
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
  }
</style>
