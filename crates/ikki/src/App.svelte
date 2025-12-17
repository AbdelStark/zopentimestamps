<script lang="ts">
  import { onMount } from "svelte";
  import { ui, currentView, needsOnboarding, toasts } from "./lib/stores/ui";
  import { wallet } from "./lib/stores/wallet";
  import { checkWalletExists, autoLoadWallet } from "./lib/utils/tauri";

  // Views
  import Home from "./routes/Home.svelte";
  import Send from "./routes/Send.svelte";
  import Receive from "./routes/Receive.svelte";
  import History from "./routes/History.svelte";
  import Settings from "./routes/Settings.svelte";
  import Onboarding from "./routes/Onboarding.svelte";

  // Components
  import BottomNav from "./lib/components/BottomNav.svelte";
  import Toast from "./lib/components/Toast.svelte";

  let loading = true;

  onMount(async () => {
    try {
      const exists = await checkWalletExists();
      if (exists) {
        // Try to auto-load the wallet
        const walletInfo = await autoLoadWallet();
        if (walletInfo) {
          wallet.setInfo({
            address: walletInfo.address,
            balance: walletInfo.balance,
            blockHeight: walletInfo.block_height,
          });
          ui.setNeedsOnboarding(false);
        } else {
          // Config exists but couldn't load - show onboarding
          ui.setNeedsOnboarding(true);
        }
      } else {
        ui.setNeedsOnboarding(true);
      }
    } catch (e) {
      console.error("Failed to load wallet:", e);
      ui.setNeedsOnboarding(true);
    } finally {
      loading = false;
    }
  });
</script>

<main class="app">
  {#if loading}
    <div class="loading-screen">
      <div class="loading-logo">
        <span class="logo-text">Ikki</span>
      </div>
      <div class="loading-spinner"></div>
    </div>
  {:else if $needsOnboarding}
    <Onboarding />
  {:else}
    <div class="app-content">
      {#if $currentView === "home"}
        <Home />
      {:else if $currentView === "send"}
        <Send />
      {:else if $currentView === "receive"}
        <Receive />
      {:else if $currentView === "history"}
        <History />
      {:else if $currentView === "settings"}
        <Settings />
      {/if}
    </div>
    <BottomNav />
  {/if}

  <!-- Toasts -->
  <div class="toast-container">
    {#each $toasts as toast (toast.id)}
      <Toast {toast} />
    {/each}
  </div>
</main>

<style>
  .app {
    width: 100%;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
    position: relative;
  }

  .loading-screen {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-xl);
  }

  .loading-logo {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .logo-text {
    font-size: 3rem;
    font-weight: var(--weight-bold);
    background: var(--gradient-accent);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--bg-elevated);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .app-content {
    flex: 1;
    padding-bottom: var(--nav-height);
    overflow-y: auto;
  }

  .toast-container {
    position: fixed;
    bottom: calc(var(--nav-height) + var(--space-md));
    left: var(--space-md);
    right: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    z-index: 1000;
    pointer-events: none;
  }

  .toast-container > :global(*) {
    pointer-events: auto;
  }
</style>
