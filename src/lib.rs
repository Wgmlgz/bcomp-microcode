#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;
// extern crate cfg_if;
// extern crate wasm_bindgen;


mod control_signals;
mod regexes;
mod table;
mod tests;
mod command;

use command::{Command, ParseErr};
// use cfg_if::cfg_if;
// use wasm_bindgen::prelude::*;

// // use super::packed;

// cfg_if! {
//     // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
//     // allocator.
//     if #[cfg(feature = "wee_alloc")] {
//         extern crate wee_alloc;
//         #[global_allocator]
//         static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
//     }
// }



// #[wasm_bindgen(catch)]
// pub fn parse_microcode(s: String) -> Result<u64, String> {
//   s.parse().map(|cmd: Command| cmd.cmd)
// }

// #[wasm_bindgen]
// pub fn greet() -> String {
//   "sus3".into()
// }