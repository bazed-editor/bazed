#![feature(iter_intersperse)]

use proc_macro::TokenStream;
use proc_macro2::{Literal, Span};
use proc_macro_error::abort;
use quote::quote;
use syn::{bracketed, parse::Parse, parse_macro_input, spanned::Spanned};

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
