mod core;
mod traits;
mod utils;

use wasmtime::*;
use serde::{Deserialize};
use std::io::Read;
use sha2::{Sha256, Digest};
use crate::core::abi_parser::abi_parser;
use crate::core::runtime::abi_reader;
use crate::traits::traits::ABIType;

fn main() {
    // abi_parser().expect("ERROR: error at abi_parser.rs file");
    abi_reader("./orascript/output/orscriptABI.json","./orascript/output/orscript.wasm")
        .expect("ERROR: Error occur at abi_reader() in main.rs");
}
