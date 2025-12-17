<script lang="ts">
  import { X, CheckCircle, AlertCircle, AlertTriangle, Info } from "lucide-svelte";
  import { ui, type Toast } from "../stores/ui";

  export let toast: Toast;

  const icons = {
    success: CheckCircle,
    error: AlertCircle,
    warning: AlertTriangle,
    info: Info,
  };

  $: Icon = icons[toast.type];
</script>

<div class="toast toast-{toast.type}" role="alert">
  <div class="toast-icon">
    <Icon size={20} />
  </div>
  <span class="toast-message">{toast.message}</span>
  <button class="toast-close" onclick={() => ui.dismissToast(toast.id)}>
    <X size={16} />
  </button>
</div>

<style>
  .toast {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-md);
    background: var(--bg-card);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    animation: slideUp var(--transition-normal) ease-out;
    border-left: 3px solid;
  }

  .toast-success {
    border-left-color: var(--success);
  }

  .toast-error {
    border-left-color: var(--error);
  }

  .toast-warning {
    border-left-color: var(--warning);
  }

  .toast-info {
    border-left-color: var(--info);
  }

  .toast-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .toast-success .toast-icon {
    color: var(--success);
  }

  .toast-error .toast-icon {
    color: var(--error);
  }

  .toast-warning .toast-icon {
    color: var(--warning);
  }

  .toast-info .toast-icon {
    color: var(--info);
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
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }

  .toast-close:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }
</style>
