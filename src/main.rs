mod core;
mod traits;
mod utils;
mod server;

use std::collections::HashMap;
use std::convert::Infallible;
use wasmtime::*;
use serde::{Deserialize};
use std::io::Read;
use hyper::Server;
use hyper::service::{make_service_fn, service_fn};
use sha2::{Sha256, Digest};
use crate::core::abi_parser::abi_parser;
use crate::core::runtime::abi_reader;
use crate::traits::traits::ABIType;


#[tokio::main]
async fn main() {
    // abi_parser().expect("ERROR: error at abi_parser.rs file");

    abi_reader("./orascript/output/orscript2ABI.json","./orascript/assembly/orscript2.wasm")
            .expect("ERROR: Error occur at abi_reader() in main.rs");

    // let url = "https://catfact.ninja/fact";
    //
    // let mut params = HashMap::new();
    // params.insert("category", "finance");
    // params.insert("https", "true");
    //
    // let json = crate::server::server::fetch_with_query(url, Some(&params), None).await.unwrap();
    // println!("{}", json);
    //
    // // Without params
    // let json2 = crate::server::server::fetch_with_query(url, None, None).await.unwrap();
    // println!("Without params:\n{}", json2);
}

