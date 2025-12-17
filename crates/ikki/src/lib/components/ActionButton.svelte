<script lang="ts">
  import { ArrowUp, ArrowDown } from "lucide-svelte";

  export let variant: "send" | "receive" = "send";
  export let onclick: () => void = () => {};
  export let disabled: boolean = false;

  const config = {
    send: { icon: ArrowUp, label: "Send" },
    receive: { icon: ArrowDown, label: "Receive" },
  };

  $: Icon = config[variant].icon;
  $: label = config[variant].label;
</script>

<button
  class="action-button"
  class:send={variant === "send"}
  class:receive={variant === "receive"}
  {disabled}
  {onclick}
>
  <div class="action-icon">
    <Icon size={22} strokeWidth={2.5} />
  </div>
  <span class="action-label">{label}</span>
</button>

<style>
  .action-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-6);
    background: none;
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .action-button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .action-icon {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    border: 1px solid var(--border);
    color: var(--text-primary);
    transition: all var(--transition-normal);
    position: relative;
  }

  .action-icon::before {
    content: '';
    position: absolute;
    inset: -1px;
    border-radius: 50%;
    background: transparent;
    transition: all var(--transition-normal);
    z-index: -1;
  }

  .action-button:not(:disabled):hover .action-icon {
    background: var(--bg-elevated);
    border-color: var(--border-light);
    transform: translateY(-2px);
  }

  .action-button:not(:disabled):hover .action-icon::before {
    box-shadow: 0 8px 24px rgba(255, 255, 255, 0.04);
  }

  .action-button:not(:disabled):active .action-icon {
    transform: translateY(0);
  }

  .action-button.send:not(:disabled):hover .action-icon {
    border-color: rgba(248, 113, 113, 0.3);
  }

  .action-button.receive:not(:disabled):hover .action-icon {
    border-color: rgba(74, 222, 128, 0.3);
  }

  .action-label {
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    letter-spacing: 0.01em;
  }
</style>
