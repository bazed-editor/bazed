use proc_macro2::Span;

pub fn wrap_return_type(
    mut sig: syn::Signature,
    wrap: impl FnOnce(Box<syn::Type>) -> syn::Type,
) -> syn::Signature {
    sig.output = match sig.output {
        syn::ReturnType::Default => syn::ReturnType::Type(
            syn::Token![->](Span::call_site()),
            Box::new(wrap(syn::parse_quote!(()))),
        ),
        syn::ReturnType::Type(tok, old) => syn::ReturnType::Type(tok, Box::new(wrap(old))),
    };
    sig
}

pub fn returns_nothing(sig: &syn::Signature) -> bool {
    match &sig.output {
        syn::ReturnType::Default => true,
        syn::ReturnType::Type(_, ty) => match &**ty {
            syn::Type::Path(p) => {
                let path = &p.path;
                path.segments.len() == 1 && path.segments[0].ident == "()"
            },
            _ => false,
        },
    }
}

pub fn returns_result(sig: &syn::Signature) -> bool {
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
