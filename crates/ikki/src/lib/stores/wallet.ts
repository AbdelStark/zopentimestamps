import { writable, derived } from "svelte/store";

export interface WalletInfo {
  address: string;
  balance: number;
  blockHeight: number;
}

export interface BalanceInfo {
  total: number;
  shielded: number;
  transparent: number;
}

export interface WalletState {
  initialized: boolean;
  loading: boolean;
  syncing: boolean;
  info: WalletInfo | null;
  balance: BalanceInfo | null;
  error: string | null;
}

const initialState: WalletState = {
  initialized: false,
  loading: false,
  syncing: false,
  info: null,
  balance: null,
  error: null,
};

function createWalletStore() {
  const { subscribe, set, update } = writable<WalletState>(initialState);

  return {
    subscribe,
    setLoading: (loading: boolean) => update((s) => ({ ...s, loading })),
    setSyncing: (syncing: boolean) => update((s) => ({ ...s, syncing })),
    setError: (error: string | null) => update((s) => ({ ...s, error })),
    setInfo: (info: WalletInfo) =>
      update((s) => ({
        ...s,
        initialized: true,
        info,
        balance: {
          total: info.balance,
          shielded: info.balance,
          transparent: 0,
        },
      })),
    setAddress: (address: string) =>
      update((s) => ({
        ...s,
        info: s.info ? { ...s.info, address } : null,
      })),
    updateBalance: (balance: BalanceInfo) => update((s) => ({ ...s, balance })),
    reset: () => set(initialState),
  };
}

export const wallet = createWalletStore();

// Derived stores for convenience
export const isInitialized = derived(wallet, ($w) => $w.initialized);
export const isLoading = derived(wallet, ($w) => $w.loading);
export const isSyncing = derived(wallet, ($w) => $w.syncing);
export const walletError = derived(wallet, ($w) => $w.error);
export const balance = derived(wallet, ($w) => $w.balance?.total ?? 0);
export const address = derived(wallet, ($w) => $w.info?.address ?? "");
