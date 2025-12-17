<script lang="ts">
  import { Home, History, Settings } from "lucide-svelte";
  import { ui, currentView } from "../stores/ui";

  const navItems = [
    { id: "home" as const, icon: Home, label: "Home" },
    { id: "history" as const, icon: History, label: "Activity" },
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
        <item.icon size={24} strokeWidth={$currentView === item.id ? 2.5 : 2} />
      </div>
      <span class="nav-label">{item.label}</span>
      {#if $currentView === item.id}
        <div class="nav-indicator"></div>
      {/if}
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
    padding: 0 var(--space-md);
    z-index: 100;
  }

  .nav-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-xs);
    padding: var(--space-sm);
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-tertiary);
    transition: color var(--transition-fast);
    position: relative;
  }

  .nav-item:hover {
    color: var(--text-secondary);
  }

  .nav-item.active {
    color: var(--accent);
  }

  .nav-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
  }

  .nav-label {
    font-size: var(--text-caption);
    font-weight: var(--weight-medium);
  }

  .nav-indicator {
    position: absolute;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 24px;
    height: 3px;
    background: var(--accent);
    border-radius: 0 0 var(--radius-full) var(--radius-full);
  }
</style>
