use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::{Read, Write};
use regex::Regex;
use clap::Parser;
use sha2::{Sha256, Digest};
use hex;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "./addTwo.ts")]
    input: String,
    #[clap(short, long, default_value = "abi_addTwo.json")]
    output: String,
}

#[derive(Serialize, Deserialize)]
struct Abi {
    functions: Vec<AbiFunction>,
    classes: Vec<AbiClass>,
    variables: Vec<AbiVariable>,
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

pub fn parser() -> Result<()> {
    let args = Args::parse();

    let mut file = File::open(&args.input)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Initialize ABI
    let mut abi = Abi {
        functions: Vec::new(),
        classes: Vec::new(),
        variables: Vec::new(),
    };
    let func_re = Regex::new(r"export\s+function\s+(\w+)\s*\((.*?)\)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)\s*\{")?;
    let class_re = Regex::new(r"export\s+class\s+(\w+)\s*\{")?;
    let constructor_re = Regex::new(r"constructor\s*\((.*?)\)\s*\{")?;
    let method_re = Regex::new(r"(\w+)\s*\((.*?)\)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)\s*\{")?;
    let param_re = Regex::new(r"(public\s+)?(\w+)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)")?;
    let var_re = Regex::new(r"export\s+const\s+(\w+)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)\s*=\s*[^;]+;")?;
    let doc_re = Regex::new(r"/\*\*\s*(.*?)\s*\*/")?;

    let lines: Vec<&str> = content.lines().collect();
    let mut current_class: Option<AbiClass> = None;
    let mut last_doc: Option<String> = None;

    for (i, line) in lines.iter().enumerate() {
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

        // Parse exported classes
        if let Some(captures) = class_re.captures(line) {
            let class_name = captures[1].to_string();
            current_class = Some(AbiClass {
                name: class_name,
                fields: Vec::new(),
                methods: Vec::new(),
                doc: last_doc.take(),
            });
            continue;
        }

        // Parse constructor for fields
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

    let json = serde_json::to_string_pretty(&abi)?;
    let mut file = File::create(&args.output)?;
    file.write_all(json.as_bytes())?;

    println!("Generated ABI written to {}", args.output);
    Ok(())
}