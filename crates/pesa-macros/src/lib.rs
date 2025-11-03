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
    mappings: Punctuated<CommandMapping, Token![,]>,
}

impl Parse for CommandMappings {
    fn parse(input: ParseStream) -> Result<Self> {
        let mappings = Punctuated::<CommandMapping, Token![,]>::parse_terminated(input)?;
        Ok(CommandMappings { mappings })
    }
}

fn to_camel_case(s: &str) -> String {
    s.split('_')
        .filter(|seg| !seg.is_empty())
        .map(|seg| {
            let mut c = seg.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<String>()
}

#[proc_macro]
pub fn generate_tauri_wrappers(input: TokenStream) -> TokenStream {
    let CommandMappings { mappings } = parse_macro_input!(input as CommandMappings);

    let generated_functions = mappings.iter().map(|mapping| {
        let command_name = &mapping.command;
        let args = &mapping.args;
        let core_fn = &mapping.core_fn;

        let has_no_context_attr = mapping
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("no_context"));

        let arg_names = args.iter().map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    return &pat_ident.ident;
                }
            }
            panic!("Expected identifier for argument name");
        });

        let (state_arg, state_pass) = if has_no_context_attr {
            (quote! {}, quote! {})
        } else {
            (
                quote! { state: tauri::State<'_, pesa_core::AppContext>, },
                quote! { &state, },
            )
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
            let has_no_context_attr = mapping.attrs.iter().any(|attr| attr.path().is_ident("no_context"));

            let arg_fields: Vec<_> = args.iter().map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    let pat = &pat_type.pat;
                    let ty = &pat_type.ty;
                    quote! { pub #pat: #ty }
                } else {
                    panic!("Unsupported argument type");
                }
            }).collect();

            let arg_names: Vec<_> = args.iter().map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        return &pat_ident.ident;
                    }
                }
                panic!("Expected identifier for argument name");
            }).collect();

            let struct_name = syn::Ident::new(&format!("{}RpcArgs", to_camel_case(&command_name_str)), mapping.command.span());
            let struct_def = if arg_fields.is_empty() {
                quote! {}
            } else {
                quote! {
                    #[derive(serde::Deserialize)]
                    #[serde(rename_all = "camelCase")]
                    struct #struct_name {
                        #(#arg_fields),*
                    }
                }
            };

            
            let function_call_args = if arg_fields.is_empty() {
                quote! { }
            } else {
                quote! { #(p.#arg_names),* }
            };

            
            let function_call = if has_no_context_attr {
                quote! { #core_fn(#function_call_args) }
            } else {
                quote! { #core_fn(&state.core_context, #function_call_args) }
            };

            let parse_p_tokens = if arg_fields.is_empty() {
                quote! {}
            } else {
                quote! {
                    let p = serde_json::from_value::<#struct_name>(params_val)?;
                }
            };

            
            quote! {
                #command_name_str => {
                    #struct_def
                
                    let params_val = payload.params.unwrap_or(serde_json::Value::Null);
                    let call_result: std::result::Result<serde_json::Value, anyhow::Error> = async move {
                        #parse_p_tokens
                        
                        let res = match #function_call.await {
                          Ok(val) => val,
                          Err(e) => return Err(anyhow::anyhow!(e.to_string())),
                        };
                        Ok(serde_json::to_value(res)?)
                    }.await;
                    match call_result {
                        Ok(data) => {
                            response = serde_json::json!(
                                {"jsonrpc": "2.0", "result": data, "id": payload.id}
                            );
                        },
                        Err(e) => {
                            status_code = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
                            response = serde_json::json!(
                                {"jsonrpc": "2.0", "error": {"code": -32700, "message": format!("Invalid params: {}", e)}, "id": payload.id}
                            );
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
            params: Option<serde_json::Value>,
        }

        pub async fn rpc_handler(
            axum::extract::State(state): axum::extract::State<AxumAppState>,
            axum::Json(payload): axum::Json<RpcRequest>,
        ) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
            let mut response = serde_json::json!({});
            let mut status_code = axum::http::StatusCode::OK;
            
            match payload.method.as_str() {
                #(#match_arms)*
                _ => {
                    response = serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32601, "message": "Method not found"}, "id": payload.id});
                }
            }
            (status_code, axum::Json(response))
        }
    };

    TokenStream::from(expanded)
}
