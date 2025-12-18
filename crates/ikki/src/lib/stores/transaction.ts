import { writable, derived } from "svelte/store";
import type { Transaction } from "../utils/tauri";

interface TransactionState {
  selected: Transaction | null;
}

const initialState: TransactionState = {
  selected: null,
};

function createTransactionStore() {
  const { subscribe, set, update } = writable<TransactionState>(initialState);

  return {
    subscribe,
    select: (tx: Transaction) => update((s) => ({ ...s, selected: tx })),
    clear: () => update((s) => ({ ...s, selected: null })),
    reset: () => set(initialState),
  };
}

export const transaction = createTransactionStore();

export const selectedTransaction = derived(
  transaction,
  ($tx) => $tx.selected
);
