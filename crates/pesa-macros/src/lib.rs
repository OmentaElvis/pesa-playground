extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, Path, Token, Type,
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

struct TauriCommandMappings {
    app_state_type: Type,
    _comma: Token![,],
    mappings: Punctuated<CommandMapping, Token![,]>,
}

impl Parse for TauriCommandMappings {
    fn parse(input: ParseStream) -> Result<Self> {
        let app_state_type: Type = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let mappings = Punctuated::<CommandMapping, Token![,]>::parse_terminated(input)?;
        Ok(TauriCommandMappings {
            app_state_type,
            _comma,
            mappings,
        })
    }
}

struct AxumRpcHandler {
    fn_name: Ident,
    _comma: Token![,],
    mappings: Punctuated<CommandMapping, Token![,]>,
}

impl Parse for AxumRpcHandler {
    fn parse(input: ParseStream) -> Result<Self> {
        let fn_name: Ident = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let mappings = Punctuated::<CommandMapping, Token![,]>::parse_terminated(input)?;
        Ok(AxumRpcHandler {
            fn_name,
            _comma,
            mappings,
        })
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
    let TauriCommandMappings {
        app_state_type,
        mappings,
        ..
    } = parse_macro_input!(input as TauriCommandMappings);

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
                quote! { state: tauri::State<'_, #app_state_type>, },
                quote! { &state.context, },
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
                    Err(err) => Err(format!("{:?}", err)),
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
    let AxumRpcHandler {
        fn_name, mappings, ..
    } = parse_macro_input!(input as AxumRpcHandler);

    let match_arms = mappings.iter().map(|mapping| {
        let command_name_str = mapping.command.to_string();
        let core_fn = &mapping.core_fn;
        let args = &mapping.args;
        let has_no_context_attr = mapping
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("no_context"));

        let arg_fields: Vec<_> = args
            .iter()
            .map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    let pat = &pat_type.pat;
                    let ty = &pat_type.ty;
                    quote! { pub #pat: #ty }
                } else {
                    panic!("Unsupported argument type");
                }
            })
            .collect();

        let arg_names: Vec<_> = args
            .iter()
            .map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        return &pat_ident.ident;
                    }
                }
                panic!("Expected identifier for argument name");
            })
            .collect();

        let struct_name = syn::Ident::new(
            &format!("{}RpcArgs", to_camel_case(&command_name_str)),
            mapping.command.span(),
        );
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
            quote! {}
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

                let params_val = payload.params.clone().unwrap_or(serde_json::Value::Null);
                let call_result: std::result::Result<serde_json::Value, anyhow::Error> = async {
                    #parse_p_tokens

                    let res = match #function_call.await {
                      Ok(val) => val,
                      Err(e) => {
                          return Err(anyhow::anyhow!(e));
                      },
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
                        log::error!(target: "core", "{:?}", e);
                        status_code = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
                        response = serde_json::json!(
                            {"jsonrpc": "2.0", "error": {"code": -32700, "message": format!("Invalid params: {:?}", e)}, "id": payload.id}
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

        pub async fn #fn_name(
            axum::extract::State(state): axum::extract::State<AxumAppState>,
            axum::Json(payload): axum::Json<RpcRequest>,
        ) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
            let mut response = serde_json::json!({});
            let mut status_code = axum::http::StatusCode::OK;

            match payload.method.as_str() {
                #(#match_arms)*
                _ => {
                    status_code = axum::http::StatusCode::NOT_FOUND;
                    response = serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32601, "message": "Method not found"}, "id": payload.id});
                }
            }
            (status_code, axum::Json(response))
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn generate_lua_bindings(input: TokenStream) -> TokenStream {
    let CommandMappings { mappings } = parse_macro_input!(input as CommandMappings);

    let registration_logic = mappings.iter().map(|mapping| {
        let command_name_str = mapping.command.to_string();
        let core_fn = &mapping.core_fn;
        let args = &mapping.args;
        let has_no_context_attr = mapping.attrs.iter().any(|attr| attr.path().is_ident("no_context"));

        let arg_names: Vec<_> = args.iter().map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    return &pat_ident.ident;
                }
            }
            panic!("Expected identifier for argument name");
        }).collect();

        let arg_types: Vec<_> = args.iter().map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                return &pat_type.ty;
            }
            panic!("Expected type for argument");
        }).collect();

        let call_args = args.iter().map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                let pat = &pat_type.pat;
                // Check for our `#[wrap]` attribute
                let has_wrap_attr = pat_type.attrs.iter().any(|attr| attr.path().is_ident("wrap"));
                if has_wrap_attr {
                    return quote! { #pat.0 };
                }
                return quote! { #pat };
            }
            panic!("Unsupported arg type");
        });

        let closure_args = quote! { (#(#arg_names),*): (#(#arg_types),*) };

        let function_call = if has_no_context_attr {
            quote! { #core_fn(#(#call_args),*).await }
        } else {
            quote! {
                {
                    let app_context = match get_app_context_from_lua(&lua) {
                        Ok(ctx) => ctx,
                        Err(e) => return Err(::mlua::Error::external(format!("Failed to get AppContext: {}", e))),
                    };
                    #core_fn(&app_context, #(#call_args),*).await
                }
            }
        };

        quote! {
            {
                let func = lua.create_async_function(|lua, #closure_args| async move {
                    let result = #function_call;

                    match result {
                        Ok(value) => {
                            match lua.to_value(&value) {
                                Ok(lua_value) => Ok(lua_value),
                                Err(e) => Err(::mlua::Error::external(format!("Failed to serialize return value: {}", e))),
                            }
                        },
                        Err(e) => Err(::mlua::Error::external(e)),
                    }
                })?;
                pesa_table.set(#command_name_str, func)?;
            }
        }
    });

    let expanded = quote! {
        fn register_lua_bindings(lua: &::mlua::Lua) -> ::mlua::Result<()> {
            let globals = lua.globals();
            let pesa_table: ::mlua::Table = globals.get("pesa")?;

            #(#registration_logic)*

            Ok(())
        }
    };

    TokenStream::from(expanded)
}

struct TypeWrapping {
    struct_name: Ident,
    core_path: Path,
}

impl Parse for TypeWrapping {
    fn parse(input: ParseStream) -> Result<Self> {
        let struct_name: Ident = input.parse()?;
        let from_keyword: Ident = input.parse()?; // Parse 'from' as an Ident
        if from_keyword != "from" {
            return Err(syn::Error::new(
                from_keyword.span(),
                "expected `from` keyword",
            ));
        }
        let core_path: Path = input.parse()?;
        Ok(TypeWrapping {
            struct_name,
            core_path,
        })
    }
}

struct TypeWrappings {
    mappings: Punctuated<TypeWrapping, Token![,]>,
}

impl Parse for TypeWrappings {
    fn parse(input: ParseStream) -> Result<Self> {
        let mappings = Punctuated::<TypeWrapping, Token![,]>::parse_terminated(input)?;
        Ok(TypeWrappings { mappings })
    }
}

#[proc_macro]
pub fn wrap_core_types(input: TokenStream) -> TokenStream {
    let TypeWrappings { mappings } = parse_macro_input!(input as TypeWrappings);

    let generated_wrappers = mappings.iter().map(|mapping| {
        let struct_name = &mapping.struct_name;
        let core_path = &mapping.core_path;
        // Use the provided struct_name directly for the inner type
        // let core_struct_name = &core_path.segments.last().unwrap().ident; // This line was incorrect

        quote! {
            #[derive(serde::Deserialize)]
            pub struct #struct_name(pub #core_path::#struct_name); // Corrected path construction

            impl ::mlua::FromLua for #struct_name {
                fn from_lua(value: ::mlua::Value, lua: &::mlua::Lua) -> ::mlua::Result<Self> {
                    use ::mlua::LuaSerdeExt; // Import the trait here
                    Ok(Self(lua.from_value(value)?))
                }
            }
        }
    });

    let expanded = quote! {
        #(#generated_wrappers)*
    };

    TokenStream::from(expanded)
}
