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
use cfg_if::cfg_if;
use table::{Table};
use wasm_bindgen::prelude::*;

use crate::table::{Instruction, get_table};

// use super::packed;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

// #[wasm_bindgen]
// pub fn get_table() -> Table {
//   TABLE.clone()
// }
#[wasm_bindgen]
pub struct WTable {
  table: Vec<Instruction>,
}

#[wasm_bindgen]
impl WTable {
  #[wasm_bindgen]
  pub fn new() -> Self {
    Self { table: get_table() }
  }
  #[wasm_bindgen]
  pub fn from_string(&self, s: String) -> Result<u64, String> {
    s.parse().map(|cmd: Command| cmd.cmd)
  }
  #[wasm_bindgen]
  pub fn to_string(&self, s: u64) -> String {
    Command::new(s).to_string(Some(&self.table.as_slice()))
  }
  #[wasm_bindgen]
  pub fn set_label(&mut self, idx: usize, s: String) {
    self.table[idx].label = s;
  }
  #[wasm_bindgen]
  pub fn get(&mut self, idx: usize) -> Instruction {
    self.table[idx].clone()
  }
}

