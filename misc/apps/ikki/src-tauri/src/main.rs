//! Ikki - A beautiful Zcash wallet for everyone
//!
//! Tauri application entry point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ikki_lib::run()
}
