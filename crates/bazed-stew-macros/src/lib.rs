#![feature(iter_intersperse, proc_macro_def_site)]

use darling::{FromMeta, ToTokens};
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, emit_error};
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, spanned::Spanned, TraitItemMethod};
use utils::{returns_nothing, returns_result, wrap_return_type};

mod utils;

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

    let trayt = parse_macro_input!(input as syn::ItemTrait);

    let functions = trayt
        .items
        .iter()
        .map(|x| match x {
            syn::TraitItem::Method(x) => {
                if x.sig.asyncness.is_none() {
                    abort!(x.span(), "All methods in a plugin must be async");
                }
                x
            },
            _ => abort!(x.span(), "Only methods are allowed in a plugin trait"),
        })
        .collect::<Vec<_>>();

    let trait_name = &trayt.ident;

    let client_impl_name = format_ident!("{}Client", trait_name);
    let client_impl = make_client_impl(&args, &client_impl_name, &functions);
    let server_module = make_server_module(&args, trait_name, &functions);

    let internal_ident = syn::Ident::new("internal", proc_macro::Span::def_site().into());

    quote! {
        #trayt

        pub use #internal_ident::server;
        pub use #internal_ident::#client_impl_name;
        mod #internal_ident {
            use super::*;
            use ::bazed_stew_interface::{
                stew_rpc::{self, StewConnectionSender, StewConnectionReceiver, StewSession, StewSessionBase},
                rpc_proto::{StewRpcCall, StewRpcMessage, FunctionId, PluginId, PluginMetadata},
                re_exports
            };

            #server_module
            #client_impl

        }
    }
    .into()
}

fn make_metadata_struct_instance(args: &PluginAttr) -> proc_macro2::TokenStream {
    let plugin_version = &args.version;
    let stew_version_maj = args.stew_version.1;
    let stew_version_min = args.stew_version.2;
    let plugin_name = &args.name;
    quote! {
        PluginMetadata {
            api_major: #stew_version_maj,
            api_minor: #stew_version_min,
            name: #plugin_name.to_string(),
            version: #plugin_version.parse().unwrap(),
        }
    }
}

fn make_server_module(
    args: &PluginAttr,
    trait_name: &syn::Ident,
    functions: &[&TraitItemMethod],
) -> proc_macro2::TokenStream {
    let trait_name_str = trait_name.to_string();
    let register_fns = functions.iter().map(|x| make_register_fn(x));
    let metadata = make_metadata_struct_instance(args);
    quote! {
        pub mod server {
            use super::*;
            #[doc = ::std::concat!(
                "Register this plugin with the given StewSession by sending Metadata and registering all functions.\n",
                "This function _must_ be called before using your [`", #trait_name_str, "`]-instance"
            )]
            pub async fn initialize<D>(client: &mut StewSession<D>) -> Result<(), stew_rpc::Error>
            where
                D: #trait_name + Send + Sync + 'static
            {
                client.send_call(StewRpcCall::Metadata(#metadata)).await?;
                #(#register_fns)*
                Ok(())
            }
        }
    }
}

fn make_client_impl(
    args: &PluginAttr,
    client_impl_name: &syn::Ident,
    functions: &[&syn::TraitItemMethod],
) -> proc_macro2::TokenStream {
    let client_impl_fns = functions
        .iter()
        .enumerate()
        .map(|(idx, x)| make_client_impl_fn(idx, (*x).clone()));

    let plugin_version = &args.version;
    let plugin_name = &args.name;

    let client_get_fns = functions.iter().map(|function| {
        let name = &function.sig.ident;
        let name_str = syn::LitStr::new(&name.to_string(), name.span());
        quote!(client.get_fn(plugin_id, #name_str.to_string()).await?)
    });

    quote! {
        #[derive(Clone)]
        pub struct #client_impl_name {
            client: StewSessionBase,
            functions: Vec<FunctionId>,
        }

        impl  #client_impl_name {
            pub async fn load(mut client: StewSessionBase) -> Result<Self, stew_rpc::Error> {
                Self::load_at(client, #plugin_version.parse().unwrap())
                    .await
            }

            pub async fn load_at(
                mut client: StewSessionBase,
                version: re_exports::semver::VersionReq
            ) -> Result<Self, stew_rpc::Error> {
                let plugin_info = client
                    .load_plugin(#plugin_name.to_string(), version)
                    .await?;
                Self::initialize(client, plugin_info.plugin_id).await
            }

            pub async fn initialize(mut client: StewSessionBase, plugin_id: PluginId) -> Result<Self, stew_rpc::Error> {
                let functions = vec![ #(#client_get_fns),* ];
                Ok(Self { client, functions })
            }

            #(#client_impl_fns)*
        }

    }
}

#[derive(darling::FromMeta)]
struct PluginAttr {
    name: syn::LitStr,
    version: syn::LitStr,
    stew_version: Version,
}

fn make_client_impl_fn(n: usize, mut function: TraitItemMethod) -> proc_macro2::TokenStream {
    // we have to get this before changing the signature
    let returns_result = returns_result(&function.sig);
    let returns_nothing = returns_nothing(&function.sig);

    function.sig = wrap_return_type(
        function.sig,
        |old| parse_quote!(::std::result::Result<#old, ::bazed_stew_interface::stew_rpc::Error>),
    );
    let inputs = &function.sig.inputs;
    let args: Vec<_> = inputs
        .iter()
        .filter_map(|x| match x {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(x) => Some(x),
        })
        .collect();

    let arg_names = args.iter().map(|x| match &*x.pat {
        syn::Pat::Ident(x) => Some(x),
        _ => abort!(x.pat.span(), "Expected identifier"),
    });

    let call_fn_line = if returns_nothing {
        quote! { self.client.call_fn_ignore_response(self.functions[#n], args).await }
    } else if returns_result {
        quote! { self.client.call_fn_and_await_response(self.functions[#n], args).await }
    } else {
        quote! {
            match self.client.call_fn_and_await_response(self.functions[#n], args).await? {
                Ok(result) => Ok(result),
                Err(err) => Err(::bazed_stew_interface::stew_rpc::Error::InfallibleFunctionFailed(err)),
            }
        }
    };

    let function_sig = &function.sig;
    let attrs = &function.attrs;

    quote! {
        #[tracing::instrument(skip(self))]
        #(#attrs)*
        pub #function_sig {
            #[derive(serde::Serialize)]
            struct Args { #(#args),* }
            let args = Args { #(#arg_names),* };
            #call_fn_line
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
    let arg_names = args.iter().map(|x| match &*x.pat {
        syn::Pat::Ident(x) => Some(x),
        _ => abort!(x.pat.span(), "Expected identifier"),
    });

    let return_code = if returns_result(&function.sig) {
        quote!(match result {
            Ok(x) => Ok(serde_json::to_value(x).unwrap()),
            Err(x) => Err(serde_json::to_value(x).unwrap()),
        })
    } else {
        quote!(Ok(serde_json::to_value(result).unwrap()))
    };

    quote! {{
        #[derive(serde::Deserialize)]
        struct Args { #(#args),* }

        client.register_fn(#name_str, |userdata, args| Box::pin(async move {
            let args: Args = serde_json::from_value(args).map_err(|e| serde_json::json!(e.to_string()))?;
            let result = userdata.#name(#(args.#arg_names),*).await;
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
            syn::Lit::Str(x) => x.clone(),
            _ => {
                emit_error!(version, "Version must be formatted as \"<major>.<minor>\".");
                syn::LitStr::new("999.999", version.span())
            },
        };
        if let Some((major, minor)) = version.value().split_once('.') {
            match (major.parse(), minor.parse()) {
                (Ok(major), Ok(minor)) => Ok(Version(version.span(), major, minor)),
                (Err(e), _) => {
                    emit_error!(version, "Malformed major version, must be an integer"; cause = format!("{e}"););
                    Ok(Version(version.span(), 999, 999))
                },
                (_, Err(e)) => {
                    emit_error!(version, "Malformed minor version, must be an integer"; cause = format!("{e}"););
                    Ok(Version(version.span(), 999, 999))
                },
            }
        } else {
            emit_error!(version, "Version must be formatted as \"<major>.<minor>\".");
            Ok(Version(version.span(), 999, 999))
        }
    }
}
