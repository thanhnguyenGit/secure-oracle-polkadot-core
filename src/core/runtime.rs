use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use anyhow::{anyhow,Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use wasmtime::{Engine, Func, Instance, Module, Store, Val, Linker, Caller, ImportType, Table, Extern, ExternType, ValType, V128, Ref};
use dynamic_struct::generate_struct;
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
    class_selector : String,
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
    classes_schema : HashMap<String,(String,Vec<Param>)>
}

pub fn abi_reader(abi_path: &str, wasm_bytecode_path: &str) -> anyhow::Result<()> {
    let file_content = fs::read_to_string(abi_path)?;
    let root: Root = serde_json::from_str(&file_content)?;

    let mut selector_registry = SelectorRegistry {
        origin: root.headers.header.clone(),
        functions: HashMap::default(),
        variables: HashMap::default(),
        classes_schema : HashMap::default(),
    };

    // assert_eq!(check_header_hash(&root.headers.header,wasm_bytecode_path), true,
    //            "The hash from {:?} is not equal to the header {:?}",abi_path,root.headers.header
    // );

    for (_,value) in root.functions.into_iter().enumerate() {
        selector_registry.functions.insert(value.selector.clone(),value);
    }
    for value in root.classes.into_iter().flat_map(|class| {
        selector_registry.classes_schema.insert(class.class_selector.clone(),(class.name,class.fields));
        class.methods.into_iter()
    }) {
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
    // generate_struct!(
    // r#"[
    //     {
    //         "name": "a",
    //         "param_type": "i32"
    //     },
    //     {
    //         "name": "b",
    //         "param_type": "i32"
    //     }
    // ]"#
    // );
    // let instance = ASCStruct { a: 42, b: 100 };
    // println!("Proc macro work a: {}, b: {}", instance.a, instance.b);

    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = Module::from_file(&engine, path)?;

    // DEBUG:
    // ------
    for (k,v) in register.functions.iter() {
        println!("Key: {k:#?} - Value: {v:#?}");
    }
    for (k,v) in register.classes_schema.iter() {
        println!("Key: {k:#?} - Value: {v:#?}");
    }
    // -----
    let mut externlts : Vec<Extern> = Vec::new();
    if module.imports().len() != 0 {
        for import in module.imports() {
            println!(
                "Import module: {:#?}, name: {:#?}, type: {:#?}",
                import.module(),
                import.name(),
                import.ty()
            );
            match import.ty() {
                ExternType::Func(func_ty) => {
                    let func = Func::new(&mut store, func_ty.clone(), |_,_,_| {
                        println!("called imported");
                        Ok(())
                    });
                    externlts.push(func.into());
                }
                ExternType::Global(global_ty) => {
                    let val = match global_ty.content() {
                        ValType::I32 => Val::I32(0),
                        ValType::I64 => Val::I64(0),
                        ValType::F32 => Val::F32(0),
                        ValType::F64 => Val::F64(0),
                        _ => unimplemented!("Unsupported global type"),
                    };
                }
                ExternType::Table(table_ty) => {
                    let table = Table::new(&mut store, table_ty, Ref::Func(None))?;
                    externlts.push(table.into());
                }
                _ => unimplemented!("Don't care bruv"),
            }
        }
    }

    let instance = Instance::new(&mut store, &module, &externlts)?;

    // READ CONSTANT VARIABLE
    // ---------------------------------------------------------------------------------------
    // Cái này để test, sau này sẽ chỉ có một entrypoint.
    // let entry_selector = "0xf212493a";

    // let trigger_func = func.call(&mut store, &typ_val_map, &mut [I32(12)]).expect("Bruh");
    // println!("Result of add(42, 58): {}", trigger_func);

    // let add_func = instance
    //     .get_typed_func::<(i32, i32), i32>(&mut store, "add")?;
    // let sum = add_func.call(&mut store, (42, 58))?;
    // println!("Result of add(42, 58): {}", sum);
    //
    // let shout_func = instance
    //     .get_typed_func::<(), i32>(&mut store, "shout")?;
    // let shout_result = shout_func.call(&mut store, ())?;
    // println!("Result of shout(): {}", shout_result);
    //
    // let url_global = instance.get_global(&mut store, "url").unwrap();
    // let url_value = url_global.get(&mut store).i32().unwrap();
    // println!("Value of url global: {}", url_value);

    // let memory = instance
    //     .get_memory(&mut store, "memory")
    //     .ok_or_else(|| anyhow::anyhow!("Failed to get memory"))?;
    // let memory_data = memory.data(&store);
    //
    // let offset_1036 = 1036;
    // let data_at_1036 = memory_data[offset_1036 as usize];
    // println!("Data at offset 1036: {:x}", data_at_1036);
    //
    // let offset_1048 = 1048;
    // print!("Raw bytes at offset 1048: ");
    // for i in 0..20 {
    //     if (offset_1048 + i) < memory_data.len() as u32 {
    //         print!("{:02x} ", memory_data[(offset_1048 + i) as usize]);
    //     }
    // }
    // println!();

    // let string_offset = 1056;
    // let mut chars = Vec::new();
    // let mut i = url_value as usize;
    // while i + 1 < memory_data.len() {
    //     let low_byte = memory_data[i];
    //     let high_byte = memory_data[i + 1];
    //     if low_byte == 0 && high_byte == 0 {
    //         break; // Null terminator
    //     }
    //     let char_code = u16::from_le_bytes([low_byte, high_byte]);
    //     chars.push(char::from_u32(char_code as u32).unwrap_or('?'));
    //     i += 2;
    // }
    // let string = chars.into_iter().collect::<String>();
    // println!("String at offset 1056: {}", string);
    // ---------------------------------------------------------------------------------------

    let memory = instance.get_memory(&mut store, "memory").expect("Memory not found");
    let alloc = instance.get_typed_func::<(i32,i32),i32>(&mut store, "__new")?;
    let process_func = instance.get_typed_func::<i32,i32>(&mut store, "process")?;


    let input_ptr = alloc.call(&mut store, (8,0))?;
    // Input schema:
    // {
    //      a : i32,
    //      b : i32
    // }
    let a : i32 = 20;
    let b : i32 = 69;
    let input_offset = input_ptr as usize;
    memory.write(&mut store, input_offset, &a.to_le_bytes())?;
    memory.write(&mut store, input_offset + 4, &b.to_le_bytes())?;

    let output_ptr = process_func.call(&mut store, input_ptr)?;
    let output_offset = output_ptr as usize;
    let mut sum_bytes = [0u8;4];
    let mut product_bytes = [0u8;4];
    memory.read(&store, output_offset, &mut sum_bytes)?;
    memory.read(&store, output_offset + 4, &mut product_bytes)?;
    // Output schema:
    // {
    //      sum : i32,
    //      product : i32
    // }
    let sum = i32::from_le_bytes(sum_bytes);
    let product = i32::from_le_bytes(product_bytes);

    println!("Output: \n Sum: {} \n, Product: {}", sum, product);
    Ok(())
}



