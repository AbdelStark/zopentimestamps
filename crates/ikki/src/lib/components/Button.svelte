<script lang="ts">
  import { Loader2 } from "lucide-svelte";

  export let variant: "primary" | "secondary" | "ghost" | "danger" = "primary";
  export let size: "sm" | "md" | "lg" = "md";
  export let disabled: boolean = false;
  export let loading: boolean = false;
  export let fullWidth: boolean = false;
  export let onclick: () => void = () => {};
</script>

<button
  class="button button-{variant} button-{size}"
  class:full-width={fullWidth}
  disabled={disabled || loading}
  {onclick}
>
  {#if loading}
    <Loader2 size={size === "sm" ? 14 : size === "lg" ? 20 : 16} class="spin" />
  {/if}
  <slot />
</button>

<style>
  .button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    font-family: var(--font-family);
    font-weight: var(--weight-semibold);
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
    white-space: nowrap;
  }

  .button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .button :global(.spin) {
    animation: spin 1s linear infinite;
  }

  /* Sizes */
  .button-sm {
    padding: var(--space-sm) var(--space-md);
    font-size: var(--text-small);
    border-radius: var(--radius-sm);
  }

  .button-md {
    padding: var(--space-md) var(--space-lg);
    font-size: var(--text-body);
    border-radius: var(--radius-md);
  }

  .button-lg {
    padding: var(--space-lg) var(--space-xl);
    font-size: var(--text-body);
    border-radius: var(--radius-md);
  }

  /* Variants */
  .button-primary {
    background: var(--accent);
    color: var(--bg-primary);
  }

  .button-primary:not(:disabled):hover {
    background: var(--accent-hover);
    box-shadow: var(--shadow-glow);
  }

  .button-secondary {
    background: var(--bg-card);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .button-secondary:not(:disabled):hover {
    background: var(--bg-elevated);
    border-color: var(--border-light);
  }

  .button-ghost {
    background: transparent;
    color: var(--text-secondary);
  }

  .button-ghost:not(:disabled):hover {
    background: var(--bg-card);
    color: var(--text-primary);
  }

  .button-danger {
    background: var(--send);
    color: white;
  }

  .button-danger:not(:disabled):hover {
    background: #e63e35;
    box-shadow: 0 4px 16px rgba(255, 69, 58, 0.4);
  }

  .full-width {
    width: 100%;
  }
</style>
