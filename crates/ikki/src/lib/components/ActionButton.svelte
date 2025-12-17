<script lang="ts">
  import { ArrowUpRight, ArrowDownLeft, RefreshCw, MoreHorizontal } from "lucide-svelte";

  export let variant: "send" | "receive" | "swap" | "more" = "send";
  export let label: string = "";
  export let onclick: () => void = () => {};
  export let disabled: boolean = false;

  const icons = {
    send: ArrowUpRight,
    receive: ArrowDownLeft,
    swap: RefreshCw,
    more: MoreHorizontal,
  };

  const defaultLabels = {
    send: "Send",
    receive: "Receive",
    swap: "Swap",
    more: "More",
  };

  $: Icon = icons[variant];
  $: displayLabel = label || defaultLabels[variant];
</script>

<button
  class="action-button action-{variant}"
  {disabled}
  {onclick}
>
  <div class="action-icon">
    <Icon size={24} strokeWidth={2.5} />
  </div>
  <span class="action-label">{displayLabel}</span>
</button>

<style>
  .action-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-md);
    background: none;
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .action-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-icon {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
  }

  .action-button:not(:disabled):hover .action-icon {
    transform: scale(1.05);
  }

  .action-button:not(:disabled):active .action-icon {
    transform: scale(0.95);
  }

  .action-send .action-icon {
    background: var(--send);
    color: white;
    box-shadow: 0 4px 16px rgba(255, 69, 58, 0.4);
  }

  .action-receive .action-icon {
    background: var(--receive);
    color: white;
    box-shadow: 0 4px 16px rgba(48, 209, 88, 0.4);
  }

  .action-swap .action-icon {
    background: var(--accent);
    color: var(--bg-primary);
    box-shadow: 0 4px 16px var(--accent-glow);
  }

  .action-more .action-icon {
    background: var(--bg-card);
    color: var(--text-secondary);
  }

  .action-label {
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }
</style>
