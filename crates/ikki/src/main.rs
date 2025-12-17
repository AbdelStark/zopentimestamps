//! Ikki - A beautiful Zcash wallet for everyone
//!
//! Ikki is a consumer-grade desktop wallet designed with world-class UX,
//! inspired by modern neobank applications like Revolut.

#![allow(dead_code)]

mod app;
mod components;
mod message;
mod theme;
mod views;

fn main() -> iced::Result {
    app::run()
}
