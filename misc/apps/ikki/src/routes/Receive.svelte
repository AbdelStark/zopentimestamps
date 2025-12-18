<script lang="ts">
  import { onMount } from "svelte";
  import { ArrowLeft, Copy, Check, Share2, Shield, RefreshCw } from "lucide-svelte";
  import QRCode from "qrcode";
  import { address, wallet } from "../lib/stores/wallet";
  import { ui } from "../lib/stores/ui";
  import { copyToClipboard } from "../lib/utils/format";
  import { getNewAddress } from "../lib/utils/tauri";
  import Button from "../lib/components/Button.svelte";

  let copied = false;
  let generating = false;
  let qrCodeSvg = "";
  let qrLoading = true;

  async function generateQR(addr: string) {
    if (!addr) return;
    qrLoading = true;
    try {
      qrCodeSvg = await QRCode.toString(addr, {
        type: "svg",
        width: 180,
        margin: 0,
        color: {
          dark: "#ffffff",
          light: "#00000000",
        },
        errorCorrectionLevel: "M",
      });
    } catch (e) {
      console.error("Failed to generate QR code:", e);
      qrCodeSvg = "";
    } finally {
      qrLoading = false;
    }
  }

  // Generate QR code when address changes
  $: generateQR($address);

  onMount(() => {
    generateQR($address);
  });

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

  async function handleNewAddress() {
    if (generating) return;
    generating = true;
    try {
      const newAddress = await getNewAddress();
      wallet.setAddress(newAddress);
      ui.showToast("New address generated", "success");
    } catch (e) {
      ui.showToast("Failed to generate address", "error");
    } finally {
      generating = false;
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
        {#if qrLoading}
          <div class="qr-loading">
            <div class="qr-skeleton"></div>
          </div>
        {:else if qrCodeSvg}
          <div class="qr-code">
            {@html qrCodeSvg}
          </div>
        {:else}
          <div class="qr-placeholder">
            <svg width="80" height="80" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="0.75" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="3" width="7" height="7"/>
              <rect x="14" y="3" width="7" height="7"/>
              <rect x="14" y="14" width="7" height="7"/>
              <rect x="3" y="14" width="7" height="7"/>
            </svg>
          </div>
        {/if}
        <div class="qr-corners">
          <div class="corner corner-tl"></div>
          <div class="corner corner-tr"></div>
          <div class="corner corner-bl"></div>
          <div class="corner corner-br"></div>
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
      <div class="actions-row">
        <Button variant="secondary" size="lg" fullWidth onclick={handleShare}>
          <Share2 size={16} strokeWidth={2} />
          Share
        </Button>
        <Button variant="secondary" size="lg" fullWidth onclick={handleNewAddress} disabled={generating}>
          <RefreshCw size={16} strokeWidth={2} class={generating ? "spinning" : ""} />
          {generating ? "..." : "New"}
        </Button>
      </div>
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
    width: 220px;
    height: 220px;
    background: var(--bg-card);
    border-radius: var(--radius-xl);
    border: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  .qr-container::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--gradient-card);
    pointer-events: none;
  }

  .qr-corners {
    position: absolute;
    inset: 12px;
    pointer-events: none;
  }

  .corner {
    position: absolute;
    width: 16px;
    height: 16px;
    border-color: var(--border-emphasis);
    border-style: solid;
    border-width: 0;
  }

  .corner-tl {
    top: 0;
    left: 0;
    border-top-width: 2px;
    border-left-width: 2px;
    border-top-left-radius: 4px;
  }

  .corner-tr {
    top: 0;
    right: 0;
    border-top-width: 2px;
    border-right-width: 2px;
    border-top-right-radius: 4px;
  }

  .corner-bl {
    bottom: 0;
    left: 0;
    border-bottom-width: 2px;
    border-left-width: 2px;
    border-bottom-left-radius: 4px;
  }

  .corner-br {
    bottom: 0;
    right: 0;
    border-bottom-width: 2px;
    border-right-width: 2px;
    border-bottom-right-radius: 4px;
  }

  .qr-code {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    animation: scaleIn var(--duration-normal) var(--ease-out);
  }

  .qr-code :global(svg) {
    display: block;
  }

  .qr-loading {
    position: relative;
    z-index: 1;
  }

  .qr-skeleton {
    width: 160px;
    height: 160px;
    background: linear-gradient(
      90deg,
      var(--bg-elevated) 0%,
      var(--bg-hover) 50%,
      var(--bg-elevated) 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s ease-in-out infinite;
    border-radius: var(--radius-md);
  }

  .qr-placeholder {
    color: var(--text-tertiary);
    opacity: 0.2;
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

  .actions-row {
    display: flex;
    gap: var(--space-3);
  }

  .actions-row > :global(*) {
    flex: 1;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @keyframes shimmer {
    0% { background-position: -200% 0; }
    100% { background-position: 200% 0; }
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
