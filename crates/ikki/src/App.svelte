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
  import Contacts from "./routes/Contacts.svelte";
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
            balance: {
              total: walletInfo.balance.total,
              shielded: walletInfo.balance.shielded,
              transparent: walletInfo.balance.transparent,
            },
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
        <span class="logo-text">ikki</span>
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
      {:else if $currentView === "contacts"}
        <Contacts />
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
    gap: var(--space-8);
    animation: fadeIn var(--duration-normal) var(--ease-out);
  }

  .loading-logo {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .logo-text {
    font-size: 3rem;
    font-weight: var(--font-bold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tighter);
  }

  .loading-spinner {
    width: 28px;
    height: 28px;
    border: 2px solid var(--border);
    border-top-color: var(--text-primary);
    border-radius: var(--radius-full);
    animation: spin 0.9s linear infinite;
  }

  .app-content {
    flex: 1;
    padding-bottom: var(--nav-height);
    overflow-y: auto;
  }

  .toast-container {
    position: fixed;
    bottom: calc(var(--nav-height) + var(--space-4));
    left: var(--space-4);
    right: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    z-index: 1000;
    pointer-events: none;
    max-width: var(--max-width);
    margin: 0 auto;
  }

  .toast-container > :global(*) {
    pointer-events: auto;
  }
</style>
