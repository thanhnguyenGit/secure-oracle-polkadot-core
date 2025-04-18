mod abi_parser;

use wasmtime::*;
use anyhow::Result;
use crate::abi_parser::parser;

fn main() {
    parser().expect("INFO: error at abi_parser.rs file");
}

fn wasmtime_runner() -> Result<()> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = Module::from_file(&engine, "./addTwo.wasm")?;
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
    let mut i = string_offset as usize;
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