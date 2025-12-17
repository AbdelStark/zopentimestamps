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
  <!-- Header -->
  {#if $sendPhase !== "complete"}
    <header class="send-header">
      <button class="back-button" onclick={handleBack}>
        <ArrowLeft size={20} />
      </button>
      <h1>Send</h1>
      <div class="header-spacer"></div>
    </header>
  {/if}

  <div class="send-content">
    {#if $sendPhase === "input"}
      <!-- Input Phase -->
      <div class="input-phase animate-fade-in">
        <div class="balance-display">
          <span class="balance-label">Available Balance</span>
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
            placeholder="Enter address"
            value={$sendAddress}
            oninput={handleAddressInput}
          />

          <Input
            label="Memo (optional)"
            placeholder="Add a note"
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
      <!-- Preview Phase -->
      <div class="preview-phase animate-fade-in">
        <div class="preview-amount">
          <span class="amount-value">{formatZec(amountZatoshis)}</span>
          <span class="amount-currency">ZEC</span>
        </div>

        <div class="preview-card">
          <div class="preview-row">
            <span class="preview-label">To</span>
            <span class="preview-value address">{truncateAddress($sendAddress, 10)}</span>
          </div>
          <div class="preview-divider"></div>
          <div class="preview-row">
            <span class="preview-label">Amount</span>
            <span class="preview-value">{formatZec(amountZatoshis)} ZEC</span>
          </div>
          <div class="preview-row">
            <span class="preview-label">Network Fee</span>
            <span class="preview-value">{formatZec(FEE)} ZEC</span>
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
          <Button
            variant="primary"
            size="lg"
            fullWidth
            onclick={confirmSend}
          >
            Confirm Send
          </Button>
          <Button
            variant="ghost"
            size="lg"
            fullWidth
            onclick={goBackToInput}
          >
            Edit
          </Button>
        </div>
      </div>

    {:else if $sendPhase === "sending"}
      <!-- Sending Phase -->
      <div class="sending-phase animate-fade-in">
        <div class="sending-spinner">
          <Loader2 size={40} class="spin" />
        </div>
        <h2>Processing</h2>
        <p>Please wait while your transaction is being sent...</p>
      </div>

    {:else if $sendPhase === "complete"}
      <!-- Complete Phase -->
      <div class="complete-phase animate-fade-in">
        <div class="success-icon">
          <Check size={32} strokeWidth={2.5} />
        </div>
        <h2>Sent</h2>
        <p class="txid">
          {truncateAddress($sendTxid || "", 12)}
        </p>
        <div class="form-actions">
          <Button
            variant="primary"
            size="lg"
            fullWidth
            onclick={handleDone}
          >
            Done
          </Button>
        </div>
      </div>

    {:else if $sendPhase === "error"}
      <!-- Error Phase -->
      <div class="error-phase animate-fade-in">
        <div class="error-icon">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"/>
            <line x1="15" y1="9" x2="9" y2="15"/>
            <line x1="9" y1="9" x2="15" y2="15"/>
          </svg>
        </div>
        <h2>Failed</h2>
        <p class="error-message">{$sendError}</p>
        <div class="form-actions">
          <Button
            variant="primary"
            size="lg"
            fullWidth
            onclick={goBackToInput}
          >
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

  .send-header h1 {
    font-size: var(--text-body);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    letter-spacing: 0.02em;
  }

  .header-spacer {
    width: 40px;
  }

  .send-content {
    flex: 1;
    padding: var(--space-lg);
    max-width: var(--max-width);
    margin: 0 auto;
    width: 100%;
  }

  /* Input Phase */
  .input-phase {
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
  }

  .balance-display {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-md) var(--space-lg);
    background: var(--bg-card);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .balance-label {
    font-size: var(--text-small);
    color: var(--text-secondary);
  }

  .balance-value {
    font-size: var(--text-body);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    font-family: var(--font-mono);
  }

  .form-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .amount-input-wrapper {
    position: relative;
  }

  .max-button {
    position: absolute;
    right: var(--space-sm);
    top: 50%;
    transform: translateY(25%);
    padding: var(--space-xs) var(--space-md);
    background: var(--bg-elevated);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: var(--text-caption);
    font-weight: var(--weight-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
    letter-spacing: 0.05em;
  }

  .max-button:hover {
    background: var(--text-primary);
    color: var(--bg-primary);
    border-color: var(--text-primary);
  }

  .form-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    margin-top: auto;
    padding-top: var(--space-xl);
  }

  /* Preview Phase */
  .preview-phase {
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
  }

  .preview-amount {
    text-align: center;
    padding: var(--space-xl) 0;
  }

  .amount-value {
    font-size: var(--text-display);
    font-weight: var(--weight-bold);
    color: var(--text-primary);
    font-family: var(--font-mono);
  }

  .amount-currency {
    font-size: var(--text-h3);
    font-weight: var(--weight-medium);
    color: var(--text-tertiary);
    margin-left: var(--space-sm);
  }

  .preview-card {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    padding: var(--space-lg);
  }

  .preview-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: var(--space-sm) 0;
  }

  .preview-row.total {
    font-weight: var(--weight-semibold);
  }

  .preview-label {
    font-size: var(--text-small);
    color: var(--text-secondary);
  }

  .preview-value {
    font-size: var(--text-small);
    color: var(--text-primary);
    text-align: right;
    max-width: 60%;
    word-break: break-all;
  }

  .preview-value.address {
    font-family: var(--font-mono);
  }

  .preview-value.memo {
    font-style: italic;
    color: var(--text-secondary);
  }

  .preview-divider {
    height: 1px;
    background: var(--border);
    margin: var(--space-sm) 0;
  }

  /* Sending Phase */
  .sending-phase {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: var(--space-3xl) 0;
    gap: var(--space-lg);
  }

  .sending-spinner {
    color: var(--text-secondary);
  }

  .sending-spinner :global(.spin) {
    animation: spin 1s linear infinite;
  }

  .sending-phase h2 {
    font-size: var(--text-h3);
    color: var(--text-primary);
  }

  .sending-phase p {
    color: var(--text-secondary);
    font-size: var(--text-small);
  }

  /* Complete Phase */
  .complete-phase {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: var(--space-3xl) 0;
    gap: var(--space-lg);
  }

  .success-icon {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    border: 2px solid var(--text-secondary);
    color: var(--text-primary);
  }

  .complete-phase h2 {
    font-size: var(--text-h3);
    color: var(--text-primary);
  }

  .txid {
    font-family: var(--font-mono);
    font-size: var(--text-small);
    color: var(--text-tertiary);
    background: var(--bg-card);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  /* Error Phase */
  .error-phase {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: var(--space-3xl) 0;
    gap: var(--space-lg);
  }

  .error-icon {
    color: var(--text-tertiary);
  }

  .error-phase h2 {
    font-size: var(--text-h3);
    color: var(--text-primary);
  }

  .error-message {
    color: var(--text-secondary);
    max-width: 80%;
    font-size: var(--text-small);
  }
</style>
