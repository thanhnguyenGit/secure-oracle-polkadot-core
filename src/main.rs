mod abi_parser;
mod regex_str;

use wasmtime::*;
use anyhow::Result;
use crate::abi_parser::parser;

fn main() {
    parser().expect("INFO: error at abi_parser.rs file");
    // wasmtime_runner().expect("INFO: error at wastime_runner() in main.rs");
    // abi_reader("./orascripts/addTwoABI.json").unwrap()
}

use serde::{Deserialize};
use std::fs;

#[derive(Debug, Deserialize)]
struct Root {
    functions: Vec<Function>,
    classes: Vec<Class>,
    variables: Vec<Variable>,
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
    // Add class fields when needed
}

#[derive(Debug, Deserialize)]
struct Variable {
    name: String,
    #[serde(rename = "type")]
    var_type: String,
    selector: String,
}

// trait AbiType {
//     fn get_
// }

fn abi_reader(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string(path)?;
    let root: Root = serde_json::from_str(&file_content)?;
    let target_selector = "0xf212493a";

    if let Some(func) = root.functions.iter().find(|f| f.selector == target_selector) {
        println!("Function with selector {}: {:#?}", target_selector, func);
    } else if let Some(var) = root.variables.iter().find(|v| v.selector == target_selector) {
        println!("Variable with selector {}: {:#?}", target_selector, var);
    } else {
        println!("Selector {} not found", target_selector);
    }
    Ok(())
}


fn wasmtime_runner() -> Result<()> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = Module::from_file(&engine, "./orascripts/addTwo.wasm")?;
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