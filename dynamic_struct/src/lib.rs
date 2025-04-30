use proc_macro::TokenStream;
use quote::quote;
use serde_json::from_str;
use syn::{parse_macro_input, LitStr};
use serde::Deserialize;

#[derive(Deserialize)]
struct StructField {
    name: String,
    typ: String,
}

#[proc_macro]
pub fn generate_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let json_str = input.value();

    let fields: Vec<StructField> = match from_str(&json_str) {
        Ok(fields) => fields,
        Err(e) => {
            return syn::Error::new(
                input.span(),
                format!("ERROR: Failed to parse JSON schema: {}", e),
            )
                .to_compile_error()
                .into();
        }
    };

    let fields_typ = fields.iter().map(|field| {
        let name = syn::Ident::new(&field.name, input.span());
        let typ: syn::Type = match field.typ.as_str() {
            "i32" => syn::parse_quote! { i32 },
            "i64" => syn::parse_quote! { i64 },
            "f32" => syn::parse_quote! { f32 },
            "f64" => syn::parse_quote! { f64 },
            "string" => syn::parse_quote! { String },
            _ => syn::parse_quote! { () },
        };
        quote! {
            pub #name: #typ
        }
    });

    let struct_name = syn::Ident::new("ASCStruct", input.span());
    let output = quote! {
        pub struct #struct_name {
            #(#fields_typ),*
        }
    };

    output.into()
}