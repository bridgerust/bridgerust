use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

pub fn export_exception(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let vis = &input.vis;

    // Parse attributes to get module name
    // Usage: #[bridgerust::exception(module = "embex")]
    let args = syn::parse_macro_input!(attr as ExceptionArgs);
    let module_name = match args.module {
        Some(m) => m,
        None => "builtins".to_string(), // Default or error? explicit is better.
    };
    let module_ident = syn::Ident::new(&module_name, proc_macro2::Span::call_site());

    // Python expansion: delegate to create_exception!
    // Note: create_exception! defines the struct, so we only emit it for python feature.
    let python_expanded = quote! {
        #[cfg(feature = "python")]
        ::bridgerust::pyo3::create_exception!(#module_ident, #struct_name, ::bridgerust::pyo3::exceptions::PyException);
    };

    // Node/Rust expansion: define a standard struct
    // We assume unit struct for now as exceptions usually don't carry fields in this macro usage pattern
    let rust_expanded = quote! {
        #[cfg(not(feature = "python"))]
        #[derive(Debug, Clone)]
        #vis struct #struct_name;

        #[cfg(not(feature = "python"))]
        impl std::fmt::Display for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, stringify!(#struct_name))
            }
        }

        #[cfg(not(feature = "python"))]
        impl std::error::Error for #struct_name {}

        // Helper to convert to Napi Error if needed
        #[cfg(all(feature = "nodejs", not(feature = "python")))]
        impl From<#struct_name> for ::bridgerust::napi::Error {
            fn from(e: #struct_name) -> Self {
                ::bridgerust::napi::Error::from_reason(e.to_string())
            }
        }
    };

    let expanded = quote! {
        #python_expanded
        #rust_expanded
    };

    TokenStream::from(expanded)
}

struct ExceptionArgs {
    module: Option<String>,
}

impl syn::parse::Parse for ExceptionArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut module = None;
        if !input.is_empty() {
            let opts =
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated(input)?;
            for opt in opts {
                if let syn::Meta::NameValue(nv) = opt
                    && nv.path.is_ident("module")
                    && let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit),
                        ..
                    }) = nv.value
                {
                    module = Some(lit.value());
                }
            }
        }
        Ok(ExceptionArgs { module })
    }
}
