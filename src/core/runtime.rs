use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use anyhow::{anyhow,Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use wasmtime::{Engine, Func, Instance, Module, Store, Val, Linker, Caller, ImportType, Extern};

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

#[derive(Default)]
struct SelectorRegistry {
    origin : String,
    functions : HashMap<String,Function>,
    variables : HashMap<String,Variable>,
}

pub fn abi_reader(abi_path: &str, wasm_bytecode_path: &str) -> anyhow::Result<()> {
    let file_content = fs::read_to_string(abi_path)?;
    let root: Root = serde_json::from_str(&file_content)?;

    let mut selector_registry = SelectorRegistry {
        origin: root.headers.header.clone(),
        functions: HashMap::default(),
        variables: HashMap::default(),
    };

    assert_eq!(check_header_hash(&root.headers.header,wasm_bytecode_path), true,
               "The hash from {:?} is not equal to the header {:?}",abi_path,root.headers.header
    );

    for (_,value) in root.functions.into_iter().enumerate() {
        selector_registry.functions.insert(value.selector.clone(),value);
    }
    for value in root.classes.into_iter().flat_map(|class| class.methods.into_iter()) {
        selector_registry.functions.insert(value.selector.clone(),value);
    }
    for (_,value) in root.variables.into_iter().enumerate() {
        selector_registry.variables.insert(value.selector.clone(),value);
    }
    wasmtime_runner(wasm_bytecode_path,selector_registry).expect("DEBUG: wasmtime runner run");
    Ok(())
}

fn check_header_hash(header : &str,wasm_bytecode_path: &str) -> bool {
    let wasm_hash = {
        let mut header_hasher = Sha256::new();
        let mut wasm_reader = File::open(wasm_bytecode_path)
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

fn wasmtime_runner(path: &str,register : SelectorRegistry) -> anyhow::Result<()> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = Module::from_file(&engine, path)?;

    // let mut linker = Linker::new(&engine);
    // linker.func_wrap(
    //     "env",
    //     "abort",
    //     |caller: Caller<'_, ()>, ptr: i32, file: i32, line: i32, col: i32| {
    //         println!("Abort called: ptr={}, file={}, line={}, col={}", ptr, file, line, col);
    //         std::process::exit(1);
    //     },
    // )?;

    if module.imports().len() != 0 {
        for import_type in module.imports() {
            println!("Import module: {:#?} - Import name: {:#?}", import_type.module(), import_type.name());
        }
    }
    for i in module.exports() {
        println!("Module export type: {:#?}", i);
    }

    println!("Module name: {:#?}", module.name());

    let instance = Instance::new(&mut store, &module, &[])?;

    // Cái này để test, sau này sẽ chỉ có một entrypoint.
    let entry_selector = "0xf212493a";

    let meta = register.functions.iter().find_map(|(selector,meta)| if *selector == entry_selector.to_string() {
        Some(meta)
    } else {
        None
    }).expect("DEBUG: expect a function");

    // Cái này để test, vì sau này các value sẽ được fetch từ JSON
    // nên sẽ có định dạng string sau khi parse
    let arg_values = vec!["69", "425"];

    let args_types : Vec<String> = meta.params.iter().map(|param| param.param_type.to_owned()).collect();

    assert_eq!(arg_values.len(),args_types.len());

    let typ_val_map : Vec<Val>= args_types.iter().zip(arg_values.iter()).map(|(typ,val)| {
        map_type(typ,*val).expect("Invalid arg")
    }).collect();

    for i in &typ_val_map {
        println!("DEBUG: ele in typ_val_map: {i:?}")
    }

    let func = instance.get_func(&mut store, meta.name.as_ref()).expect("Function name not exist");
    
    // let trigger_func = func.call(&mut store, &typ_val_map, &mut [I32(12)]).expect("Bruh");
    // println!("Result of add(42, 58): {}", trigger_func);
    
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
fn map_type(abi_type: &str, arg_value: &str) -> Option<Val> {
    match abi_type {
        "i32" => arg_value.parse::<i32>().ok().map(Val::I32),
        "i64" => arg_value.parse::<i64>().ok().map(Val::I64),
        "f32" => arg_value.parse::<u32>().ok().map(Val::F32),
        "f64" => arg_value.parse::<u64>().ok().map(Val::F64),
        _ => None
    }
}