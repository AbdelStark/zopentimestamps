<script lang="ts">
  import { Home, Clock, Settings } from "lucide-svelte";
  import { ui, currentView } from "../stores/ui";

  const navItems = [
    { id: "home" as const, icon: Home, label: "Home" },
    { id: "history" as const, icon: Clock, label: "Activity" },
    { id: "settings" as const, icon: Settings, label: "Settings" },
  ];
</script>

<nav class="nav">
  <div class="nav-container">
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={$currentView === item.id}
        onclick={() => ui.navigate(item.id)}
        aria-label={item.label}
      >
        <div class="nav-icon-container">
          {#if $currentView === item.id}
            <div class="active-bg"></div>
          {/if}
          <div class="nav-icon">
            <item.icon size={20} strokeWidth={$currentView === item.id ? 2 : 1.5} />
          </div>
        </div>
        <span class="nav-label">{item.label}</span>
      </button>
    {/each}
  </div>
</nav>

<style>
  .nav {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 100;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-subtle);
    padding-bottom: var(--safe-bottom);
  }

  .nav::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg,
      transparent,
      rgba(255, 255, 255, 0.03),
      transparent
    );
  }

  .nav-container {
    display: flex;
    align-items: center;
    justify-content: space-around;
    height: var(--nav-height);
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 0 var(--space-2);
  }

  .nav-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    padding: var(--space-2);
    color: var(--text-tertiary);
    transition: color var(--duration-fast) var(--ease-out);
  }

  .nav-item:hover {
    color: var(--text-secondary);
  }

  .nav-item.active {
    color: var(--text-primary);
  }

  .nav-icon-container {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 32px;
  }

  .active-bg {
    position: absolute;
    inset: 0;
    background: var(--accent-muted);
    border-radius: var(--radius-md);
    animation: scaleIn var(--duration-fast) var(--ease-out);
  }

  .nav-icon {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform var(--duration-normal) var(--ease-spring);
  }

  .nav-item:active .nav-icon {
    transform: scale(0.9);
  }

  .nav-label {
    font-size: var(--text-2xs);
    font-weight: var(--font-medium);
    letter-spacing: var(--tracking-wide);
  }

  .nav-item.active .nav-label {
    font-weight: var(--font-semibold);
  }
</style>
