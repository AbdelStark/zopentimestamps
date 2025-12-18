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
  class="btn btn-{variant} btn-{size}"
  class:full-width={fullWidth}
  disabled={disabled || loading}
  {onclick}
>
  {#if loading}
    <Loader2 size={size === "sm" ? 14 : 16} class="spinning" />
  {/if}
  <span class="btn-content"><slot /></span>
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    font-family: var(--font-sans);
    font-weight: var(--font-medium);
    border: none;
    cursor: pointer;
    white-space: nowrap;
    letter-spacing: var(--tracking-wide);
    position: relative;
    overflow: hidden;
    transition:
      background var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
    pointer-events: none;
  }

  .btn :global(.spinning) {
    animation: spin 0.9s linear infinite;
  }

  .btn-content {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
  }

  /* Sizes */
  .btn-sm {
    padding: 0 var(--space-3);
    font-size: var(--text-sm);
    border-radius: var(--radius-sm);
    height: 32px;
  }

  .btn-md {
    padding: 0 var(--space-5);
    font-size: var(--text-base);
    border-radius: var(--radius-md);
    height: 44px;
  }

  .btn-lg {
    padding: 0 var(--space-6);
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    border-radius: var(--radius-md);
    height: 52px;
  }

  /* Primary */
  .btn-primary {
    background: var(--text-primary);
    color: var(--text-inverse);
  }

  .btn-primary:not(:disabled):hover {
    background: var(--accent-soft);
  }

  .btn-primary:not(:disabled):active {
    transform: scale(0.98);
    background: #d1d5db;
  }

  /* Secondary */
  .btn-secondary {
    background: transparent;
    color: var(--text-primary);
    box-shadow: inset 0 0 0 1px var(--border);
  }

  .btn-secondary:not(:disabled):hover {
    background: var(--bg-hover);
    box-shadow: inset 0 0 0 1px var(--border-emphasis);
  }

  .btn-secondary:not(:disabled):active {
    transform: scale(0.98);
    background: var(--bg-active);
  }

  /* Ghost */
  .btn-ghost {
    background: transparent;
    color: var(--text-secondary);
  }

  .btn-ghost:not(:disabled):hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .btn-ghost:not(:disabled):active {
    transform: scale(0.98);
    background: var(--bg-active);
  }

  /* Danger */
  .btn-danger {
    background: var(--error);
    color: white;
  }

  .btn-danger:not(:disabled):hover {
    background: #dc2626;
  }

  .btn-danger:not(:disabled):active {
    transform: scale(0.98);
  }

  .full-width {
    width: 100%;
  }
</style>
