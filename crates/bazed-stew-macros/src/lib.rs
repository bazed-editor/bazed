#![feature(iter_intersperse)]

use darling::{FromMeta, ToTokens};
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, TraitItemMethod};

#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn plugin(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attrs as syn::AttributeArgs);
    let args = match PluginAttr::from_list(&args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        },
    };

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
                syn::ReturnType::Default => {},
                syn::ReturnType::Type(_, ref mut x) => {
                    *x = syn::parse_quote!(
                        ::std::result::Result<#x, ::bazed_stew_interface::stew_rpc::Error>
                    )
                },
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

    let plugin_version = args.version;
    let stew_version_maj = args.stew_version.1;
    let stew_version_min = args.stew_version.2;
    let plugin_name = args.name;

    quote! {
        #trayt

        pub use __internal::server;
        pub use __internal::#client_impl_name;

        mod __internal {
            use super::*;
            use bazed_stew_interface::{
                stew_rpc::{self, StewConnectionSender, StewConnectionReceiver, StewClient},
                rpc_proto::{StewRpcCall, StewRpcMessage, FunctionId, PluginId, PluginMetadata},
                semver,
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
                pub async fn load(mut client: StewClient<S, D>) -> Result<Self, stew_rpc::Error> {
                    Self::load_at(client, #plugin_version.parse().unwrap())
                        .await
                }

                pub async fn load_at(mut client: StewClient<S, D>, version: semver::VersionReq) -> Result<Self, stew_rpc::Error> {
                    let plugin_info = client
                        .load_plugin(#plugin_name.to_string(), version)
                        .await?;
                    Self::initialize(client, plugin_info.plugin_id).await
                }

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


            pub mod server {
                use super::*;
                pub async fn initialize<S, D>(client: &mut StewClient<S, D>) -> Result<(), stew_rpc::Error>
                where
                    S: StewConnectionSender<StewRpcCall> + Clone + 'static,
                    D: #trait_name + Send + Sync + 'static
                {
                    client
                        .send_call(StewRpcCall::Metadata(PluginMetadata {
                            api_major: #stew_version_maj,
                            api_minor: #stew_version_min,
                            name: #plugin_name.to_string(),
                            version: #plugin_version.parse().unwrap(),
                        }))
                        .await?;

                    #(#register_fns)*
                    Ok(())
                }
            }
        }
    }
    .into()
}

#[derive(darling::FromMeta)]
struct PluginAttr {
    name: syn::LitStr,
    version: syn::LitStr,
    stew_version: Version,
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

    let call_fn_line = if returns_result(&function.sig) {
        quote! { self.client.call_fn_and_await_response(self.functions[#n], args).await }
    } else {
        quote! { self.client.call_fn_and_await_response_infallible(self.functions[#n], args).await }
    };

    quote! {
        #function_sig {
            #[derive(serde::Serialize)]
            struct Args { #(#args),* }
            let args = Args { #(#arg_names),* };
            #call_fn_line
        }
    }
}

fn returns_result(sig: &syn::Signature) -> bool {
    match &sig.output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => match &**ty {
            syn::Type::Path(p) => {
                let path = &p.path;
                path.segments.len() == 1 && path.segments[0].ident == "Result"
            },
            _ => false,
        },
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

    let return_code = if returns_result(&function.sig) {
        quote! {
            match result {
                Ok(x) => Ok(serde_json::to_value(x).unwrap()),
                Err(x) => Err(serde_json::to_value(x).unwrap()),
            }
        }
    } else {
        quote! {
            Ok(serde_json::to_value(result).unwrap())
        }
    };

    quote! {{
        #[derive(serde::Deserialize)]
        struct Args { #(#args),* }

        client.register_fn(#name_str, |userdata, args| Box::pin(async move {
            let args: Args = serde_json::from_value(args).map_err(|e| serde_json::json!(e.to_string()))?;
            let result = userdata.#name(#(args.#arg_names),*).await.unwrap();
            #return_code
        })).await?;
    }}
}

struct Version(Span, usize, usize);

impl ToTokens for Version {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let version = format!("{}.{}", self.1, self.2);
        let version = syn::LitStr::new(&version, self.0);
        tokens.extend(quote!(#version))
    }
}

impl darling::FromMeta for Version {
    fn from_value(version: &syn::Lit) -> darling::Result<Self> {
        let version = match version {
            syn::Lit::Str(x) => x,
            _ => abort!(version, "Version must be formatted as \"<major>.<minor>\""),
        };

        if let Some((major, minor)) = version.value().split_once('.') {
            match (major.parse(), minor.parse()) {
                (Ok(major), Ok(minor)) => Ok(Version(version.span(), major, minor)),
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
