import { writable, derived } from "svelte/store";

export type View = "home" | "send" | "receive" | "history" | "settings" | "contacts" | "transaction-detail";

export type ToastType = "success" | "error" | "warning" | "info";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
}

export interface UIState {
  currentView: View;
  needsOnboarding: boolean;
  toasts: Toast[];
  modalOpen: boolean;
}

const initialState: UIState = {
  currentView: "home",
  needsOnboarding: true,
  toasts: [],
  modalOpen: false,
};

function createUIStore() {
  const { subscribe, set, update } = writable<UIState>(initialState);

  return {
    subscribe,
    navigate: (view: View) => update((s) => ({ ...s, currentView: view })),
    setOnboardingComplete: () =>
      update((s) => ({ ...s, needsOnboarding: false })),
    setNeedsOnboarding: (needs: boolean) =>
      update((s) => ({ ...s, needsOnboarding: needs })),
    showToast: (message: string, type: ToastType = "info", duration = 3000) => {
      const id = Math.random().toString(36).substring(2, 9);
      update((s) => ({
        ...s,
        toasts: [...s.toasts, { id, message, type, duration }],
      }));

      if (duration > 0) {
        setTimeout(() => {
          update((s) => ({
            ...s,
            toasts: s.toasts.filter((t) => t.id !== id),
          }));
        }, duration);
      }
    },
    dismissToast: (id: string) =>
      update((s) => ({
        ...s,
        toasts: s.toasts.filter((t) => t.id !== id),
      })),
    setModalOpen: (open: boolean) => update((s) => ({ ...s, modalOpen: open })),
    reset: () => set(initialState),
  };
}

export const ui = createUIStore();

// Derived stores
export const currentView = derived(ui, ($ui) => $ui.currentView);
export const needsOnboarding = derived(ui, ($ui) => $ui.needsOnboarding);
export const toasts = derived(ui, ($ui) => $ui.toasts);
