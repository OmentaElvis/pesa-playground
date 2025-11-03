extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, Path, Token,
};

struct CommandMapping {
    attrs: Vec<syn::Attribute>,
    command: Ident,
    args: Punctuated<syn::FnArg, Token![,]>,
    core_fn: Path,
}

impl Parse for CommandMapping {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let command: Ident = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        let args = content.parse_terminated(syn::FnArg::parse, Token![,])?;
        input.parse::<Token![=>]>()?;
        let core_fn: Path = input.parse()?;
        Ok(CommandMapping {
            attrs,
            command,
            args,
            core_fn,
        })
    }
}

struct CommandMappings {
    mappings: Punctuated<CommandMapping, Token![,]>
}

impl Parse for CommandMappings {
    fn parse(input: ParseStream) -> Result<Self> {
        let mappings = Punctuated::<CommandMapping, Token![,]>::parse_terminated(input)?;
        Ok(CommandMappings { mappings })
    }
}

#[proc_macro]
pub fn generate_tauri_wrappers(input: TokenStream) -> TokenStream {
    let CommandMappings { mappings } = parse_macro_input!(input as CommandMappings);

    let generated_functions = mappings.iter().map(|mapping| {
        let command_name = &mapping.command;
        let args = &mapping.args;
        let core_fn = &mapping.core_fn;

        let has_no_context_attr = mapping.attrs.iter().any(|attr| attr.path().is_ident("no_context"));

        let arg_names = args.iter().map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    return &pat_ident.ident;
                }
            }
            panic!("Expected identifier for argument name");
        });

        let (state_arg, state_pass) = if has_no_context_attr {
            (quote!{}, quote!{})
        } else {
            (quote!{ state: tauri::State<'_, pesa_core::AppContext>, }, quote!{ &state, })
        };

        quote! {
            #[tauri::command]
            async fn #command_name(
                #state_arg
                #args
            ) -> std::result::Result<serde_json::Value, String> {
                match #core_fn(#state_pass #(#arg_names),*).await {
                    Ok(value) => serde_json::to_value(value).map_err(|e| e.to_string()),
                    Err(err) => Err(err.to_string()),
                }
            }
        }
    });

    let expanded = quote! {
        #(#generated_functions)*
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn generate_axum_rpc_handler(input: TokenStream) -> TokenStream {
    let CommandMappings { mappings } = parse_macro_input!(input as CommandMappings);

    let match_arms = mappings.iter().map(|mapping| {
        let command_name_str = mapping.command.to_string();
        let core_fn = &mapping.core_fn;
        let args = &mapping.args;

        // Create a tuple of the argument types for deserialization
        let arg_types: Vec<_> = args.iter().map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                &pat_type.ty
            } else {
                panic!("Unsupported argument type");
            }
        }).collect();
        let arg_type_tuple = quote! { (#(#arg_types,)*) };

        // Create a list of argument names for the function call
        let arg_names: Vec<_> = (0..arg_types.len()).map(|i| {
            let index = syn::Index::from(i);
            quote! { p.#index }
        }).collect();

        let has_no_context_attr = mapping.attrs.iter().any(|attr| attr.path().is_ident("no_context"));

        let function_call = if has_no_context_attr {
            quote! { #core_fn(#(#arg_names),*) }
        } else {
            quote! { #core_fn(&state.core_context, #(#arg_names),*) }
        };

        quote! {
            #command_name_str => {
                match serde_json::from_value::<#arg_type_tuple>(payload.params) {
                    Ok(p) => {
                        match #function_call.await {
                            Ok(data) => {
                                let result = serde_json::to_value(data).unwrap();
                                response = serde_json::json!({"jsonrpc": "2.0", "result": result, "id": payload.id});
                            },
                            Err(e) => {
                                response = serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32000, "message": e.to_string()}, "id": payload.id});
                            }
                        }
                    },
                    Err(e) => {
                        response = serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32700, "message": format!("Invalid params: {}", e)}, "id": payload.id});
                    }
                }
            }
        }
    });

    let expanded = quote! {
        #[derive(serde::Deserialize)]
        pub struct RpcRequest {
            jsonrpc: String,
            id: serde_json::Value,
            method: String,
            params: serde_json::Value,
        }

        pub async fn rpc_handler(
            axum::extract::State(state): axum::extract::State<AxumAppState>,
            axum::Json(payload): axum::Json<RpcRequest>,
        ) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
            let mut response = serde_json::json!({});
            match payload.method.as_str() {
                #(#match_arms)*
                _ => {
                    response = serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32601, "message": "Method not found"}, "id": payload.id});
                }
            }
            (axum::http::StatusCode::OK, axum::Json(response))
        }
    };

    TokenStream::from(expanded)
}
