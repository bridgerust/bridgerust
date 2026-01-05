use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ImplItem, ItemEnum, ItemFn, ItemImpl, ItemStruct, parse_macro_input};

// Helper to parse arguments like #[export(object)]
struct ExportArgs {
    is_object: bool,
}

impl syn::parse::Parse for ExportArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut is_object = false;

        if !input.is_empty() {
            let vars =
                syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated(input)?;
            for var in vars {
                if var == "object" {
                    is_object = true;
                }
            }
        }

        Ok(ExportArgs { is_object })
    }
}

/// Enhanced export macro that generates bindings for both Python and Node.js
#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attributes
    let args = syn::parse_macro_input!(attr as ExportArgs);

    // Try to parse as function first
    if let Ok(input_fn) = syn::parse::<ItemFn>(item.clone()) {
        return export_function(input_fn);
    }

    // Try to parse as struct
    if let Ok(input_struct) = syn::parse::<ItemStruct>(item.clone()) {
        return export_struct(input_struct, args);
    }

    // Try to parse as enum
    if let Ok(input_enum) = syn::parse::<ItemEnum>(item.clone()) {
        return export_enum(input_enum);
    }

    // Try to parse as impl block (methods)
    if let Ok(input_impl) = syn::parse::<ItemImpl>(item.clone()) {
        return export_impl(input_impl);
    }

    // If none work, try as generic item and provide helpful error
    let input = parse_macro_input!(item as DeriveInput);
    syn::Error::new_spanned(
        &input.ident,
        "#[bridgerust::export] can only be applied to functions, structs, enums, or impl blocks",
    )
    .to_compile_error()
    .into()
}

fn export_function(input_fn: ItemFn) -> TokenStream {
    // Validate function visibility
    if !matches!(input_fn.vis, syn::Visibility::Public(_)) {
        return syn::Error::new_spanned(
            &input_fn.sig.ident,
            "Functions exported with #[bridgerust::export] must be public (use `pub fn`)",
        )
        .to_compile_error()
        .into();
    }

    // Check for generic type parameters
    if !input_fn.sig.generics.params.is_empty() {
        // (Simplify error for brevity)
        return syn::Error::new_spanned(
            &input_fn.sig.generics,
            "Generic functions are not supported by BridgeRust export",
        )
        .to_compile_error()
        .into();
    }

    let is_async = input_fn.sig.asyncness.is_some();

    // Check if return type is an iterator
    let is_iterator = if let syn::ReturnType::Type(_, return_type) = &input_fn.sig.output {
        is_iterator_type(return_type)
    } else {
        false
    };

    if is_iterator && let syn::ReturnType::Type(_, return_type) = &input_fn.sig.output {
        return syn::Error::new_spanned(
            return_type,
            "Iterator return types used in export are not yet supported. Return Vec<T> instead.",
        )
        .to_compile_error()
        .into();
    }

    if let syn::ReturnType::Type(_, return_type) = &input_fn.sig.output
        && let Err(err) = validate_type(return_type, "return type")
    {
        return err;
    }

    for input in &input_fn.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input
            && let Err(err) = validate_type(&pat_type.ty, "parameter type")
        {
            return err;
        }
    }

    if is_async {
        export_async_function(input_fn)
    } else {
        let expanded = quote! {
            #[cfg_attr(feature = "python", ::bridgerust::pyo3::pyfunction(crate = "::bridgerust::pyo3"))]
            #[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi)]
            #input_fn
        };
        TokenStream::from(expanded)
    }
}

fn export_async_function(input_fn: ItemFn) -> TokenStream {
    let fn_name = &input_fn.sig.ident;
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;

    let mut wrapper_params = Vec::new();
    let mut call_args = Vec::new();

    for input in &fn_sig.inputs {
        match input {
            syn::FnArg::Receiver(_) => {
                return syn::Error::new_spanned(
                    fn_sig,
                    "Methods are not supported in async functions via export_function logic",
                )
                .to_compile_error()
                .into();
            }
            syn::FnArg::Typed(pat_type) => {
                let param_name = &pat_type.pat;
                let param_type = &pat_type.ty;
                wrapper_params.push(quote! { #param_name: #param_type });
                call_args.push(quote! { #param_name });
            }
        }
    }

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig #fn_block

        #[cfg(feature = "nodejs")]
        #[bridgerust::napi_derive::napi]
        #(#fn_attrs)*
        #fn_vis #fn_sig #fn_block

        #[cfg(feature = "python")]
        #[bridgerust::pyo3::pyfunction(crate = "bridgerust::pyo3")]
        pub fn #fn_name(
            py: bridgerust::pyo3::Python<'_>,
            #(#wrapper_params),*
        ) -> bridgerust::pyo3::PyResult<bridgerust::pyo3::PyObject> {
            use bridgerust::pyo3::IntoPy;
            bridgerust::pyo3_async_runtimes::tokio::future_into_py(py, async move {
                #fn_name(#(#call_args),*).await.into_py(py)
            })
        }
    };

    TokenStream::from(expanded)
}

fn is_iterator_type(ty: &syn::Type) -> bool {
    // Simplified check
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let name = segment.ident.to_string();
        return name == "Iterator" || name == "IntoIterator";
    }
    false
}

fn validate_type(ty: &syn::Type, _context: &str) -> Result<(), TokenStream> {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let name = segment.ident.to_string();
        if matches!(name.as_str(), "HashMap" | "HashSet" | "BTreeMap") {
            return Err(syn::Error::new_spanned(
                ty,
                format!(
                    "Type {} not supported in bridge, use Vec instead or custom wrapper",
                    name
                ),
            )
            .to_compile_error()
            .into());
        }
        if (name == "Vec" || name == "Option")
            && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
        {
            return validate_type(inner, _context);
        }
    }
    Ok(())
}

fn export_struct(mut input_struct: ItemStruct, args: ExportArgs) -> TokenStream {
    if !matches!(input_struct.vis, syn::Visibility::Public(_)) {
        return syn::Error::new_spanned(&input_struct.ident, "Structs exported must be public")
            .to_compile_error()
            .into();
    }

    if !input_struct.generics.params.is_empty() {
        return syn::Error::new_spanned(&input_struct.generics, "Generics not supported")
            .to_compile_error()
            .into();
    }

    if let syn::Fields::Unnamed(_) = &input_struct.fields {
        return syn::Error::new_spanned(&input_struct.ident, "Tuple structs not supported")
            .to_compile_error()
            .into();
    }

    // Prepare fields for Python (with PyO3 attrs)
    let mut fields_py = syn::punctuated::Punctuated::<syn::Field, syn::token::Comma>::new();
    // Prepare fields for Node (without PyO3 attrs)
    let mut fields_node = syn::punctuated::Punctuated::<syn::Field, syn::token::Comma>::new();

    if let syn::Fields::Named(fields) = &mut input_struct.fields {
        for field in &mut fields.named {
            if let Err(err) = validate_type(&field.ty, "struct field") {
                return err;
            }

            // Base attributes (common)
            let mut base_attrs = Vec::new();
            let mut is_readonly = false;
            for attr in &field.attrs {
                if attr.path().is_ident("readonly") {
                    is_readonly = true;
                } else {
                    base_attrs.push(attr.clone());
                }
            }

            if matches!(field.vis, syn::Visibility::Public(_)) {
                // Python Field
                let mut fp = field.clone();
                fp.attrs = base_attrs.clone();
                if is_readonly {
                    fp.attrs.push(syn::parse_quote!(#[pyo3(get)]));
                } else {
                    fp.attrs.push(syn::parse_quote!(#[pyo3(get, set)]));
                }
                // Add Napi attrs if needed (for mixed builds)
                if is_readonly && !args.is_object {
                    fp.attrs.push(syn::parse_quote!(#[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi(readonly))]));
                }
                fields_py.push(fp);

                // Node Field
                let mut fn_ = field.clone();
                fn_.attrs = base_attrs.clone();
                // No PyO3 attrs
                if is_readonly && !args.is_object {
                    fn_.attrs.push(syn::parse_quote!(#[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi(readonly))]));
                }
                fields_node.push(fn_);
            } else {
                // Private: add to both
                let mut fp = field.clone();
                fp.attrs = base_attrs.clone(); // Strip readonly if accidentally on private?
                fields_py.push(fp);

                let mut fn_ = field.clone();
                fn_.attrs = base_attrs.clone();
                fields_node.push(fn_);
            }
        }
    }

    // Construct Python Struct
    let mut struct_py = input_struct.clone();
    if let syn::Fields::Named(f) = &mut struct_py.fields {
        f.named = fields_py;
    }

    // Construct Node Struct
    let mut struct_node = input_struct.clone();
    if let syn::Fields::Named(f) = &mut struct_node.fields {
        f.named = fields_node;
    }

    let napi_attr = if args.is_object {
        quote! { #[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi(object))] }
    } else {
        quote! { #[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi)] }
    };

    let expanded = quote! {
        #[cfg(feature = "python")]
        #[::bridgerust::pyo3::pyclass(crate = "::bridgerust::pyo3")]
        #napi_attr
        #struct_py

        #[cfg(not(feature = "python"))]
        #napi_attr
        #struct_node
    };
    TokenStream::from(expanded)
}

fn export_enum(input_enum: ItemEnum) -> TokenStream {
    if !matches!(input_enum.vis, syn::Visibility::Public(_)) {
        return syn::Error::new_spanned(&input_enum.ident, "Enums must be public")
            .to_compile_error()
            .into();
    }
    if !input_enum.generics.params.is_empty() {
        return syn::Error::new_spanned(&input_enum.generics, "Generic enums not supported")
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        #[cfg_attr(feature = "python", ::bridgerust::pyo3::pyclass(crate = "::bridgerust::pyo3"))]
        #[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi)]
        #input_enum
    };
    TokenStream::from(expanded)
}

fn export_impl(mut input_impl: ItemImpl) -> TokenStream {
    if input_impl.trait_.is_some() {
        return syn::Error::new_spanned(input_impl.self_ty, "Trait impls not supported")
            .to_compile_error()
            .into();
    }

    for item in &mut input_impl.items {
        if let ImplItem::Fn(method) = item {
            let mut is_constructor = false;
            let mut new_attrs = Vec::new();
            for attr in &method.attrs {
                if attr.path().is_ident("constructor") {
                    is_constructor = true;
                } else {
                    new_attrs.push(attr.clone());
                }
            }
            method.attrs = new_attrs;

            if is_constructor {
                method.attrs.push(syn::parse_quote!(#[new]));
                method.attrs.push(syn::parse_quote!(#[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi(constructor))]));
            } else {
                let has_self = method
                    .sig
                    .inputs
                    .iter()
                    .any(|arg| matches!(arg, syn::FnArg::Receiver(_)));
                if !has_self {
                    method.attrs.push(syn::parse_quote!(#[staticmethod]));
                }
            }
        }
    }

    let expanded = quote! {
        #[cfg_attr(feature = "python", ::bridgerust::pyo3::pymethods(crate = "::bridgerust::pyo3"))]
        #[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi)]
        #input_impl
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let enum_name = &input.ident;
    if !matches!(input.data, syn::Data::Enum(_)) {
        return syn::Error::new_spanned(&input.ident, "error macro only for enums")
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        #input
        #[cfg(feature = "python")]
        #[allow(unexpected_cfgs)]
        pub fn to_py_err(err: #enum_name) -> bridgerust::pyo3::PyErr {
            use std::fmt::Display;
            bridgerust::pyo3::exceptions::PyRuntimeError::new_err(err.to_string())
        }

        #[cfg(feature = "nodejs")]
        #[allow(unexpected_cfgs)]
        pub fn to_napi_err(err: #enum_name) -> bridgerust::napi::Error {
            use std::fmt::Display;
            bridgerust::napi::Error::from_reason(err.to_string())
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn new(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn staticmethod(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn pyo3_dummy(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

mod exception;

/// Export a struct as a cross-platform exception.
///
/// Usage: `#[bridgerust::exception(module = "my_module")]`
#[proc_macro_attribute]
pub fn exception(attr: TokenStream, item: TokenStream) -> TokenStream {
    exception::export_exception(attr, item)
}
