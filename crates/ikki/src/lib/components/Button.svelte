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
    <Loader2 size={size === "sm" ? 14 : 16} class="spin" />
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
    font-weight: var(--weight-medium);
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
    white-space: nowrap;
    letter-spacing: 0.01em;
  }

  .button:disabled {
    opacity: 0.4;
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
    height: 36px;
  }

  .button-md {
    padding: var(--space-md) var(--space-lg);
    font-size: var(--text-body);
    border-radius: var(--radius-md);
    height: 44px;
  }

  .button-lg {
    padding: var(--space-md) var(--space-xl);
    font-size: var(--text-body);
    border-radius: var(--radius-md);
    height: 52px;
  }

  /* Primary - White/Light */
  .button-primary {
    background: var(--text-primary);
    color: var(--text-inverse);
  }

  .button-primary:not(:disabled):hover {
    background: var(--accent-secondary);
  }

  /* Secondary - Border */
  .button-secondary {
    background: transparent;
    color: var(--text-primary);
    border: 1px solid var(--border-light);
  }

  .button-secondary:not(:disabled):hover {
    background: var(--bg-card);
    border-color: var(--border-focus);
  }

  /* Ghost */
  .button-ghost {
    background: transparent;
    color: var(--text-secondary);
  }

  .button-ghost:not(:disabled):hover {
    background: var(--bg-card);
    color: var(--text-primary);
  }

  /* Danger */
  .button-danger {
    background: var(--error);
    color: white;
  }

  .button-danger:not(:disabled):hover {
    background: #ef4444;
  }

  .full-width {
    width: 100%;
  }
</style>
