<script lang="ts">
  import { ArrowLeft, Check, Loader2 } from "lucide-svelte";
  import { send, sendPhase, sendAmount, sendAddress, sendMemo, sendTxid, sendError, canProceed } from "../lib/stores/send";
  import { balance } from "../lib/stores/wallet";
  import { ui } from "../lib/stores/ui";
  import { sendTransaction } from "../lib/utils/tauri";
  import { formatZec, parseZec, truncateAddress } from "../lib/utils/format";
  import Button from "../lib/components/Button.svelte";
  import Input from "../lib/components/Input.svelte";

  const FEE = 10000; // 0.0001 ZEC in zatoshis

  function handleAmountInput(e: Event) {
    const target = e.target as HTMLInputElement;
    send.setAmount(target.value);
  }

  function handleAddressInput(e: Event) {
    const target = e.target as HTMLInputElement;
    send.setAddress(target.value);
  }

  function handleMemoInput(e: Event) {
    const target = e.target as HTMLInputElement;
    send.setMemo(target.value);
  }

  function setMaxAmount() {
    const maxZatoshis = Math.max(0, $balance - FEE);
    const maxZec = maxZatoshis / 100_000_000;
    send.setAmount(maxZec.toString());
  }

  function goToPreview() {
    send.setPhase("preview");
  }

  function goBackToInput() {
    send.setPhase("input");
  }

  async function confirmSend() {
    send.setPhase("sending");
    try {
      const amountZatoshis = parseZec($sendAmount);
      const result = await sendTransaction($sendAddress, amountZatoshis, $sendMemo || undefined);
      send.setTxid(result.txid);
      ui.showToast("Transaction sent", "success");
    } catch (e) {
      send.setError(String(e));
      ui.showToast(`Send failed: ${e}`, "error");
    }
  }

  function handleDone() {
    send.reset();
    ui.navigate("home");
  }

  function handleBack() {
    if ($sendPhase === "preview") {
      goBackToInput();
    } else if ($sendPhase === "input") {
      send.reset();
      ui.navigate("home");
    }
  }

  $: amountZatoshis = parseZec($sendAmount);
  $: totalWithFee = amountZatoshis + FEE;
</script>

<div class="send">
  {#if $sendPhase !== "complete"}
    <header class="send-header">
      <button class="back-button" onclick={handleBack}>
        <ArrowLeft size={20} strokeWidth={2} />
      </button>
      <h1>Send</h1>
      <div class="header-spacer"></div>
    </header>
  {/if}

  <div class="send-content">
    {#if $sendPhase === "input"}
      <div class="input-phase">
        <div class="balance-display">
          <span class="balance-label">Available</span>
          <span class="balance-value">{formatZec($balance)} ZEC</span>
        </div>

        <div class="form-section">
          <div class="amount-input-wrapper">
            <Input
              type="text"
              inputmode="decimal"
              label="Amount"
              placeholder="0.00"
              value={$sendAmount}
              oninput={handleAmountInput}
            />
            <button class="max-button" onclick={setMaxAmount}>MAX</button>
          </div>

          <Input
            label="Recipient Address"
            placeholder="Enter Zcash address"
            value={$sendAddress}
            oninput={handleAddressInput}
          />

          <Input
            label="Memo (optional)"
            placeholder="Add a private note"
            value={$sendMemo}
            oninput={handleMemoInput}
          />
        </div>

        <div class="form-actions">
          <Button
            variant="primary"
            size="lg"
            fullWidth
            disabled={!$canProceed}
            onclick={goToPreview}
          >
            Review
          </Button>
        </div>
      </div>

    {:else if $sendPhase === "preview"}
      <div class="preview-phase">
        <div class="preview-amount">
          <span class="amount-value">{formatZec(amountZatoshis)}</span>
          <span class="amount-currency">ZEC</span>
        </div>

        <div class="preview-card">
          <div class="preview-row">
            <span class="preview-label">To</span>
            <span class="preview-value mono">{truncateAddress($sendAddress, 12)}</span>
          </div>
          <div class="preview-divider"></div>
          <div class="preview-row">
            <span class="preview-label">Amount</span>
            <span class="preview-value">{formatZec(amountZatoshis)} ZEC</span>
          </div>
          <div class="preview-row">
            <span class="preview-label">Network fee</span>
            <span class="preview-value secondary">{formatZec(FEE)} ZEC</span>
          </div>
          <div class="preview-divider"></div>
          <div class="preview-row total">
            <span class="preview-label">Total</span>
            <span class="preview-value">{formatZec(totalWithFee)} ZEC</span>
          </div>
          {#if $sendMemo}
            <div class="preview-divider"></div>
            <div class="preview-row">
              <span class="preview-label">Memo</span>
              <span class="preview-value memo">{$sendMemo}</span>
            </div>
          {/if}
        </div>

        <div class="form-actions">
          <Button variant="primary" size="lg" fullWidth onclick={confirmSend}>
            Confirm Send
          </Button>
          <Button variant="ghost" size="lg" fullWidth onclick={goBackToInput}>
            Edit
          </Button>
        </div>
      </div>

    {:else if $sendPhase === "sending"}
      <div class="status-phase">
        <div class="status-icon spinning">
          <Loader2 size={32} class="spin" />
        </div>
        <h2>Sending</h2>
        <p>Broadcasting transaction to the network...</p>
      </div>

    {:else if $sendPhase === "complete"}
      <div class="status-phase">
        <div class="status-icon success">
          <Check size={28} strokeWidth={2.5} />
        </div>
        <h2>Sent</h2>
        <div class="txid-badge">
          {truncateAddress($sendTxid || "", 14)}
        </div>
        <div class="form-actions wide">
          <Button variant="primary" size="lg" fullWidth onclick={handleDone}>
            Done
          </Button>
        </div>
      </div>

    {:else if $sendPhase === "error"}
      <div class="status-phase">
        <div class="status-icon error">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </div>
        <h2>Failed</h2>
        <p class="error-text">{$sendError}</p>
        <div class="form-actions wide">
          <Button variant="primary" size="lg" fullWidth onclick={goBackToInput}>
            Try Again
          </Button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .send {
    min-height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
  }

  .send-header {
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

  .send-header h1 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-wide);
  }

  .header-spacer {
    width: 40px;
  }

  .send-content {
    flex: 1;
    padding: var(--space-5);
    max-width: var(--max-width);
    margin: 0 auto;
    width: 100%;
    animation: fadeIn var(--duration-normal) var(--ease-out);
  }

  /* Input Phase */
  .input-phase {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    min-height: 100%;
  }

  .balance-display {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-3) var(--space-4);
    background: var(--bg-card);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .balance-label {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    letter-spacing: var(--tracking-wide);
  }

  .balance-value {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }

  .form-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .amount-input-wrapper {
    position: relative;
  }

  .max-button {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(20%);
    padding: 5px 10px;
    background: var(--bg-elevated);
    color: var(--text-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 9px;
    font-weight: var(--font-semibold);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
    letter-spacing: var(--tracking-widest);
    -webkit-tap-highlight-color: transparent;
  }

  .max-button:hover {
    background: var(--text-primary);
    color: var(--text-inverse);
    border-color: var(--text-primary);
  }

  .max-button:active {
    transform: translateY(20%) scale(0.95);
  }

  .form-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-top: auto;
    padding-top: var(--space-6);
  }

  .form-actions.wide {
    width: 100%;
    padding: var(--space-6) var(--space-4) 0;
  }

  /* Preview Phase */
  .preview-phase {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .preview-amount {
    text-align: center;
    padding: var(--space-6) 0 var(--space-4);
  }

  .amount-value {
    font-size: var(--text-2xl);
    font-weight: var(--font-bold);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    letter-spacing: var(--tracking-tight);
  }

  .amount-currency {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-tertiary);
    margin-left: var(--space-2);
    letter-spacing: var(--tracking-wider);
  }

  .preview-card {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    padding: var(--space-4);
    position: relative;
  }

  .preview-card::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--gradient-card);
    pointer-events: none;
  }

  .preview-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: var(--space-2) 0;
    position: relative;
  }

  .preview-row.total {
    padding-top: var(--space-3);
  }

  .preview-row.total .preview-label,
  .preview-row.total .preview-value {
    font-weight: var(--font-semibold);
  }

  .preview-label {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    letter-spacing: var(--tracking-wide);
  }

  .preview-value {
    font-size: var(--text-xs);
    color: var(--text-primary);
    text-align: right;
    max-width: 60%;
    word-break: break-all;
    letter-spacing: var(--tracking-wide);
  }

  .preview-value.mono {
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
  }

  .preview-value.secondary {
    color: var(--text-tertiary);
  }

  .preview-value.memo {
    font-style: italic;
    color: var(--text-secondary);
  }

  .preview-divider {
    height: 1px;
    background: var(--divider);
    margin: var(--space-2) 0;
  }

  /* Status Phases */
  .status-phase {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: var(--space-16) var(--space-4);
    gap: var(--space-4);
    min-height: 60vh;
    animation: fadeIn var(--duration-normal) var(--ease-out);
  }

  .status-icon {
    width: 72px;
    height: 72px;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-2);
  }

  .status-icon.spinning {
    background: var(--bg-card);
    border: 1px solid var(--border);
    color: var(--text-secondary);
  }

  .status-icon.spinning :global(.spin) {
    animation: spin 1s linear infinite;
  }

  .status-icon.success {
    background: var(--receive-muted);
    border: 1px solid rgba(52, 211, 153, 0.2);
    color: var(--receive);
    animation: scaleIn var(--duration-normal) var(--ease-spring);
  }

  .status-icon.error {
    background: var(--error-muted);
    border: 1px solid rgba(239, 68, 68, 0.2);
    color: var(--error);
    animation: scaleIn var(--duration-normal) var(--ease-spring);
  }

  .status-phase h2 {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tight);
  }

  .status-phase p {
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    max-width: 260px;
    line-height: var(--leading-relaxed);
  }

  .txid-badge {
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    color: var(--text-tertiary);
    background: var(--bg-card);
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    letter-spacing: var(--tracking-wide);
  }

  .error-text {
    color: var(--text-secondary);
    font-size: var(--text-xs);
    max-width: 280px;
  }
</style>
