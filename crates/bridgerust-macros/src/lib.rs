use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, ImplItem, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, parse_macro_input,
};

// Helper to parse arguments like #[export(object)]
struct ExportArgs {
    is_object: bool,
}

#[derive(Default, Clone, PartialEq)]
struct ValidationRules {
    required: bool,
    email: bool,
    url: bool,
    min: Option<f64>,
    max: Option<f64>,
    len: Option<usize>,
    pattern: Option<String>,
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

/// Shorthand for #[export] - more ergonomic
#[proc_macro_attribute]
pub fn bridge(attr: TokenStream, item: TokenStream) -> TokenStream {
    process_export(attr, item)
}

#[proc_macro_attribute]
pub fn validate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(
        attr with syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
    );

    for meta in attrs {
        match meta {
            syn::Meta::Path(path) => {
                let is_supported =
                    path.is_ident("required") || path.is_ident("email") || path.is_ident("url");
                if !is_supported {
                    return syn::Error::new_spanned(
                        path,
                        "Unsupported #[validate] flag. Supported flags: required, email, url",
                    )
                    .to_compile_error()
                    .into();
                }
            }
            syn::Meta::NameValue(nv) => {
                let key = nv
                    .path
                    .get_ident()
                    .map(ToString::to_string)
                    .unwrap_or_default();
                let is_supported = matches!(key.as_str(), "min" | "max" | "len" | "pattern");
                if !is_supported {
                    return syn::Error::new_spanned(
                        nv.path,
                        "Unsupported #[validate] key. Supported keys: min, max, len, pattern",
                    )
                    .to_compile_error()
                    .into();
                }

                if key == "pattern"
                    && !matches!(
                        &nv.value,
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(_),
                            ..
                        })
                    )
                {
                    return syn::Error::new_spanned(
                        nv.value,
                        "#[validate(pattern = ...)] expects a string literal",
                    )
                    .to_compile_error()
                    .into();
                }
            }
            syn::Meta::List(list) => {
                return syn::Error::new_spanned(
                    list,
                    "Unsupported #[validate(...)] list form. Use flags (required) or key/value pairs (min = 1)",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    item
}

#[proc_macro_attribute]
pub fn bridge_async(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "#[bridge_async] can only be used on async functions. \
             Either add 'async' keyword or use #[bridge] instead.",
        )
        .to_compile_error()
        .into();
    }

    export_async_function(input_fn)
}

#[proc_macro_attribute]
pub fn bridge_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(item as ItemMod);

    if let Some((_, items)) = &mut input.content {
        for item in items {
            process_item_for_bridge(item);
        }
    }

    quote!(#input).into()
}

fn parse_validation_rules(
    attr: &syn::Attribute,
    current: &mut ValidationRules,
) -> Result<(), syn::Error> {
    let metas = attr.parse_args_with(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
    )?;

    for meta in metas {
        match meta {
            syn::Meta::Path(path) => {
                if path.is_ident("required") {
                    current.required = true;
                } else if path.is_ident("email") {
                    current.email = true;
                } else if path.is_ident("url") {
                    current.url = true;
                } else {
                    return Err(syn::Error::new_spanned(
                        path,
                        "Unsupported #[validate] flag. Supported flags: required, email, url",
                    ));
                }
            }
            syn::Meta::NameValue(nv) => {
                let key = nv
                    .path
                    .get_ident()
                    .map(ToString::to_string)
                    .unwrap_or_default();
                match key.as_str() {
                    "min" => current.min = Some(parse_numeric_literal(&nv.value)?),
                    "max" => current.max = Some(parse_numeric_literal(&nv.value)?),
                    "len" => current.len = Some(parse_usize_literal(&nv.value)?),
                    "pattern" => current.pattern = Some(parse_string_literal(&nv.value)?),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            nv.path,
                            "Unsupported #[validate] key. Supported keys: min, max, len, pattern",
                        ));
                    }
                }
            }
            syn::Meta::List(list) => {
                return Err(syn::Error::new_spanned(
                    list,
                    "Unsupported #[validate(...)] list form. Use flags or key/value pairs",
                ));
            }
        }
    }

    Ok(())
}

fn parse_numeric_literal(expr: &syn::Expr) -> Result<f64, syn::Error> {
    let syn::Expr::Lit(syn::ExprLit { lit, .. }) = expr else {
        return Err(syn::Error::new_spanned(
            expr,
            "Expected numeric literal in #[validate(...)]",
        ));
    };

    match lit {
        syn::Lit::Int(v) => v
            .base10_parse::<f64>()
            .map_err(|_| syn::Error::new_spanned(v, "Invalid integer literal")),
        syn::Lit::Float(v) => v
            .base10_parse::<f64>()
            .map_err(|_| syn::Error::new_spanned(v, "Invalid float literal")),
        _ => Err(syn::Error::new_spanned(
            lit,
            "Expected numeric literal in #[validate(...)]",
        )),
    }
}

fn parse_usize_literal(expr: &syn::Expr) -> Result<usize, syn::Error> {
    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(v),
        ..
    }) = expr
    else {
        return Err(syn::Error::new_spanned(
            expr,
            "Expected unsigned integer literal in #[validate(len = ...)]",
        ));
    };

    v.base10_parse::<usize>()
        .map_err(|_| syn::Error::new_spanned(v, "Invalid usize literal"))
}

fn parse_string_literal(expr: &syn::Expr) -> Result<String, syn::Error> {
    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(v),
        ..
    }) = expr
    else {
        return Err(syn::Error::new_spanned(
            expr,
            "Expected string literal in #[validate(pattern = ...)]",
        ));
    };

    Ok(v.value())
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "String";
    }
    false
}

fn is_len_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let ident = segment.ident.to_string();
        return matches!(
            ident.as_str(),
            "String" | "Vec" | "HashMap" | "HashSet" | "BTreeMap"
        );
    }
    false
}

fn is_numeric_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let ident = segment.ident.to_string();
        return matches!(
            ident.as_str(),
            "u8" | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "f32"
                | "f64"
        );
    }
    false
}

fn process_item_for_bridge(item: &mut syn::Item) {
    match item {
        syn::Item::Fn(item_fn) => {
            if matches!(item_fn.vis, syn::Visibility::Public(_)) {
                add_bridge_if_missing(&mut item_fn.attrs);
            }
        }
        syn::Item::Struct(item_struct) => {
            if matches!(item_struct.vis, syn::Visibility::Public(_)) {
                add_bridge_if_missing(&mut item_struct.attrs);
            }
        }
        syn::Item::Enum(item_enum) => {
            if matches!(item_enum.vis, syn::Visibility::Public(_)) {
                add_bridge_if_missing(&mut item_enum.attrs);
            }
        }
        syn::Item::Impl(item_impl) => {
            add_bridge_if_missing(&mut item_impl.attrs);
        }
        syn::Item::Mod(item_mod) => {
            // Recursively process nested modules
            if let Some((_, items)) = &mut item_mod.content {
                for nested_item in items {
                    process_item_for_bridge(nested_item);
                }
            }
        }
        _ => {}
    }
}

fn add_bridge_if_missing(attrs: &mut Vec<syn::Attribute>) {
    let has_bridge = attrs
        .iter()
        .any(|attr| attr.path().is_ident("bridge") || attr.path().is_ident("export"));
    if !has_bridge {
        attrs.push(syn::parse_quote!(#[::bridgerust::bridge]));
    }
}

/// Enhanced export macro that generates bindings for both Python and Node.js
#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    process_export(attr, item)
}

fn process_export(attr: TokenStream, item: TokenStream) -> TokenStream {
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
        "#[bridge] / #[export] can only be applied to functions, structs, enums, or impl blocks",
    )
    .to_compile_error()
    .into()
}

fn helpful_error<T: quote::ToTokens>(
    span: &T,
    limitation: &str,
    workaround: &str,
    issue: Option<&str>,
) -> TokenStream {
    let mut msg = format!("{}\n\nWorkaround: {}", limitation, workaround);
    if let Some(url) = issue {
        msg.push_str(&format!("\n\nTrack support: {}", url));
    }

    syn::Error::new_spanned(span, msg).to_compile_error().into()
}

fn export_function(input_fn: ItemFn) -> TokenStream {
    // Validate function visibility
    if !matches!(input_fn.vis, syn::Visibility::Public(_)) {
        return syn::Error::new_spanned(
            &input_fn.sig.ident,
            "Functions exported with #[bridge] must be public (use `pub fn`)",
        )
        .to_compile_error()
        .into();
    }

    // Check for generic type parameters
    if !input_fn.sig.generics.params.is_empty() {
        return helpful_error(
            &input_fn.sig.generics,
            "Generic functions are not supported by BridgeRust yet.",
            "Use dynamic dispatch with Box<dyn Trait> or enum variants instead.",
            None,
        );
    }

    let is_async = input_fn.sig.asyncness.is_some();

    // Check if return type is an iterator
    let is_iterator = if let syn::ReturnType::Type(_, return_type) = &input_fn.sig.output {
        is_iterator_type(return_type)
    } else {
        false
    };

    if is_iterator && let syn::ReturnType::Type(_, return_type) = &input_fn.sig.output {
        return helpful_error(
            return_type,
            "Iterator return types used in export are not yet supported.",
            "Return Vec<T> instead.",
            None,
        );
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
                #fn_name(#(#call_args),*).await.map_err(Into::into)
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
            return Err(helpful_error(
                ty,
                &format!("Type {} requires a wrapper for cross-language use.", name),
                &format!(
                    "Use bridgerust::collections::{}Wrapper or convert to Vec<(K,V)>.\n\
                     \n\
                     Example:\n\
                     use bridgerust::collections::HashMapWrapper;\n\
                     \n\
                     #[bridge]\n\
                     pub fn get_map() -> HashMapWrapper<String, i32> {{\n\
                         // ...\n\
                     }}\n\
                     \n\
                     Or convert:\n\
                     pub fn get_map() -> Vec<(String, i32)> {{\n\
                         my_hashmap.into_iter().collect()\n\
                     }}",
                    name
                ),
                None,
            ));
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
    let mut validation_checks: Vec<proc_macro2::TokenStream> = Vec::new();
    let struct_ident = input_struct.ident.clone();

    if let syn::Fields::Named(fields) = &mut input_struct.fields {
        for field in &mut fields.named {
            if let Err(err) = validate_type(&field.ty, "struct field") {
                return err;
            }

            // Base attributes (common)
            let mut base_attrs = Vec::new();
            let mut is_readonly = false;
            let mut rules = ValidationRules::default();
            for attr in &field.attrs {
                if attr.path().is_ident("readonly") {
                    is_readonly = true;
                } else if attr.path().is_ident("validate") {
                    if let Err(err) = parse_validation_rules(attr, &mut rules) {
                        return err.to_compile_error().into();
                    }
                } else {
                    base_attrs.push(attr.clone());
                }
            }

            if rules != ValidationRules::default() {
                let Some(field_ident) = &field.ident else {
                    return syn::Error::new_spanned(field, "Named field expected")
                        .to_compile_error()
                        .into();
                };

                let field_name = field_ident.to_string();
                let ty = &field.ty;
                let field_expr = quote!(self.#field_ident);
                let option_type = is_option_type(ty);
                let string_type = is_string_type(ty);
                let len_type = is_len_type(ty);
                let numeric_type = is_numeric_type(ty);
                let min_value = rules.min;
                let max_value = rules.max;
                let len_value = rules.len;
                let pattern_value = rules.pattern.clone();

                let required_check = if rules.required {
                    if option_type {
                        quote! {
                            ::bridgerust::validation::required_value(#field_name, #field_expr.is_some())?;
                        }
                    } else if string_type {
                        quote! {
                            ::bridgerust::validation::required_value(
                                #field_name,
                                !#field_expr.trim().is_empty()
                            )?;
                        }
                    } else {
                        quote! {
                            ::bridgerust::validation::required_value(#field_name, true)?;
                        }
                    }
                } else {
                    quote! {}
                };

                let email_check = if rules.email {
                    if string_type {
                        quote! {
                            if !::bridgerust::validation::is_valid_email(&#field_expr) {
                                return Err(::bridgerust::BridgeError(
                                    format!("Validation failed for `{}`: invalid email", #field_name)
                                ));
                            }
                        }
                    } else {
                        return syn::Error::new_spanned(
                            ty,
                            "#[validate(email)] can only be applied to String fields",
                        )
                        .to_compile_error()
                        .into();
                    }
                } else {
                    quote! {}
                };

                let url_check = if rules.url {
                    if string_type {
                        quote! {
                            if !::bridgerust::validation::is_valid_url(&#field_expr) {
                                return Err(::bridgerust::BridgeError(
                                    format!("Validation failed for `{}`: invalid URL", #field_name)
                                ));
                            }
                        }
                    } else {
                        return syn::Error::new_spanned(
                            ty,
                            "#[validate(url)] can only be applied to String fields",
                        )
                        .to_compile_error()
                        .into();
                    }
                } else {
                    quote! {}
                };

                let min_check = if let Some(min) = min_value {
                    if numeric_type {
                        quote! {
                            ::bridgerust::validation::min_value(#field_name, #field_expr as f64, #min)?;
                        }
                    } else if len_type {
                        quote! {
                            ::bridgerust::validation::min_value(
                                #field_name,
                                #field_expr.len() as f64,
                                #min
                            )?;
                        }
                    } else {
                        return syn::Error::new_spanned(
                            ty,
                            "#[validate(min = ...)] expects a numeric, String, or collection field",
                        )
                        .to_compile_error()
                        .into();
                    }
                } else {
                    quote! {}
                };

                let max_check = if let Some(max) = max_value {
                    if numeric_type {
                        quote! {
                            ::bridgerust::validation::max_value(#field_name, #field_expr as f64, #max)?;
                        }
                    } else if len_type {
                        quote! {
                            ::bridgerust::validation::max_value(
                                #field_name,
                                #field_expr.len() as f64,
                                #max
                            )?;
                        }
                    } else {
                        return syn::Error::new_spanned(
                            ty,
                            "#[validate(max = ...)] expects a numeric, String, or collection field",
                        )
                        .to_compile_error()
                        .into();
                    }
                } else {
                    quote! {}
                };

                let len_check = if let Some(len) = len_value {
                    if len_type {
                        quote! {
                            ::bridgerust::validation::exact_len(#field_name, #field_expr.len(), #len)?;
                        }
                    } else {
                        return syn::Error::new_spanned(
                            ty,
                            "#[validate(len = ...)] expects a String or collection field",
                        )
                        .to_compile_error()
                        .into();
                    }
                } else {
                    quote! {}
                };

                let pattern_check = if let Some(pattern) = pattern_value {
                    if string_type {
                        quote! {
                            if !::bridgerust::validation::matches_pattern(&#field_expr, #pattern) {
                                return Err(::bridgerust::BridgeError(
                                    format!(
                                        "Validation failed for `{}`: value does not match pattern `{}`",
                                        #field_name,
                                        #pattern
                                    )
                                ));
                            }
                        }
                    } else {
                        return syn::Error::new_spanned(
                            ty,
                            "#[validate(pattern = ...)] can only be applied to String fields",
                        )
                        .to_compile_error()
                        .into();
                    }
                } else {
                    quote! {}
                };

                validation_checks.push(quote! {
                    #required_check
                    #email_check
                    #url_check
                    #min_check
                    #max_check
                    #len_check
                    #pattern_check
                });
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

        impl ::bridgerust::validation::RuntimeValidate for #struct_ident {
            fn runtime_validate(&self) -> ::bridgerust::Result<()> {
                #(#validation_checks)*
                Ok(())
            }
        }
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
        const _: () = {
            #[allow(unused_imports)]
            use ::bridgerust::{new, staticmethod};

            #[cfg_attr(feature = "python", ::bridgerust::pyo3::pymethods(crate = "::bridgerust::pyo3"))]
            #[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi)]
            #input_impl
        };
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
