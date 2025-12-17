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
  <div class="nav-inner">
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={$currentView === item.id}
        onclick={() => ui.navigate(item.id)}
      >
        <div class="nav-icon-wrapper">
          {#if $currentView === item.id}
            <div class="active-indicator"></div>
          {/if}
          <div class="nav-icon">
            <item.icon size={20} strokeWidth={$currentView === item.id ? 2.25 : 1.75} />
          </div>
        </div>
        <span class="nav-label">{item.label}</span>
      </button>
    {/each}
  </div>
</nav>

<style>
  .bottom-nav {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    padding-bottom: var(--safe-area-bottom);
    z-index: 100;
  }

  .nav-inner {
    height: var(--nav-height);
    display: flex;
    align-items: center;
    justify-content: space-around;
    padding: 0 var(--space-4);
    max-width: var(--max-width);
    margin: 0 auto;
  }

  .nav-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: var(--space-2) var(--space-2);
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-tertiary);
    transition: color var(--transition-fast);
    position: relative;
    -webkit-tap-highlight-color: transparent;
  }

  .nav-item:hover {
    color: var(--text-secondary);
  }

  .nav-item.active {
    color: var(--text-primary);
  }

  .nav-icon-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .active-indicator {
    position: absolute;
    width: 40px;
    height: 40px;
    border-radius: var(--radius-md);
    background: var(--accent-dim);
    animation: scaleIn var(--transition-fast) ease-out;
  }

  .nav-icon {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    transition: transform var(--transition-spring);
  }

  .nav-item.active .nav-icon {
    transform: scale(1.05);
  }

  .nav-item:active .nav-icon {
    transform: scale(0.95);
  }

  .nav-label {
    font-size: 10px;
    font-weight: var(--weight-medium);
    letter-spacing: 0.03em;
    transition: opacity var(--transition-fast);
  }

  .nav-item.active .nav-label {
    font-weight: var(--weight-semibold);
  }
</style>
