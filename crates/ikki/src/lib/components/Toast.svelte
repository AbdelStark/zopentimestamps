<script lang="ts">
  import { CheckCircle, XCircle, AlertCircle, Info, X } from "lucide-svelte";
  import { ui, type Toast } from "../stores/ui";

  export let toast: Toast;

  const icons = {
    success: CheckCircle,
    error: XCircle,
    warning: AlertCircle,
    info: Info,
  };

  $: Icon = icons[toast.type] || Info;
</script>

<div class="toast toast-{toast.type}" role="alert">
  <div class="toast-icon">
    <Icon size={16} strokeWidth={2} />
  </div>
  <span class="toast-message">{toast.message}</span>
  <button class="toast-dismiss" onclick={() => ui.dismissToast(toast.id)} aria-label="Dismiss">
    <X size={14} strokeWidth={2} />
  </button>
</div>

<style>
  .toast {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    animation: fadeInUp var(--duration-normal) var(--ease-out);
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
    color: var(--text-secondary);
  }

  .toast-message {
    flex: 1;
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    line-height: var(--leading-snug);
  }

  .toast-dismiss {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    color: var(--text-tertiary);
    border-radius: var(--radius-sm);
    transition: all var(--duration-fast) var(--ease-out);
  }

  .toast-dismiss:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
</style>
