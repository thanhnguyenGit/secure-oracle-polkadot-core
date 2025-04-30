use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::{Read, Write};
use regex::Regex;
use clap::Parser as ClapParser;
use sha2::{Sha256, Digest};
use hex;
use std::path::Path;
use std::process::Command;
use wasmparser::{Parser, Payload, ImportSectionReader, TypeRef};

#[derive(ClapParser, Debug)]
#[command(about = "Generate JSON ABI from an AssemblyScript .ts file")]
struct Args {
    input: String,
    output: String,
}

#[derive(Serialize, Deserialize)]
struct Abi {
    headers: AbiHeader,
    functions: Vec<AbiFunction>,
    classes: Vec<AbiClass>,
    variables: Vec<AbiVariable>,
    imports: Vec<AbiImport>
}

#[derive(Serialize,Deserialize)]
struct AbiImport {
    module: String,
    name: String,
    kind: ImportKind
}

#[derive(Serialize,Deserialize)]
enum ImportKind {
    Function {
        params: Vec<String>,
        result: Option<String>,
    },
    Memory {
        min: u32,
        max: Option<u32>,
    },
    Global {
        type_: String,
        mutable: bool,
    },
    Table {
        type_: String,
        min: u32,
        max: Option<u32>,
    },
}

#[derive(Serialize,Deserialize,Default)]
struct AbiHeader {
    name: Option<String>,
    header: String
}

#[derive(Serialize, Deserialize)]
struct AbiFunction {
    name: String,
    params: Vec<AbiParam>,
    result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<String>,
    selector: String,
}

#[derive(Serialize, Deserialize)]
struct AbiParam {
    name: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize, Deserialize)]
struct AbiClass {
    class_selector: String,
    name: String,
    fields: Vec<AbiField>,
    methods: Vec<AbiFunction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AbiField {
    name: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize, Deserialize)]
struct AbiVariable {
    name: String,
    #[serde(rename = "type")]
    type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<String>,
    selector: String,
}

fn validate_args(args: &Args) -> Result<()> {
    let input_path = Path::new(&args.input);
    if !input_path.exists() {
        return Err(anyhow!("Input file '{}' does not exist", args.input));
    }
    if input_path.extension().and_then(|ext| ext.to_str()) != Some("ts") {
        return Err(anyhow!("Input file '{}' must have .ts extension", args.input));
    }
    let output_path = Path::new(&args.output);
    if output_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
        return Err(anyhow!("Output file '{}' must have .json extension", args.output));
    }
    Ok(())
}

fn convert_ts_to_wasm(input: &str, wasm_output: &str) -> Result<()> {
    let output = Command::new("asc")
        .arg(input)
        .arg("--outFile")
        .arg(wasm_output)
        .arg("--optimize")
        .output()
        .map_err(|err| anyhow!("Failed to execute 'asc' command, error at: {}", err))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("asc - AssemblyScript Compiler failed: {}", stderr))
    }
    Ok(())
}

fn compute_selector(name: &str, params: &[AbiParam]) -> String {
    let signature = if params.is_empty() {
        format!("{}()", name)
    } else {
        let param_names = params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(",");
        format!("{}({})", name, param_names)
    };
    let mut hasher = Sha256::new();
    hasher.update(signature);
    let result = hasher.finalize();
    format!("0x{}", hex::encode(&result[..4]))
}

pub fn abi_parser() -> Result<()> {
    let args = Args::parse();
    validate_args(&args)?;
    let wasm_output = args.input.replace(".ts", ".wasm");
    convert_ts_to_wasm(&args.input, &wasm_output)?;

    let wasm_hash = {
        let mut header_hasher = Sha256::new();
        let mut wasm_reader = File::open(&wasm_output)
            .map_err(|e| anyhow!("ERROR: failed to read WASM file: {}", e))?;
        let mut wasm_content = Vec::new();
        wasm_reader.read_to_end(&mut wasm_content)
            .map_err(|e| anyhow!("ERROR: failed to parse the wasm content to variable: {}", e))?;
        header_hasher.update(&wasm_content);
        let hash = header_hasher.finalize();
        format!("0x{:x}", hash)
    };

    let mut file = File::open(&args.input)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut abi = Abi {
        headers: Default::default(),
        functions: Vec::new(),
        classes: Vec::new(),
        variables: Vec::new(),
        imports: Vec::new(),
    };

    let func_re = Regex::new(r"export\s+function\s+(\w+)\s*\((.*?)\)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)\s*\{")?;
    let class_re = Regex::new(r"class\s+(\w+)\s*\{")?;
    let class_field_re = Regex::new(r"(\w+)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)\s*;")?;
    let constructor_re = Regex::new(r"constructor\s*\((.*?)\)\s*\{")?;
    let method_re = Regex::new(r"(\w+)\s*\((.*?)\)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)\s*\{")?;
    let param_re = Regex::new(r"(public\s+)?(\w+)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)")?;
    let var_re = Regex::new(r"export\s+const\s+(\w+)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)\s*=\s*[^;]+;")?;
    let doc_re = Regex::new(r"/\*\*\s*(.*?)\s*\*/")?;

    let lines: Vec<&str> = content.lines().collect();
    let mut current_class: Option<AbiClass> = None;
    let mut last_doc: Option<String> = None;

    for (_, line) in lines.iter().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(captures) = doc_re.captures(line) {
            last_doc = Some(captures[1].trim().to_string());
            continue;
        }
        if let Some(captures) = var_re.captures(line) {
            let name = captures[1].to_string();
            let type_ = captures[2].to_string();
            let selector = compute_selector(&name, &[]);
            abi.variables.push(AbiVariable {
                name,
                type_,
                doc: last_doc.take(),
                selector,
            });
            continue;
        }
        if let Some(captures) = func_re.captures(line) {
            let name = captures[1].to_string();
            let params_str = captures[2].trim();
            let result = captures[3].to_string();

            let mut params = Vec::new();
            if !params_str.is_empty() {
                for param in params_str.split(',').map(|p| p.trim()) {
                    if let Some(param_cap) = param_re.captures(param) {
                        let param_name = param_cap[2].to_string();
                        let param_type = param_cap[3].to_string();
                        params.push(AbiParam {
                            name: param_name,
                            type_: param_type,
                        });
                    }
                }
            }

            let selector = compute_selector(&name, &params);
            abi.functions.push(AbiFunction {
                name,
                params,
                result,
                doc: last_doc.take(),
                selector,
            });
            continue;
        }
        if let Some(captures) = class_re.captures(line) {
            let class_name = captures[1].to_string();
            current_class = Some(AbiClass {
                class_selector: compute_selector(&class_name,&[]),
                name: class_name,
                fields: Vec::new(),
                methods: Vec::new(),
                doc: last_doc.take(),
            });
            continue;
        }
        if let Some(ref mut class) = current_class {
            if let Some(captures) = constructor_re.captures(line) {
                let params_str = captures[1].trim();
                if !params_str.is_empty() {
                    for param in params_str.split(',').map(|p| p.trim()) {
                        if let Some(param_cap) = param_re.captures(param) {
                            if param_cap.get(1).is_some() { // Public fields
                                let field_name = param_cap[2].to_string();
                                let field_type = param_cap[3].to_string();
                                class.fields.push(AbiField {
                                    name: field_name,
                                    type_: field_type,
                                });
                            }
                        }
                    }
                }
                continue;
            }
            if let Some(captures) = class_field_re.captures(line) {
                let field_name = captures[1].to_string();
                let field_type = captures[2].to_string();
                class.fields.push(AbiField {
                    name: field_name,
                    type_: field_type,
                });
                continue;
            }
            if let Some(captures) = method_re.captures(line) {
                let method_name = captures[1].to_string();
                let params_str = captures[2].trim();
                let result = captures[3].to_string();

                let mut params = Vec::new();
                if !params_str.is_empty() {
                    for param in params_str.split(',').map(|p| p.trim()) {
                        if let Some(param_cap) = param_re.captures(param) {
                            let param_name = param_cap[2].to_string();
                            let param_type = param_cap[3].to_string();
                            params.push(AbiParam {
                                name: param_name,
                                type_: param_type,
                            });
                        }
                    }
                }
                let selector = compute_selector(&method_name, &params);
                class.methods.push(AbiFunction {
                    name: method_name,
                    params,
                    result,
                    doc: last_doc.take(),
                    selector,
                });
                continue;
            }
            if line == "}" {
                abi.classes.push(current_class.take().unwrap());
                continue;
            }
        }
        last_doc = None;
    }

    abi.headers = AbiHeader {
        name: None,
        header: wasm_hash,
    };

    let json = serde_json::to_string_pretty(&abi)?;

    let mut file = File::create(&args.output)?;
    file.write_all(json.as_bytes())?;
    println!("Generated ABI written to {}", args.output);

    Ok(())
}
