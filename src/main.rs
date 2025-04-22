mod core;
mod traits;
mod utils;

use std::collections::HashMap;
use std::fmt::format;
use wasmtime::*;
use anyhow::{anyhow, Result};
use serde::{Deserialize};
use std::fs;
use std::fs::File;
use std::io::Read;
use serde_json::json;
use sha2::{Sha256, Digest};
use crate::core::abi_parser::parser;
use crate::traits::traits::ABIType;

#[derive(Debug, Deserialize)]
struct Root {
    headers : Header,
    functions: Vec<Function>,
    classes: Vec<Class>,
    variables: Vec<Variable>,
}

#[derive(Debug,Deserialize)]
struct Header {
    name : Option<String>,
    header : String,
}

#[derive(Debug, Deserialize)]
struct Function {
    name: String,
    params: Vec<Param>,
    result: String,
    selector: String,
}

#[derive(Debug, Deserialize)]
struct Param {
    name: String,
    #[serde(rename = "type")]
    param_type: String,
}

#[derive(Debug, Deserialize)]
struct Class {
    name: String,
    fields : Vec<Param>,
    methods: Vec<Function>
}

#[derive(Debug, Deserialize)]
struct Variable {
    name: String,
    #[serde(rename = "type")]
    var_type: String,
    selector: String,
}

impl ABIType for Function {}
impl ABIType for Class {}
impl ABIType for Param {}
impl ABIType for Variable {}

#[derive(Default)]
struct SelectorRegistry {
    origin : String,
    storage : HashMap<String,Box<dyn ABIType>>,
}
fn main() {
    // parser().expect("INFO: error at abi_parser.rs file");
    abi_reader("./orascripts/orascriptABI.json","./orascripts/orascript.wasm").expect("ERROR: Error occur at abi_reader() in main.rs");
    // wasmtime_runner().expect("INFO: error at wastime_runner() in main.rs");
    // abi_reader("./orascripts/addTwoABI.json").unwrap()
}

fn check_header_hash(header : &str) -> bool {
    let wasm_hash = {
        let mut header_hasher = Sha256::new();
        let mut wasm_reader = File::open("./orascripts/orascript.wasm")
            .map_err(|e| anyhow!("ERROR: failed to read WASM file: {}", e)).expect("Re check bro 1");
        let mut wasm_content = Vec::new();
        wasm_reader.read_to_end(&mut wasm_content)
            .map_err(|e| anyhow!("ERROR: failed to parse the wasm content to variable: {}", e)).expect("Re check bro 2");
        header_hasher.update(&wasm_content);
        let hash = header_hasher.finalize();
        format!("0x{:x}", hash)
    };
    header.eq(&wasm_hash)
}

fn abi_reader(abi_path: &str, wasm_bytecode_path: &str) -> Result<()> {
    let file_content = fs::read_to_string(abi_path)?;
    let root: Root = serde_json::from_str(&file_content)?;

    let mut selector_registry = SelectorRegistry {
        origin: root.headers.header.clone(),
        storage: Default::default(),
    };

    assert_eq!(check_header_hash(&root.headers.header), true,
               "The hash from {:?} is not equal to the header {:?}",abi_path,root.headers.header
    );

    let mut insert_selector = |sel, abi_item| {
        selector_registry.storage.insert(sel, abi_item);
    };

    for (_,value) in root.functions.into_iter().enumerate() {
        insert_selector(value.selector.clone(),Box::new(value));
    }
    for value in root.classes.into_iter().flat_map(|class| class.methods.into_iter()) {
        insert_selector(value.selector.clone(),Box::new(value));
    }
    for (_,value) in root.variables.into_iter().enumerate() {
        insert_selector(value.selector.clone(),Box::new(value));
    }

    for (key,value) in selector_registry.storage.iter() {
        println!("Selector: {:#?} - Meta: {:#?}", key,value);
    }
    let _ = wasmtime_runner(wasm_bytecode_path,selector_registry);
    Ok(())
}


fn wasmtime_runner(path: &str,register : SelectorRegistry) -> Result<()> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = Module::from_file(&engine, path)?;
    let instance = Instance::new(&mut store, &module, &[])?;

    let add_func = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "add")?;
    let sum = add_func.call(&mut store, (42, 58))?;
    println!("Result of add(42, 58): {}", sum);

    let shout_func = instance
        .get_typed_func::<(), i32>(&mut store, "shout")?;
    let shout_result = shout_func.call(&mut store, ())?;
    println!("Result of shout(): {}", shout_result);

    let url_global = instance.get_global(&mut store, "url").unwrap();
    let url_value = url_global.get(&mut store).i32().unwrap();
    println!("Value of url global: {}", url_value);

    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| anyhow::anyhow!("Failed to get memory"))?;
    let memory_data = memory.data(&store);

    let offset_1036 = 1036;
    let data_at_1036 = memory_data[offset_1036 as usize];
    println!("Data at offset 1036: {:x}", data_at_1036);

    let offset_1048 = 1048;
    print!("Raw bytes at offset 1048: ");
    for i in 0..20 {
        if (offset_1048 + i) < memory_data.len() as u32 {
            print!("{:02x} ", memory_data[(offset_1048 + i) as usize]);
        }
    }
    println!();

    let string_offset = 1056;
    let mut chars = Vec::new();
    let mut i = url_value as usize;
    while i + 1 < memory_data.len() {
        let low_byte = memory_data[i];
        let high_byte = memory_data[i + 1];
        if low_byte == 0 && high_byte == 0 {
            break; // Null terminator
        }
        let char_code = u16::from_le_bytes([low_byte, high_byte]);
        chars.push(char::from_u32(char_code as u32).unwrap_or('?'));
        i += 2;
    }
    let string = chars.into_iter().collect::<String>();
    println!("String at offset 1056: {}", string);

    Ok(())
}