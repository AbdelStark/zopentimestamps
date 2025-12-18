import { writable, derived } from "svelte/store";

export type SendPhase = "input" | "preview" | "sending" | "complete" | "error";

export interface SendState {
  phase: SendPhase;
  amount: string;
  address: string;
  memo: string;
  txid: string | null;
  fee: number;
  error: string | null;
}

const initialState: SendState = {
  phase: "input",
  amount: "",
  address: "",
  memo: "",
  txid: null,
  fee: 10000, // 10,000 zatoshis = 0.0001 ZEC
  error: null,
};

function createSendStore() {
  const { subscribe, set, update } = writable<SendState>(initialState);

  return {
    subscribe,
    setAmount: (amount: string) => update((s) => ({ ...s, amount })),
    setAddress: (address: string) => update((s) => ({ ...s, address })),
    setMemo: (memo: string) => update((s) => ({ ...s, memo })),
    setPhase: (phase: SendPhase) => update((s) => ({ ...s, phase })),
    setTxid: (txid: string) =>
      update((s) => ({ ...s, txid, phase: "complete" })),
    setError: (error: string) => update((s) => ({ ...s, error, phase: "error" })),
    reset: () => set(initialState),
  };
}

export const send = createSendStore();

// Derived stores
export const sendPhase = derived(send, ($s) => $s.phase);
export const sendAmount = derived(send, ($s) => $s.amount);
export const sendAddress = derived(send, ($s) => $s.address);
export const sendMemo = derived(send, ($s) => $s.memo);
export const sendTxid = derived(send, ($s) => $s.txid);
export const sendError = derived(send, ($s) => $s.error);

// Validation
export const isValidAmount = derived(send, ($s) => {
  const num = parseFloat($s.amount);
  return !isNaN(num) && num > 0;
});

export const isValidAddress = derived(send, ($s) => {
  // Basic validation - starts with t, zs, or u for Zcash addresses
  return /^(t|zs|u)[a-zA-Z0-9]{30,}$/.test($s.address);
});

export const canProceed = derived(
  [isValidAmount, isValidAddress],
  ([$amount, $address]) => $amount && $address
);
