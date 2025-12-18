import { invoke } from "@tauri-apps/api/core";

// Types matching Rust structs
export interface BalanceInfo {
  total: number;
  shielded: number;
  transparent: number;
}

export interface WalletInfo {
  address: string;
  balance: BalanceInfo;
  block_height: number;
}

export interface SyncResult {
  block_height: number;
  balance: BalanceInfo;
}

export interface SendResult {
  txid: string;
  amount: number;
  fee: number;
}

export interface Transaction {
  txid: string;
  tx_type: "sent" | "received" | "shielding" | "internal";
  amount: number;
  timestamp: number;
  address: string | null;
  memo: string | null;
  status: "pending" | "confirmed" | "failed";
  confirmations: number;
}

// Wallet API
export async function checkWalletExists(): Promise<boolean> {
  return invoke<boolean>("check_wallet_exists");
}

export async function generateSeed(): Promise<string> {
  return invoke<string>("generate_seed");
}

export async function initWallet(
  seed: string,
  birthdayHeight?: number
): Promise<WalletInfo> {
  return invoke<WalletInfo>("init_wallet", {
    seed,
    birthdayHeight: birthdayHeight ?? null,
  });
}

export async function loadWallet(
  seed: string,
  birthdayHeight?: number
): Promise<WalletInfo> {
  return invoke<WalletInfo>("load_wallet", {
    seed,
    birthdayHeight: birthdayHeight ?? null,
  });
}

export async function resetWallet(): Promise<void> {
  return invoke<void>("reset_wallet");
}

export async function autoLoadWallet(): Promise<WalletInfo | null> {
  return invoke<WalletInfo | null>("auto_load_wallet");
}

export async function getBalance(): Promise<BalanceInfo> {
  return invoke<BalanceInfo>("get_balance");
}

export async function getAddress(): Promise<string> {
  return invoke<string>("get_address");
}

export async function getNewAddress(): Promise<string> {
  return invoke<string>("get_new_address");
}

export async function getAllAddresses(): Promise<string[]> {
  return invoke<string[]>("get_all_addresses");
}

export async function syncWallet(): Promise<SyncResult> {
  return invoke<SyncResult>("sync_wallet");
}

// Transaction API
export async function sendTransaction(
  toAddress: string,
  amount: number,
  memo?: string
): Promise<SendResult> {
  return invoke<SendResult>("send_transaction", {
    toAddress,
    amount,
    memo: memo || null,
  });
}

export async function getTransactions(): Promise<Transaction[]> {
  return invoke<Transaction[]>("get_transactions");
}
