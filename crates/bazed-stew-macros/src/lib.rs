#![feature(iter_intersperse)]

use proc_macro::TokenStream;
use proc_macro2::{Literal, Span};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::{bracketed, parse::Parse, parse_macro_input, spanned::Spanned, TraitItemMethod};

#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn plugin(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut trayt = parse_macro_input!(input as syn::ItemTrait);

    let functions = trayt
        .items
        .iter_mut()
        .filter_map(|x| match x {
            syn::TraitItem::Method(x) => Some(x),
            _ => None,
        })
        .map(|x| {
            match x.sig.output {
                syn::ReturnType::Default => {}
                syn::ReturnType::Type(_, ref mut x) => {
                    *x = syn::parse_quote!(
                        ::std::result::Result<#x, ::bazed_stew_interface::stew_rpc::Error>
                    )
                }
            }
            x
        })
        .collect::<Vec<_>>();

    let trait_name = &trayt.ident;

    let register_fns = functions
        .iter()
        .map(|x| make_register_fn(x))
        .collect::<Vec<_>>();

    let client_impl_fns = functions
        .iter()
        .enumerate()
        .map(|(idx, x)| make_client_impl_fn(idx, x))
        .collect::<Vec<_>>();

    let client_get_fns = functions
        .iter()
        .map(|x| make_client_get_fn(x))
        .collect::<Vec<_>>();

    let client_impl_name = format_ident!("{}ClientImpl", trait_name);

    quote! {
        #trayt

        pub use __internal::register_functions;
        pub use __internal::#client_impl_name;

        mod __internal {
            use super::*;
            use bazed_stew_interface::{
                stew_rpc::{self, StewConnectionSender, StewConnectionReceiver, StewClient},
                rpc_proto::{StewRpcCall, StewRpcMessage, FunctionId, PluginId}
            };

            pub struct #client_impl_name<S, D> {
                client: StewClient<S, D>,
                functions: Vec<FunctionId>,
            }

            impl <S, D> #client_impl_name<S, D> 
            where 
                S: StewConnectionSender<StewRpcCall> + Clone + 'static,
                D: Send + Sync + 'static
            {
                pub async fn initialize(mut client: StewClient<S, D>, plugin_id: PluginId) -> Result<Self, stew_rpc::Error> {
                    let mut functions = Vec::new();
                    #(functions.push(#client_get_fns);)*
                    Ok(Self {
                        client,
                        functions,
                    })
                }
            }

            #[::async_trait::async_trait]
            impl <S, D> #trait_name for #client_impl_name<S, D> 
            where 
                S: StewConnectionSender<StewRpcCall> + Clone + 'static,
                D: Send + Sync + 'static
            {
                #(#client_impl_fns)*
            }

            pub async fn register_functions<S, D>(client: &mut StewClient<S, D>) -> Result<(), stew_rpc::Error>
            where
                S: StewConnectionSender<StewRpcCall> + Clone + 'static,
                D: #trait_name + Send + Sync + 'static
            {
                #(#register_fns)*
                Ok(())
            }
        }
    }
    .into()
}

fn make_client_get_fn(function: &TraitItemMethod) -> proc_macro2::TokenStream {
    let name = &function.sig.ident;
    let name_str = syn::LitStr::new(&name.to_string(), name.span());
    quote! {
        client.get_fn(plugin_id, #name_str.to_string()).await?
    }
}

fn make_client_impl_fn(n: usize, function: &TraitItemMethod) -> proc_macro2::TokenStream {
    let args: Vec<_> = function
        .sig
        .inputs
        .iter()
        .filter_map(|x| match x {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(x) => Some(x),
        })
        .collect();
    let arg_names: Vec<_> = args
        .iter()
        .map(|x| match &*x.pat {
            syn::Pat::Ident(x) => Some(x),
            _ => proc_macro_error::abort!(x.pat.span(), "Expected identifier"),
        })
        .collect();

    let function_sig = &function.sig;

    quote! {
        #function_sig {
            #[derive(serde::Serialize)]
            struct Args { #(#args),* }
            let args = Args { #(#arg_names),* };
            self.client.call_fn_and_await_response(self.functions[#n], args).await
        }
    }
}

fn make_register_fn(function: &TraitItemMethod) -> proc_macro2::TokenStream {
    let name = &function.sig.ident;
    let name_str = syn::LitStr::new(&name.to_string(), name.span());
    let args: Vec<_> = function
        .sig
        .inputs
        .iter()
        .filter_map(|x| match x {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(x) => Some(x),
        })
        .collect();
    let arg_names: Vec<_> = args
        .iter()
        .map(|x| match &*x.pat {
            syn::Pat::Ident(x) => Some(x),
            _ => proc_macro_error::abort!(x.pat.span(), "Expected identifier"),
        })
        .collect();

    quote! {{
        #[derive(serde::Deserialize)]
        struct Args { #(#args),* }
        client.register_fn(#name_str, |userdata, args| Box::pin(async move {
            let args: Args = serde_json::from_value(args).map_err(|e| serde_json::json!(e.to_string()))?;
            let result = userdata.#name(#(args.#arg_names),*).await.unwrap();
            match result {
                Ok(x) => Ok(serde_json::to_value(x).unwrap()),
                Err(x) => Err(serde_json::to_value(x).unwrap()),
            }
        })).await?;
    }}
}

#[proc_macro]
#[proc_macro_error::proc_macro_error]
pub fn stew_plugin(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as PluginDefinition);
    let mut api_major = Literal::usize_unsuffixed(def.api_version.1);
    let mut api_minor = Literal::usize_unsuffixed(def.api_version.2);
    api_major.set_span(def.api_version.0);
    api_minor.set_span(def.api_version.0);
    let name = def.name;
    let version = def.version;
    let init = def.init;
    let main = def.main;

    let (get_fns, actual_fns): (Vec<_>, Vec<_>) = def
        .imports
        .into_iter()
        .enumerate()
        .map(|(n, (path, sig))| {
            let bytes: String = path
                .segments
                .iter()
                .map(|x| x.ident.to_string())
                .intersperse("::".to_string())
                .collect();
            let mut bytes = bytes.as_bytes().to_vec();
            bytes.push(b'\0');
            (
                make_get_fn(
                    syn::LitByteStr::new(&bytes, path.segments.first().unwrap().ident.span()),
                    n,
                ),
                make_proxy_fn(path, n, sig),
            )
        })
        .unzip();

    quote::quote! {
        fn metadata() -> PluginMetadata {
            PluginMetadata {
                api_major: #api_major,
                api_minor: #api_minor,
                name: ::std::ffi::CStr::from_bytes_with_nul(#name).unwrap().as_ptr(),
                version: ::std::ffi::CStr::from_bytes_with_nul(#version).unwrap().as_ptr(),
                init: #init,
                main: #main,
            }
        }
        unsafe extern "C" fn init_shit(stew: *const StewVft0) {
            #(#get_fns;)*
        }

        struct Thingy { stew: *const StewVft0, }
        impl Thingy { #(#actual_fns)* }
    }
    .into()
}

fn make_get_fn(name: syn::LitByteStr, n: usize) -> proc_macro2::TokenStream {
    quote! {
        let _ = ((*stew).get_fn)(
            std::ffi::CStr::from_bytes_with_nul(#name).unwrap().as_ptr(),
            #n,
        );
    }
}

fn make_proxy_fn(path: syn::Path, n: usize, sig: syn::TypeBareFn) -> proc_macro2::TokenStream {
    let fn_name: String = path
        .segments
        .iter()
        .map(|x| x.ident.to_string())
        .intersperse("_".to_string())
        .collect();
    let fn_token = sig.fn_token;
    let inputs = sig.inputs;
    let output = sig.output;
    let lifetimes = sig.lifetimes;
    let fn_name = proc_macro2::Ident::new(&fn_name, path.span());
    let input_names: Vec<_> = inputs.iter().cloned().map(|x| x.name.unwrap().0).collect();
    quote! {
        extern "C" #fn_token #fn_name #lifetimes(&self, #inputs) #output {
            #[no_mangle]
            #[repr(C)]
            struct Args {
                #inputs
            }
            unsafe {
                ((*self.stew).call)(#n, Box::into_raw(Box::new(Args {
                    #(#input_names: #input_names),*
                })) as *mut ::std::ffi::c_void)
            }
        }
    }
}

struct Version(Span, usize, usize);

impl Parse for Version {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let version = input.parse::<syn::LitStr>()?;
        if let Some((major, minor)) = version.value().split_once('.') {
            match (major.parse(), minor.parse()) {
                (Ok(major), Ok(minor)) => Ok(Self(version.span(), major, minor)),
                (Err(_), _) => {
                    abort!(version, "Malformed major version, must be an integer")
                },
                (_, Err(_)) => {
                    abort!(version, "Malformed minor version, must be an integer")
                },
            }
        } else {
            abort!(version, "Version must be formatted as <major>.<minor>.");
        }
    }
}

struct PluginDefinition {
    api_version: Version,
    version: syn::LitByteStr,
    name: syn::LitByteStr,
    init: syn::Expr,
    main: syn::Expr,
    imports: Vec<(syn::Path, syn::TypeBareFn)>,
}

fn parse_kv_right_side<V: syn::parse::Parse>(input: syn::parse::ParseStream) -> syn::Result<V> {
    let _ = input.parse::<syn::Token![=]>()?;
    input.parse()
}

impl Parse for PluginDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut api_version = None;
        let mut version = None;
        let mut name = None;
        let mut init = None;
        let mut main = None;
        let mut imports = None;

        while !input.is_empty() {
            let ident = input.parse::<syn::Ident>()?;
            match ident.to_string().as_str() {
                "api_version" => api_version = Some(parse_kv_right_side(input)?),
                "name" => name = Some(parse_kv_right_side(input)?),
                "version" => version = Some(parse_kv_right_side(input)?),
                "init" => init = Some(parse_kv_right_side(input)?),
                "main" => main = Some(parse_kv_right_side(input)?),
                "imports" => {
                    imports = Some({
                        let _ = input.parse::<syn::Token![=]>()?;
                        let content;
                        let _bracket_token = bracketed!(content in input);
                        let content = content.parse_terminated::<_, syn::Token![,]>(|stream| {
                            let path = stream.parse()?;
                            let _colon_token = stream.parse::<syn::Token![:]>()?;
                            let sig = stream.parse()?;
                            Ok((path, sig))
                        })?;
                        content.into_iter().collect()
                    })
                },
                _ => abort!(ident, "Unexpected field `{ident}`"),
            }
            if input.is_empty() {
                break;
            } else {
                _ = input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Self {
            api_version: api_version
                .unwrap_or_else(|| abort!(input.span(), "missing field `api_version`")),
            version: version.unwrap_or_else(|| abort!(input.span(), "missing field `version`")),
            name: name.unwrap_or_else(|| abort!(input.span(), "missing field `name`")),
            init: init.unwrap_or_else(|| abort!(input.span(), "missing field `init`")),
            main: main.unwrap_or_else(|| abort!(input.span(), "missing field `main`")),
            imports: imports.unwrap_or_else(|| abort!(input.span(), "missing field `functions`")),
        })
    }
}
