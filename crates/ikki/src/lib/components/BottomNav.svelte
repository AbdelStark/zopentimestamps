<script lang="ts">
  import { Home, Clock, Settings } from "lucide-svelte";
  import { ui, currentView } from "../stores/ui";

  const navItems = [
    { id: "home" as const, icon: Home, label: "Home" },
    { id: "history" as const, icon: Clock, label: "Activity" },
    { id: "settings" as const, icon: Settings, label: "Settings" },
  ];
</script>

<nav class="bottom-nav">
  {#each navItems as item}
    <button
      class="nav-item"
      class:active={$currentView === item.id}
      onclick={() => ui.navigate(item.id)}
    >
      <div class="nav-icon">
        <item.icon size={20} strokeWidth={$currentView === item.id ? 2 : 1.5} />
      </div>
      <span class="nav-label">{item.label}</span>
    </button>
  {/each}
</nav>

<style>
  .bottom-nav {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: var(--nav-height);
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-around;
    padding: 0 var(--space-lg);
    z-index: 100;
  }

  .nav-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: var(--space-sm);
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-tertiary);
    transition: color var(--transition-fast);
  }

  .nav-item:hover {
    color: var(--text-secondary);
  }

  .nav-item.active {
    color: var(--text-primary);
  }

  .nav-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
  }

  .nav-label {
    font-size: 11px;
    font-weight: var(--weight-medium);
    letter-spacing: 0.02em;
  }
</style>
