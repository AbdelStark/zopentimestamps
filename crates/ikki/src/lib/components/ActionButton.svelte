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

<button class="action-button" class:send={variant === "send"} class:receive={variant === "receive"} {disabled} {onclick}>
  <div class="action-icon">
    <Icon size={20} strokeWidth={2} />
  </div>
  <span class="action-label">{label}</span>
</button>

<style>
  .action-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-md) var(--space-xl);
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
    width: 48px;
    height: 48px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    border: 1px solid var(--border);
    color: var(--text-primary);
    transition: all var(--transition-fast);
  }

  .action-button:not(:disabled):hover .action-icon {
    background: var(--bg-elevated);
    border-color: var(--border-light);
    transform: translateY(-2px);
  }

  .action-button:not(:disabled):active .action-icon {
    transform: translateY(0);
  }

  .action-button.send .action-icon {
    color: var(--text-primary);
  }

  .action-button.receive .action-icon {
    color: var(--text-primary);
  }

  .action-label {
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }
</style>
