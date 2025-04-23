mod core;
mod traits;
mod utils;

use wasmtime::*;
use serde::{Deserialize};
use std::io::Read;
use sha2::{Sha256, Digest};
use crate::core::runtime::abi_reader;
use crate::traits::traits::ABIType;

fn main() {
    // parser().expect("INFO: error at abi_parser.rs file");
    abi_reader("./orascripts/orascriptABI.json","./orascripts/addTwo1.wasm").expect("ERROR: Error occur at abi_reader() in main.rs");
    // wasmtime_runner().expect("INFO: error at wastime_runner() in main.rs");
    // abi_reader("./orascripts/addTwoABI.json").unwrap()
}
