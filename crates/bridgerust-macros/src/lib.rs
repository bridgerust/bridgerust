use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemEnum, ItemFn, ItemStruct};

/// Enhanced export macro that generates bindings for both Python and Node.js
///
/// Can be applied to:
/// - Functions: Generates `#[pyfunction]` and `#[napi]` attributes
/// - Structs: Generates `#[pyclass]` and `#[napi]` attributes
/// - Enums: Generates `#[pyclass]` and `#[napi]` attributes
///
/// Usage:
/// ```rust,ignore
/// #[bridgerust::export]
/// pub fn greet(name: String) -> String { ... }
///
/// #[bridgerust::export]
/// pub struct Point {
///     pub x: f64,
///     pub y: f64,
/// }
///
/// #[bridgerust::export]
/// pub enum Status {
///     Success,
///     Error(String),
/// }
/// ```
#[proc_macro_attribute]
pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Try to parse as function first
    if let Ok(input_fn) = syn::parse::<ItemFn>(item.clone()) {
        return export_function(input_fn);
    }

    // Try to parse as struct
    if let Ok(input_struct) = syn::parse::<ItemStruct>(item.clone()) {
        return export_struct(input_struct);
    }

    // Try to parse as enum
    if let Ok(input_enum) = syn::parse::<ItemEnum>(item.clone()) {
        return export_enum(input_enum);
    }

    // If none work, try as generic item and provide helpful error
    let input = parse_macro_input!(item as DeriveInput);
    syn::Error::new_spanned(
        &input.ident,
        "#[bridgerust::export] can only be applied to functions, structs, or enums",
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
        let generic_params: Vec<String> = input_fn
            .sig
            .generics
            .params
            .iter()
            .map(|param| match param {
                syn::GenericParam::Type(type_param) => type_param.ident.to_string(),
                syn::GenericParam::Lifetime(lifetime) => format!("'{}", lifetime.lifetime.ident),
                syn::GenericParam::Const(const_param) => const_param.ident.to_string(),
            })
            .collect();

        let generic_list = generic_params.join(", ");

        return syn::Error::new_spanned(
            &input_fn.sig.generics,
            format!(
                "Generic functions (with type parameters like <{}>) are not directly supported by BridgeRust. \
                 \n\nPyO3 and napi-rs don't support generic types directly because:\n\
                 1. Rust generics are monomorphized at compile time\n\
                 2. Python and JavaScript use dynamic typing\n\
                 3. Type information is lost at the FFI boundary\n\
                 \n\nSolutions:\n\
                 1. **Specialize for concrete types** (Recommended): Create separate functions for each type you need\n\
                 2. **Use trait objects**: Convert generics to trait objects at the boundary\n\
                 3. **Use enums**: Represent different types with an enum\n\
                 4. **Manual bindings**: Implement target-specific bindings manually\n\
                 \n\nExample - Specialization:\n\
                 // Instead of:\n\
                 // #[export]\n\
                 // pub fn process<T>(item: T) -> T {{ ... }}\n\
                 \n\
                 // Use:\n\
                 #[export]\n\
                 pub fn process_i32(item: i32) -> i32 {{ /* ... */ }}\n\
                 \n\
                 #[export]\n\
                 pub fn process_string(item: String) -> String {{ /* ... */ }}\n\
                 \n\
                 // Or use a macro to generate specializations:\n\
                 macro_rules! export_process {{\n\
                     ($($t:ty),*) => {{\n\
                         $(#[export] pub fn process_$t(item: $t) -> $t {{ /* ... */ }})*\n\
                     }};\n\
                 }}\n\
                 export_process!(i32, String, f64);\n\
                 \n\nSee docs/GENERICS.md for more details and examples.",
                generic_list
            )
        ).to_compile_error().into();
    }

    let is_async = input_fn.sig.asyncness.is_some();

    // Check if return type is an iterator
    let is_iterator = if let syn::ReturnType::Type(_, return_type) = &input_fn.sig.output {
        is_iterator_type(return_type)
    } else {
        false
    };

    // If iterator return type, provide helpful guidance
    if is_iterator {
        if let syn::ReturnType::Type(_, return_type) = &input_fn.sig.output {
            return syn::Error::new_spanned(
                return_type,
                "Iterator return types (impl Iterator<Item = T>) are not yet directly supported. \
                 \n\nFor now, you have two options:\n\
                 1. Return Vec<T> instead: Change your function to return Vec<T> and collect the iterator\n\
                 2. Implement custom streaming: See docs/STREAMING.md for manual implementation\n\
                 \nExample:\n\
                 // Instead of: pub fn numbers() -> impl Iterator<Item = i32>\n\
                 // Use: pub fn numbers() -> Vec<i32> { (0..10).collect() }\n\
                 \nFor true streaming support (generators/async iterators), this will be enhanced in a future release."
            ).to_compile_error().into();
        }
    }

    // Validate return type
    if let syn::ReturnType::Type(_, return_type) = &input_fn.sig.output {
        if let Err(err) = validate_type(return_type, "return type") {
            return err;
        }
    }

    // Validate input types
    for input in &input_fn.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if let Err(err) = validate_type(&pat_type.ty, "parameter type") {
                return err;
            }
        }
    }

    if is_async {
        // Handle async functions
        export_async_function(input_fn)
    } else {
        // Handle sync functions
        let expanded = quote! {
            #[cfg_attr(feature = "python", ::bridgerust::pyo3::pyfunction(crate = "::bridgerust::pyo3"))]
            #[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi)]
            #input_fn
        };
        TokenStream::from(expanded)
    }
}

fn export_async_function(input_fn: ItemFn) -> TokenStream {
    // For Node.js: napi-rs supports async functions directly with #[napi]
    // For Python: We need to generate a wrapper using pyo3-async-runtimes

    // Extract function components
    let fn_name = &input_fn.sig.ident;
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;

    // Build parameter list for the wrapper function
    let mut wrapper_params = Vec::new();
    let mut call_args = Vec::new();

    for input in &fn_sig.inputs {
        match input {
            syn::FnArg::Receiver(_) => {
                // Methods not supported yet
                return syn::Error::new_spanned(
                    fn_sig,
                    "Methods (functions with &self) are not yet supported in async functions",
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
        // Original async function (kept for internal use)
        #(#fn_attrs)*
        #fn_vis #fn_sig #fn_block

        // Node.js binding (napi-rs supports async directly)
        #[cfg(feature = "nodejs")]
        #[bridgerust::napi_derive::napi]
        #(#fn_attrs)*
        #fn_vis #fn_sig #fn_block

        // Python binding wrapper (requires pyo3-async-runtimes)
        #[cfg(feature = "python")]
        #[bridgerust::pyo3::pyfunction(crate = "bridgerust::pyo3")]
        pub fn #fn_name(
            py: bridgerust::pyo3::Python<'_>,
            #(#wrapper_params),*
        ) -> bridgerust::pyo3::PyResult<bridgerust::pyo3::PyObject> {
            use bridgerust::pyo3::IntoPy;
            pyo3_async_runtimes::tokio::future_into_py(py, async move {
                #fn_name(#(#call_args),*).await.into_py(py)
            })
        }
    };

    TokenStream::from(expanded)
}

/// Check if a type is an iterator (impl Iterator<Item = T> or similar)
fn is_iterator_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::ImplTrait(impl_trait) => {
            // Check for `impl Iterator<Item = T>`
            for bound in &impl_trait.bounds {
                if let syn::TypeParamBound::Trait(trait_bound) = bound {
                    if let Some(segment) = trait_bound.path.segments.last() {
                        if segment.ident == "Iterator" {
                            return true;
                        }
                    }
                }
            }
            false
        }
        syn::Type::Path(type_path) => {
            // Check for Box<dyn Iterator<Item = T>> or similar
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();
                // Check for common iterator types
                if type_name == "Iterator"
                    || type_name.contains("Iter")
                    || type_name == "IntoIterator"
                {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// Validate that a type is supported by BridgeRust
/// This function recursively checks nested types (e.g., Vec<HashMap<K, V>>)
fn validate_type(ty: &syn::Type, context: &str) -> Result<(), TokenStream> {
    // Check for unsupported patterns
    match ty {
        // Generic types - check nested types
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();

                // Check for known unsupported types
                match type_name.as_str() {
                    "HashMap" | "HashSet" | "BTreeMap" | "BTreeSet" => {
                        return Err(syn::Error::new_spanned(
                            ty,
                            format!(
                                "{} `{}` is not directly supported by BridgeRust. \
                                 Consider using `Vec<(K, V)>` for HashMap or `Vec<T>` for HashSet. \
                                 For advanced use cases, implement bindings manually.",
                                context, type_name
                            ),
                        )
                        .to_compile_error()
                        .into());
                    }
                    // Allow callback types
                    "PyObject" | "PyAny" | "Function" | "JsFunction" => {
                        // These are callback types - allowed for callback support
                        return Ok(());
                    }
                    // Check Vec<T> - validate the inner type T
                    "Vec" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                // Recursively validate the inner type
                                return validate_type(
                                    inner_ty,
                                    &format!("{} element type (in Vec)", context),
                                );
                            }
                        }
                    }
                    // Check Option<T> - validate the inner type T
                    "Option" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                // Recursively validate the inner type
                                return validate_type(
                                    inner_ty,
                                    &format!("{} element type (in Option)", context),
                                );
                            }
                        }
                    }
                    // Check Result<T, E> - validate both types
                    "Result" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            // Validate Ok type
                            if let Some(syn::GenericArgument::Type(ok_ty)) = args.args.first() {
                                validate_type(ok_ty, &format!("{} Ok type (in Result)", context))?
                            }
                            // Validate Err type
                            if let Some(syn::GenericArgument::Type(err_ty)) = args.args.get(1) {
                                validate_type(err_ty, &format!("{} Err type (in Result)", context))?
                            }
                        }
                    }
                    _ => {}
                }

                // Check for callback types in paths (e.g., pyo3::PyObject, napi::Function)
                let full_path = type_path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");

                if full_path.contains("PyObject")
                    || full_path.contains("PyAny")
                    || full_path.contains("Function")
                    || full_path.contains("JsFunction")
                {
                    // Allow callback types
                    return Ok(());
                }
            }
        }
        // Function pointers - allow for callbacks (users need to handle conversion manually)
        syn::Type::BareFn(_) => {
            // Function pointers are allowed but require manual conversion
            // Users should use PyObject/napi::Function for callbacks instead
        }
        // Trait objects - allow in certain contexts with guidance
        syn::Type::TraitObject(trait_obj) => {
            // Check if it's Box<dyn Trait> which can sometimes work
            // For now, provide helpful error with suggestions
            let trait_name =
                if let Some(syn::TypeParamBound::Trait(trait_bound)) = trait_obj.bounds.first() {
                    if let Some(path_segment) = trait_bound.path.segments.last() {
                        path_segment.ident.to_string()
                    } else {
                        "Trait".to_string()
                    }
                } else {
                    "Trait".to_string()
                };

            return Err(syn::Error::new_spanned(
                ty,
                format!(
                    "{} trait objects (dyn {}) are not directly supported across language boundaries. \
                     \n\nSuggestions:\n\
                     1. Use concrete types instead of trait objects\n\
                     2. Use an enum to represent different implementations\n\
                     3. Serialize the trait object to a concrete type (JSON, etc.)\n\
                     4. For internal Rust code, keep trait objects but convert at the boundary\n\
                     5. Implement bindings manually with target-specific types\n\
                     \nExample:\n\
                     // Instead of: fn process(obj: Box<dyn Trait>) -> String\n\
                     // Use: fn process(obj: ConcreteType) -> String\n\
                     // Or: fn process(obj: TraitEnum) -> String",
                    context, trait_name
                )
            ).to_compile_error().into());
        }
        _ => {}
    }

    Ok(())
}

fn export_struct(input_struct: ItemStruct) -> TokenStream {
    // Validate struct visibility
    if !matches!(input_struct.vis, syn::Visibility::Public(_)) {
        return syn::Error::new_spanned(
            &input_struct.ident,
            "Structs exported with #[bridgerust::export] must be public (use `pub struct`)",
        )
        .to_compile_error()
        .into();
    }

    // Validate struct field types
    if let syn::Fields::Named(fields) = &input_struct.fields {
        for field in &fields.named {
            if let Err(err) = validate_type(
                &field.ty,
                &format!(
                    "field `{}`",
                    field
                        .ident
                        .as_ref()
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                ),
            ) {
                return err;
            }
        }
    } else if let syn::Fields::Unnamed(fields) = &input_struct.fields {
        for (idx, field) in fields.unnamed.iter().enumerate() {
            if let Err(err) = validate_type(&field.ty, &format!("field #{}", idx)) {
                return err;
            }
        }
    }

    // Check for generic type parameters (Same as before)
    if !input_struct.generics.params.is_empty() {
        // ... (Omitting full error message for brevity in tool call, but conceptually same block)
        let generic_params: Vec<String> = input_struct
            .generics
            .params
            .iter()
            .map(|param| match param {
                syn::GenericParam::Type(type_param) => type_param.ident.to_string(),
                syn::GenericParam::Lifetime(lifetime) => format!("'{}", lifetime.lifetime.ident),
                syn::GenericParam::Const(const_param) => const_param.ident.to_string(),
            })
            .collect();

        let generic_list = generic_params.join(", ");

        return syn::Error::new_spanned(
            &input_struct.generics,
            format!(
                "Generic structs (with type parameters like <{}>) are not directly supported by BridgeRust. \
                 \n\nPyO3 and napi-rs don't support generic types directly because:\n\
                 1. Rust generics are monomorphized at compile time\n\
                 2. Python and JavaScript use dynamic typing\n\
                 3. Type information is lost at the FFI boundary\n\
                 \n\nSolutions:\n\
                 1. **Specialize for concrete types** (Recommended): Create separate structs for each type\n\
                 2. **Use trait objects**: Convert generics to trait objects at the boundary\n\
                 3. **Use enums**: Represent different types with an enum\n\
                 4. **Manual bindings**: Implement target-specific bindings manually\n\
                 \n\nExample - Specialization:\n\
                 // Instead of:\n\
                 // #[export]\n\
                 // pub struct Container<T> {{ value: T }}\n\
                 \n\
                 // Use:\n\
                 #[export]\n\
                 pub struct ContainerI32 {{ value: i32 }}\n\
                 \n\
                 #[export]\n\
                 pub struct ContainerString {{ value: String }}\n\
                 \n\
                 // Or use a macro to generate specializations\n\
                 \n\nSee docs/GENERICS.md for more details and examples.",
                generic_list
            )
        ).to_compile_error().into();
    }

    // Check for unsupported struct types
    if let syn::Fields::Unnamed(_) = &input_struct.fields {
        return syn::Error::new_spanned(
            &input_struct.ident,
            "Tuple structs are not supported by #[bridgerust::export]. \
             Use a regular struct with named fields instead",
        )
        .to_compile_error()
        .into();
    }

    let expanded = quote! {
        #[cfg_attr(feature = "python", ::bridgerust::pyo3::pyclass(crate = "::bridgerust::pyo3", get_all, set_all))]
        #[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi)]
        #input_struct
    };
    TokenStream::from(expanded)
}

fn export_enum(input_enum: ItemEnum) -> TokenStream {
    // Validate enum visibility
    if !matches!(input_enum.vis, syn::Visibility::Public(_)) {
        return syn::Error::new_spanned(
            &input_enum.ident,
            "Enums exported with #[bridgerust::export] must be public (use `pub enum`)",
        )
        .to_compile_error()
        .into();
    }

    // Check for generic type parameters
    if !input_enum.generics.params.is_empty() {
        let generic_params: Vec<String> = input_enum
            .generics
            .params
            .iter()
            .map(|param| match param {
                syn::GenericParam::Type(type_param) => type_param.ident.to_string(),
                syn::GenericParam::Lifetime(lifetime) => format!("'{}", lifetime.lifetime.ident),
                syn::GenericParam::Const(const_param) => const_param.ident.to_string(),
            })
            .collect();

        let generic_list = generic_params.join(", ");

        return syn::Error::new_spanned(
            &input_enum.generics,
            format!(
                "Generic enums (with type parameters like <{}>) are not directly supported by BridgeRust. \
                 \n\nPyO3 and napi-rs don't support generic types directly because:\n\
                 1. Rust generics are monomorphized at compile time\n\
                 2. Python and JavaScript use dynamic typing\n\
                 3. Type information is lost at the FFI boundary\n\
                 \n\nSolutions:\n\
                 1. **Specialize for concrete types** (Recommended): Create separate enums for each type\n\
                 2. **Use trait objects**: Convert generics to trait objects at the boundary\n\
                 3. **Use nested enums**: Represent different types with nested enum variants\n\
                 4. **Manual bindings**: Implement target-specific bindings manually\n\
                 \n\nExample - Specialization:\n\
                 // Instead of:\n\
                 // #[export]\n\
                 // pub enum Result<T, E> {{ Ok(T), Err(E) }}\n\
                 \n\
                 // Use concrete types:\n\
                 #[export]\n\
                 pub enum StringResult {{ Ok(String), Err(String) }}\n\
                 \n\
                 // Or use a macro to generate specializations\n\
                 \n\nSee docs/GENERICS.md for more details and examples.",
                generic_list
            )
        ).to_compile_error().into();
    }

    let expanded = quote! {
        #[cfg_attr(feature = "python", ::bridgerust::pyo3::pyclass(crate = "::bridgerust::pyo3"))]
        #[cfg_attr(feature = "nodejs", ::bridgerust::napi_derive::napi)]
        #input_enum
    };
    TokenStream::from(expanded)
}

/// Error macro that generates basic error conversion helpers for Python and Node.js
///
/// This macro requires the error type to implement Display (via thiserror or manually).
/// It generates simple conversion functions that use the Display implementation.
///
/// Usage:
/// ```rust,ignore
/// use thiserror::Error;
///
/// #[bridgerust::error]
/// #[derive(Error, Debug)]
/// pub enum MyError {
///     #[error("Config error: {0}")]
///     Config(String),
///     #[error("Database error: {0}")]
///     Database(String),
/// }
/// ```
///
/// This generates `to_py_err()` and `to_napi_err()` helper functions.
#[proc_macro_attribute]
pub fn error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let enum_name = &input.ident;

    // Verify it's an enum
    if !matches!(input.data, syn::Data::Enum(_)) {
        return syn::Error::new_spanned(
            &input.ident,
            "#[bridgerust::error] can only be applied to enums",
        )
        .to_compile_error()
        .into();
    }

    // Generate simple conversion functions that use Display trait
    let expanded = quote! {
        #input

        // Python error conversion helper
        #[cfg(feature = "python")]
        #[allow(unexpected_cfgs)]
        /// Convert Rust error to Python exception
        ///
        /// This function uses the error's Display implementation to create a Python exception.
        /// For more fine-grained control, implement your own conversion function.
        pub fn to_py_err(err: #enum_name) -> bridgerust::pyo3::PyErr {
            use std::fmt::Display;
            bridgerust::pyo3::exceptions::PyRuntimeError::new_err(err.to_string())
        }

        // Node.js error conversion helper
        #[cfg(feature = "nodejs")]
        #[allow(unexpected_cfgs)]
        /// Convert Rust error to Node.js error
        ///
        /// This function uses the error's Display implementation to create a Node.js error.
        /// For more fine-grained control, implement your own conversion function.
        pub fn to_napi_err(err: #enum_name) -> bridgerust::napi::Error {
            use std::fmt::Display;
            bridgerust::napi::Error::from_reason(err.to_string())
        }
    };

    TokenStream::from(expanded)
}
