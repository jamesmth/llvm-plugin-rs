use proc_macro::TokenStream;

use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;

use quote::{format_ident, quote, quote_spanned, ToTokens};

use syn::{parse_quote, Attribute, AttributeArgs, Error, Item, ItemImpl, ItemMod, Meta, MetaList};

/// Macro for defining a new LLVM plugin.
///
/// This macro must be used at the module level, and must define a `name` and `version`
/// parameters.
///
/// It will look for the `#[pass(name = ..)]` and `#[analysis]` attributes inside the module,
/// to generate code that, once executed by the [opt] tool, will register passes to the LLVM
/// pass manager.
///
/// [opt]: https://llvm.org/docs/NewPassManager.html#invoking-opt
///
/// # Examples
///
/// Using the outer attribute syntax:
///
/// ```
/// #[llvm_plugin::plugin(name = "some_name", version = "0.1")]
/// mod plugin {
///     # use llvm_plugin::{
///     #    LlvmModuleAnalysis, LlvmModulePass, ModuleAnalysisManager, PreservedAnalyses,
///     # };
///     # use llvm_plugin::inkwell::module::Module;
///     #[derive(Default)]
///     struct Pass1;
///
///     #[pass(name = "some_other_name")]
///     impl LlvmModulePass for Pass1 {
///         fn run_pass(
///             &self,
///             module: &mut Module,
///             manager: &ModuleAnalysisManager,
///         ) -> PreservedAnalyses {
///             todo!()
///         }
///     }
///
///     #[derive(Default)]
///     struct Analysis1;
///
///     #[analysis]
///     impl LlvmModuleAnalysis for Analysis1 {
///         fn run_analysis(
///             &self,
///             module: &Module,
///             manager: &ModuleAnalysisManager,
///         ) -> String {
///             todo!()
///         }
///     }
/// }
/// ```
///
/// Using the inner attribute syntax (requires `nightly`):
///
/// ```
/// #![feature(prelude_import)]
/// #![feature(custom_inner_attributes)]
/// #![llvm_plugin::plugin(name = "some_name", version = "0.1")]
///
/// # use llvm_plugin::{
/// #    LlvmModuleAnalysis, LlvmModulePass, ModuleAnalysisManager, PreservedAnalyses,
/// # };
/// # use llvm_plugin::inkwell::module::Module;
/// #[derive(Default)]
/// struct Pass1;
///
/// #[pass(name = "some_other_name")]
/// impl LlvmModulePass for Pass1 {
///     fn run_pass(
///         &self,
///         module: &mut Module,
///         manager: &ModuleAnalysisManager,
///     ) -> PreservedAnalyses {
///         todo!()
///     }
/// }
///
/// #[derive(Default)]
/// struct Analysis1;
///
/// #[analysis]
/// impl LlvmModuleAnalysis for Analysis1 {
///     fn run_analysis(
///         &self,
///         module: &Module,
///         manager: &ModuleAnalysisManager,
///     ) -> String {
///         todo!()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn plugin(attrs: TokenStream, input: TokenStream) -> TokenStream {
    match plugin_impl(attrs, input) {
        Ok(ts) => ts.into(),
        Err(e) => {
            let msg = e.to_string();
            quote_spanned! { e.span() => fn error() { std::compile_error!(#msg) } }.into()
        }
    }
}

fn plugin_impl(attrs: TokenStream, input: TokenStream) -> syn::Result<TokenStream2> {
    // parse_macro_input!() should not be used, since it generates code
    // containing compile_error!() instead of std::compile_error!(), which
    // is not compatible with inner attribute proc macros.

    let args = syn::parse_macro_input::parse(attrs)?;
    let (name, version) = match parse_plugin_args(args) {
        Some(parsed) => parsed?,
        None => return Err(Error::new(Span::call_site(), "`plugin` attr missing args")),
    };

    // try to parse the tokens as if we were called as an outer module attribute
    // (e.g. #[..] mod { .. })
    if let Ok(mut parsed_mod) = syn::parse::<ItemMod>(input.clone()) {
        let items = match &mut parsed_mod.content {
            Some((_, items)) => items,
            None => {
                return Err(Error::new_spanned(
                    parsed_mod,
                    "expecting module with items",
                ));
            }
        };
        expand_plugin_items(&name, &version, items)?;
        Ok(parsed_mod.into_token_stream())
    } else {
        // try to parse the tokens as if we were called as an inner custom attribute
        // (e.g. #![..])
        let mut file = syn::parse::<syn::File>(input)?;
        expand_plugin_items(&name, &version, &mut file.items)?;
        Ok(file.into_token_stream())
    }
}

fn expand_plugin_items(name: &str, version: &str, items: &mut Vec<Item>) -> syn::Result<()> {
    const CRATE_NAME: &str = "llvm_plugin";
    const MODULE_PASS_TRAIT: &str = "LlvmModulePass";
    const MODULE_ANALYSIS_TRAIT: &str = "LlvmModuleAnalysis";
    const FUNCTION_PASS_TRAIT: &str = "LlvmFunctionPass";
    const FUNCTION_ANALYSIS_TRAIT: &str = "LlvmFunctionAnalysis";

    let mut register_snippets = Vec::new();
    let mut new_items = Vec::new();

    for item in items.iter_mut() {
        let item_impl = match item {
            Item::Impl(item) => item,
            _ => continue,
        };

        match parse_llvm_plugin_attribute(&mut item_impl.attrs) {
            Some(Ok(name)) => match &item_impl.trait_ {
                Some((_, path, _)) if matches_path(path, &[CRATE_NAME, MODULE_PASS_TRAIT]) => {
                    process_llvm_module_pass_impl(
                        &name,
                        &mut new_items,
                        item_impl,
                        &mut register_snippets,
                    )
                }
                Some((_, path, _)) if matches_path(path, &[CRATE_NAME, FUNCTION_PASS_TRAIT]) => {
                    process_llvm_function_pass_impl(
                        &name,
                        &mut new_items,
                        item_impl,
                        &mut register_snippets,
                    )
                }
                _ => {
                    return Err(Error::new_spanned(
                        item_impl,
                        format!(
                            "expected impl of traits `{}`, or `{}`",
                            MODULE_PASS_TRAIT, FUNCTION_PASS_TRAIT,
                        ),
                    ))
                }
            },
            Some(Err(e)) => return Err(e),
            None => (),
        };

        if parse_llvm_analysis_attribute(&mut item_impl.attrs).is_none() {
            continue;
        }

        let analysis_id = match &item_impl.trait_ {
            Some((_, path, _)) if matches_path(path, &[CRATE_NAME, MODULE_ANALYSIS_TRAIT]) => {
                process_llvm_module_analysis_impl(
                    &mut new_items,
                    item_impl,
                    &mut register_snippets,
                )?
            }
            Some((_, path, _)) if matches_path(path, &[CRATE_NAME, FUNCTION_ANALYSIS_TRAIT]) => {
                process_llvm_function_analysis_impl(
                    &mut new_items,
                    item_impl,
                    &mut register_snippets,
                )?
            }
            _ => {
                return Err(Error::new_spanned(
                    item_impl,
                    format!(
                        "expected impl of traits `{}`, or `{}`",
                        MODULE_ANALYSIS_TRAIT, FUNCTION_ANALYSIS_TRAIT
                    ),
                ))
            }
        };

        new_items.push(parse_quote! {
            static #analysis_id: u8 = 0; // value not relevant
        });
    }

    items.extend(new_items);
    items.push(parse_quote! {
        #[no_mangle]
        extern "C" fn llvmGetPassPluginInfo() -> llvm_plugin::PassPluginLibraryInfo {
            # ( #register_snippets )*

            llvm_plugin::PassPluginLibraryInfo {
                api_version: llvm_plugin::get_llvm_plugin_api_version__(),
                plugin_name: #name.as_ptr(),
                plugin_version: #version.as_ptr(),
                plugin_registrar: llvm_plugin::get_llvm_plugin_registrar__(),
            }
        }
    });

    Ok(())
}

fn parse_plugin_args(args: AttributeArgs) -> Option<syn::Result<(String, String)>> {
    let mut args_iter = args.iter();

    let arg = args_iter.next()?;
    let name = match arg {
        syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
            path,
            lit: syn::Lit::Str(s),
            ..
        })) if path.is_ident("name") => s.value(),
        _ => {
            return Some(Err(Error::new_spanned(
                arg,
                "expected arg `name=\"value\"`",
            )))
        }
    };

    let arg = args_iter.next()?;
    let version = match arg {
        syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
            path,
            lit: syn::Lit::Str(s),
            ..
        })) if path.is_ident("version") => s.value(),
        _ => {
            return Some(Err(Error::new_spanned(
                arg,
                "expected arg `version=\"value\"`",
            )))
        }
    };

    Some(Ok((name, version)))
}

fn parse_llvm_plugin_attribute(attrs: &mut Vec<Attribute>) -> Option<Result<String, Error>> {
    for (id, attr) in attrs.iter().enumerate() {
        let args = match attr.parse_meta() {
            Ok(Meta::List(MetaList { path, nested, .. })) if path.is_ident("pass") => nested,
            _ => continue,
        };

        let mut args_iter = args.iter();

        let name = match args_iter.next() {
            Some(arg) => match arg {
                syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                    path,
                    lit: syn::Lit::Str(s),
                    ..
                })) if path.is_ident("name") => s.value(),
                _ => {
                    return Some(Err(Error::new_spanned(
                        arg,
                        "expected arg `name=\"value\"`",
                    )))
                }
            },
            None => return Some(Err(Error::new_spanned(&args, "`pass` attr missing args"))),
        };

        attrs.remove(id);
        return Some(Ok(name));
    }

    None
}

fn parse_llvm_analysis_attribute(attrs: &mut Vec<Attribute>) -> Option<()> {
    for (id, attr) in attrs.iter().enumerate() {
        match attr.parse_meta() {
            Ok(Meta::Path(path)) if path.is_ident("analysis") => {
                attrs.remove(id);
                return Some(());
            }
            _ => continue,
        }
    }
    None
}

fn process_llvm_module_pass_impl(
    name: &str,
    module_items: &mut Vec<Item>,
    item_impl: &ItemImpl,
    register_snippets: &mut Vec<TokenStream2>,
) {
    let ItemImpl { self_ty, .. } = item_impl;
    let entrypoint = format_ident!("{}_entrypoint", name.replace('-', "_"));

    module_items.push(parse_quote! {
        extern "C" fn #entrypoint(
            module: *mut std::ffi::c_void,
            manager: *mut std::ffi::c_void,
        ) -> llvm_plugin::PreservedAnalyses {
            let mut module = unsafe { llvm_plugin::inkwell::module::Module::new(module.cast()) };
            let pass = #self_ty::default();
            let manager = unsafe { llvm_plugin::ModuleAnalysisManager::from_raw(manager, None) };
            use llvm_plugin::LlvmModulePass;
            let preserve = pass.run_pass(&mut module, &manager);
            std::mem::forget(module);
            preserve
        }
    });

    register_snippets.push(quote! {
        llvm_plugin::register_module_pass__(#name, #entrypoint);
    });
}

fn process_llvm_function_pass_impl(
    name: &str,
    module_items: &mut Vec<Item>,
    item_impl: &ItemImpl,
    register_snippets: &mut Vec<TokenStream2>,
) {
    let ItemImpl { self_ty, .. } = item_impl;
    let entrypoint = format_ident!("{}_entrypoint", name.replace('-', "_"));

    module_items.push(parse_quote! {
        extern "C" fn #entrypoint(
            function: *mut std::ffi::c_void,
            manager: *mut std::ffi::c_void,
        ) -> llvm_plugin::PreservedAnalyses {
            let mut function = unsafe { llvm_plugin::inkwell::values::FunctionValue::new(function.cast()).unwrap() };
            let pass = #self_ty::default();
            let manager = unsafe { llvm_plugin::FunctionAnalysisManager::from_raw(manager, None) };
            use llvm_plugin::LlvmFunctionPass;
            let preserve = pass.run_pass(&mut function, &manager);
            std::mem::forget(function);
            preserve
        }
    });

    register_snippets.push(quote! {
        llvm_plugin::register_function_pass__(#name, #entrypoint);
    });
}

fn process_llvm_module_analysis_impl(
    module_items: &mut Vec<Item>,
    item_impl: &mut ItemImpl,
    register_snippets: &mut Vec<TokenStream2>,
) -> syn::Result<syn::Ident> {
    let ItemImpl { self_ty, items, .. } = item_impl;

    // get trait implementor's name
    let analysis_ident = match self_ty.as_ref() {
        syn::Type::Path(path) => path
            .path
            .get_ident()
            .ok_or_else(|| Error::new_spanned(&self_ty, "expected single-ident type"))?
            .clone(),
        _ => return Err(Error::new_spanned(&item_impl, "expected path-like type")),
    };

    let mut return_ty = None;
    for item in items.iter() {
        match item {
            syn::ImplItem::Method(syn::ImplItemMethod {
                sig:
                    syn::Signature {
                        ident,
                        output: syn::ReturnType::Type(_, ty),
                        ..
                    },
                ..
            }) if ident == "run_analysis" => return_ty = Some(ty.clone()),
            _ => continue,
        }
    }

    // preprend the crate name to the analysis name, to avoid symbol resolution conflicts
    let analysis_id = analysis_ident.to_string().to_uppercase();
    let crate_name = std::env::var("CARGO_CRATE_NAME").expect("CARGO_CRATE_NAME not set");
    let analysis_id = format_ident!("{}_{}", crate_name.to_uppercase(), analysis_id);

    items.push(parse_quote! {
        type Result = #return_ty;
    });
    items.push(parse_quote! {
        fn id() -> llvm_plugin::AnalysisKey {
            &#analysis_id as *const u8 as *mut _
        }
    });

    let entrypoint = format_ident!("{}_entrypoint", analysis_ident.to_string().to_lowercase());

    module_items.push(parse_quote! {
        extern "C" fn #entrypoint(
            module: *mut std::ffi::c_void,
            manager: *mut std::ffi::c_void,
            result: *mut *mut std::ffi::c_void,
            deleter: *mut extern "C" fn(*mut std::ffi::c_void),
        ) {
            use llvm_plugin::LlvmModuleAnalysis;
            let mut module = unsafe { llvm_plugin::inkwell::module::Module::new(module.cast()) };
            let analysis = #self_ty::default(); // FIXME
            let manager = unsafe { llvm_plugin::ModuleAnalysisManager::from_raw(manager, Some(#self_ty::id())) };
            let data = analysis.run_analysis(&mut module, &manager);
            let data = Box::new(data);
            extern "C" fn free(data: *mut std::ffi::c_void) {
                drop(unsafe { Box::<<#self_ty as LlvmModuleAnalysis>::Result>::from_raw(data.cast()) })
            }
            unsafe {
                *result = Box::<<#self_ty as LlvmModuleAnalysis>::Result>::into_raw(data).cast();
                *deleter = free;
            }
            std::mem::forget(module);
        }
    });

    register_snippets.push(quote! {
        use llvm_plugin::LlvmModuleAnalysis;
        unsafe { llvm_plugin::register_module_analysis__(#self_ty::id(), #entrypoint) };
    });

    Ok(analysis_id)
}

fn process_llvm_function_analysis_impl(
    module_items: &mut Vec<Item>,
    item_impl: &mut ItemImpl,
    register_snippets: &mut Vec<TokenStream2>,
) -> syn::Result<syn::Ident> {
    let ItemImpl { self_ty, items, .. } = item_impl;

    // get trait implementor's name
    let analysis_ident = match self_ty.as_ref() {
        syn::Type::Path(path) => path
            .path
            .get_ident()
            .ok_or_else(|| Error::new_spanned(&self_ty, "expected single-ident type"))?
            .clone(),
        _ => return Err(Error::new_spanned(&item_impl, "expected path-like type")),
    };

    let mut return_ty = None;
    for item in items.iter() {
        match item {
            syn::ImplItem::Method(syn::ImplItemMethod {
                sig:
                    syn::Signature {
                        ident,
                        output: syn::ReturnType::Type(_, ty),
                        ..
                    },
                ..
            }) if ident == "run_analysis" => return_ty = Some(ty.clone()),
            _ => continue,
        }
    }

    // preprend the crate name to the analysis name, to avoid symbol resolution conflicts
    let analysis_id = analysis_ident.to_string().to_uppercase();
    let crate_name = std::env::var("CARGO_CRATE_NAME").expect("CARGO_CRATE_NAME not set");
    let analysis_id = format_ident!("{}_{}", crate_name.to_uppercase(), analysis_id);

    items.push(parse_quote! {
        type Result = #return_ty;
    });
    items.push(parse_quote! {
        fn id() -> llvm_plugin::AnalysisKey {
            &#analysis_id as *const u8 as *mut _
        }
    });

    let entrypoint = format_ident!("{}_entrypoint", analysis_ident.to_string().to_lowercase());

    module_items.push(parse_quote! {
        extern "C" fn #entrypoint(
            function: *mut std::ffi::c_void,
            manager: *mut std::ffi::c_void,
            result: *mut *mut std::ffi::c_void,
            deleter: *mut extern "C" fn(*mut std::ffi::c_void),
        ) {
            use llvm_plugin::LlvmFunctionAnalysis;
            let mut function = unsafe { llvm_plugin::inkwell::values::FunctionValue::new(function.cast()).unwrap() };
            let analysis = #self_ty::default(); // FIXME
            let manager = unsafe { llvm_plugin::FunctionAnalysisManager::from_raw(manager, Some(#self_ty::id())) };
            let data = analysis.run_analysis(&mut function, &manager);
            let data = Box::new(data);
            extern "C" fn free(data: *mut std::ffi::c_void) {
                drop(unsafe { Box::<<#self_ty as LlvmFunctionAnalysis>::Result>::from_raw(data.cast()) })
            }
            unsafe {
                *result = Box::<<#self_ty as LlvmFunctionAnalysis>::Result>::into_raw(data).cast();
                *deleter = free;
            }
            std::mem::forget(function);
        }
    });

    register_snippets.push(quote! {
        use llvm_plugin::LlvmFunctionAnalysis;
        unsafe { llvm_plugin::register_function_analysis__(#self_ty::id(), #entrypoint) };
    });

    Ok(analysis_id)
}

fn matches_path(path: &syn::Path, pattern: &[&str]) -> bool {
    !path
        .segments
        .iter()
        .rev()
        .peekable()
        .zip(pattern.iter().rev())
        .any(|(seg, s)| seg.ident != s)
}
