//! [![github]](https://github.com/jamesmth/llvm-plugin-rs)&ensp;[![crates-io]](https://crates.io/crates/llvm-plugin)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//!
//! <br>
//!
//! This crate gives the ability to safely implement passes for the [new LLVM pass manager],
//! by leveraging the strongly typed interface  provided by [Inkwell].
//!
//! If you have never developed LLVM passes before, perhaps you should take a look at this
//! [LLVM guide] before carrying on. It will give you a simple overview of the C++ API
//! wrapped by this crate.
//!
//! If you want a deeper understanding of the many concepts surrounding the new LLVM pass manager,
//! you should read the [official LLVM documentation].
//!
//! [Inkwell]: https://github.com/TheDan64/inkwell
//! [new LLVM pass manager]: https://blog.llvm.org/posts/2021-03-26-the-new-pass-manager/
//! [LLVM guide]: https://llvm.org/docs/WritingAnLLVMNewPMPass.html
//! [official LLVM documentation]: https://llvm.org/docs/NewPassManager.html
//!
//! # Example
//!
//! A simple LLVM plugin which defines two passes, one being a transformation pass that queries
//! the result of a second pass, an analysis one:
//!
//! ```
//! // Define an LLVM plugin (a name and a version is required). Only cdylib crates
//! // should define plugins, and only one definition should be done per crate.
//! #[llvm_plugin::plugin(name = "plugin_name", version = "0.1")]
//! mod plugin {
//!     use llvm_plugin::{
//!         LlvmModuleAnalysis, LlvmModulePass, ModuleAnalysisManager, PreservedAnalyses,
//!     };
//!     use llvm_plugin::inkwell::module::Module;
//!
//!     // Must implement the `Default` trait.
//!     #[derive(Default)]
//!     struct Pass1;
//!
//!     // Define a transformation pass (a name is required). Such pass is allowed to
//!     // mutate the LLVM IR. If it does, it should return `PreservedAnalysis::None`
//!     // to notify the pass manager that all analyses are now invalidated.
//!     #[pass(name = "pass_name")]
//!     impl LlvmModulePass for Pass1 {
//!         fn run_pass(
//!             &self,
//!             module: &mut Module,
//!             manager: &ModuleAnalysisManager,
//!         ) -> PreservedAnalyses {
//!             // Ask the pass manager for the result of the analysis pass `Analysis1`
//!             // defined further below. If the result is not in cache, the pass
//!             // manager will call `Analysis1::run_analysis`.
//!             let result = manager.get_result::<Analysis1>(module);
//!
//!             assert_eq!(result, "Hello World!");
//!
//!             // no modification was made on the module, so the pass manager doesn't have
//!             // to recompute any analysis
//!             PreservedAnalyses::All
//!         }
//!     }
//!
//!     // Must implement the `Default` trait.
//!     #[derive(Default)]
//!     struct Analysis1;
//!
//!     // Define an analysis pass. Such pass is not allowed to mutate the LLVM IR. It should
//!     // be used only for inspection of the LLVM IR, and can return some result that will be
//!     // efficiently cached by the pass manager (to prevent recomputing the same analysis
//!     // every time its result is needed).
//!     #[analysis]
//!     impl LlvmModuleAnalysis for Analysis1 {
//!         fn run_analysis(
//!             &self,
//!             module: &Module,
//!             manager: &ModuleAnalysisManager,
//!         ) -> String {
//!             // .. inspect the LLVM IR of the module ..
//!
//!             "Hello World!".to_owned()
//!         }
//!     }
//! }
//! ```

#![deny(missing_docs)]

mod analysis;
pub use analysis::*;

#[doc(hidden)]
pub mod ffi;
#[doc(hidden)]
pub use ffi::*;

pub use inkwell;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

/// Enum specifying whether analyses on an IR unit are not preserved due
/// to the modification of such unit by a transformation pass.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum PreservedAnalyses {
    /// This variant hints the pass manager that all the analyses are
    /// preserved, so there is no need to re-execute analysis passes.
    ///
    /// Use this variant when a transformation pass doesn't modify some
    /// IR unit.
    All,

    /// This variant hints the pass manager that all the analyses are
    /// should be re-executed.
    ///
    /// Use this variant when a transformation pass modifies some IR unit.
    None,
}

/// Trait to use for implementing a transformation pass on an LLVM module.
///
/// A transformation pass is allowed to mutate the LLVM IR.
pub trait LlvmModulePass: Default {
    /// Entrypoint for the pass.
    ///
    /// The given analysis manager allows the pass to query the pass
    /// manager for the result of specific analysis passes.
    ///
    /// If this function makes modifications on the given module IR, it
    /// should return `PreservedAnalyses::None` to indicate to the
    /// pass manager that all analyses are now invalidated.
    fn run_pass<'a>(
        &self,
        module: &mut Module<'a>,
        manager: &ModuleAnalysisManager,
    ) -> PreservedAnalyses;
}

/// Trait to use for implementing a transformation pass on an LLVM function.
///
/// A transformation pass is allowed to mutate the LLVM IR.
pub trait LlvmFunctionPass: Default {
    /// Entrypoint for the pass.
    ///
    /// The given analysis manager allows the pass to query the pass
    /// manager for the result of specific analysis passes.
    ///
    /// If this function makes modifications on the given function IR, it
    /// should return `PreservedAnalyses::None` to indicate to the
    /// pass manager that all analyses are now invalidated.
    fn run_pass<'a>(
        &self,
        function: &mut FunctionValue<'a>,
        manager: &FunctionAnalysisManager,
    ) -> PreservedAnalyses;
}

/// Trait to use for implementing an analysis pass on an LLVM module.
///
/// An analysis pass is not allowed to mutate the LLVM IR.
pub trait LlvmModuleAnalysis: Default {
    /// Result of the successful execution of this pass by the pass manager.
    ///
    /// This data can be queried by passes through a [`ModuleAnalysisManager`].
    type Result;

    /// Entrypoint for the pass.
    ///
    /// The given analysis manager allows the pass to query the pass
    /// manager for the result of specific analysis passes.
    ///
    /// The returned result will be moved into a [`Box`](`std::boxed::Box`)
    /// before being given to the pass manager. This one will then add it to
    /// its internal cache, to avoid unnecessary calls to this entrypoint.
    fn run_analysis<'a>(
        &self,
        module: &Module<'a>,
        manager: &ModuleAnalysisManager,
    ) -> Self::Result;

    #[doc(hidden)]
    fn id() -> AnalysisKey;
}

/// Trait to use for implementing an analysis pass on an LLVM function.
///
/// An analysis pass is not allowed to mutate the LLVM IR.
pub trait LlvmFunctionAnalysis: Default {
    /// Result of the successful execution of this pass by the pass manager.
    ///
    /// This data can be queried by passes through a [`FunctionAnalysisManager`].
    type Result;

    /// Entrypoint for the pass.
    ///
    /// The given analysis manager allows the pass to query the pass
    /// manager for the result of specific analysis passes.
    ///
    /// The returned result will be moved into a [`Box`](`std::boxed::Box`)
    /// before being given to the pass manager. This one will then add it to
    /// its internal cache, to avoid unnecessary calls to this entrypoint.
    fn run_analysis<'a>(
        &self,
        module: &FunctionValue<'a>,
        manager: &FunctionAnalysisManager,
    ) -> Self::Result;

    #[doc(hidden)]
    fn id() -> AnalysisKey;
}

#[doc(hidden)]
#[repr(C)]
pub struct PassPluginLibraryInfo {
    pub api_version: u32,
    pub plugin_name: *const u8,
    pub plugin_version: *const u8,
    pub plugin_registrar: unsafe extern "C" fn(*mut std::ffi::c_void),
}

#[cfg(feature = "macros")]
pub use llvm_plugin_macros::*;

// See https://github.com/jamesmth/llvm-plugin-rs/issues/1
#[cfg(all(target_os = "windows", feature = "llvm10-0"))]
compile_error!("LLVM 10 not supported on Windows");
