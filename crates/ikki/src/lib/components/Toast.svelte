<script lang="ts">
  import { X } from "lucide-svelte";
  import { ui, type Toast } from "../stores/ui";

  export let toast: Toast;
</script>

<div class="toast toast-{toast.type}" role="alert">
  <span class="toast-message">{toast.message}</span>
  <button class="toast-close" onclick={() => ui.dismissToast(toast.id)}>
    <X size={14} />
  </button>
</div>

<style>
  .toast {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md) var(--space-lg);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    animation: slideUp var(--transition-normal) ease-out;
  }

  .toast-success {
    border-left: 2px solid var(--success);
  }

  .toast-error {
    border-left: 2px solid var(--error);
  }

  .toast-warning {
    border-left: 2px solid var(--warning);
  }

  .toast-info {
    border-left: 2px solid var(--text-secondary);
  }

  .toast-message {
    flex: 1;
    font-size: var(--text-small);
    color: var(--text-primary);
  }

  .toast-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-tertiary);
    border-radius: var(--radius-xs);
    transition: all var(--transition-fast);
  }

  .toast-close:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }
</style>
