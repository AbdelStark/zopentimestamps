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
    <Icon size={20} strokeWidth={2} />
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
    transition: transform var(--duration-fast) var(--ease-out);
  }

  .action-button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
    pointer-events: none;
  }

  .action-button:not(:disabled):active {
    transform: scale(0.95);
  }

  .action-icon {
    width: 52px;
    height: 52px;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    border: 1px solid var(--border);
    color: var(--text-primary);
    position: relative;
    transition:
      background var(--duration-normal) var(--ease-out),
      border-color var(--duration-normal) var(--ease-out),
      transform var(--duration-normal) var(--ease-spring),
      box-shadow var(--duration-normal) var(--ease-out);
  }

  .action-button:not(:disabled):hover .action-icon {
    background: var(--bg-elevated);
    border-color: var(--border-emphasis);
    transform: translateY(-3px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }

  .action-button.send:not(:disabled):hover .action-icon {
    border-color: rgba(248, 113, 113, 0.4);
    box-shadow: 0 8px 24px rgba(248, 113, 113, 0.1);
  }

  .action-button.receive:not(:disabled):hover .action-icon {
    border-color: rgba(52, 211, 153, 0.4);
    box-shadow: 0 8px 24px rgba(52, 211, 153, 0.1);
  }

  .action-label {
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    color: var(--text-secondary);
    letter-spacing: var(--tracking-wide);
  }

  .action-button:not(:disabled):hover .action-label {
    color: var(--text-primary);
  }
</style>
