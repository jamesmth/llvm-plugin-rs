use std::ffi::c_void;

use inkwell::module::Module;
use inkwell::values::{AsValueRef, FunctionValue};
use inkwell::LLVMReference;

/// Struct allowing to query the pass manager for the result of
/// analyses on function IR.
pub struct FunctionAnalysisManager {
    inner: *mut c_void,
    from_analysis_id: Option<crate::AnalysisKey>,
}

impl FunctionAnalysisManager {
    #[doc(hidden)]
    pub unsafe fn from_raw(
        inner: *mut c_void,
        from_analysis_id: Option<crate::AnalysisKey>,
    ) -> Self {
        Self {
            inner,
            from_analysis_id,
        }
    }

    /// Returns the result of the analysis on a given function IR.
    ///
    /// If the result is not in cache, the pass manager will execute the
    /// analysis pass. Otherwise, the result is directly returned from cache.
    ///
    /// # Panics
    ///
    /// Panics if the given analysis wasn't registered (happens if the `#[analysis]`
    /// attribute wasn't used), or if this function was called within the given analysis
    /// itself.
    pub fn get_result<'a, A>(&self, function: &FunctionValue<'a>) -> &A::Result
    where
        A: crate::LlvmFunctionAnalysis,
    {
        let id = A::id();
        assert!(
            !matches!(self.from_analysis_id, Some(n) if id == n),
            "Analysis cannot request its own result"
        );

        unsafe {
            let res =
                crate::get_function_analysis_result(self.inner, id, function.as_value_ref().cast());
            Box::leak(Box::from_raw(res.cast()))
        }
    }
}

/// Struct allowing to query the pass manager for the result of
/// analyses on module IR.
pub struct ModuleAnalysisManager {
    inner: *mut c_void,
    from_analysis_id: Option<crate::AnalysisKey>,
}

impl ModuleAnalysisManager {
    #[doc(hidden)]
    pub unsafe fn from_raw(
        inner: *mut c_void,
        from_analysis_id: Option<crate::AnalysisKey>,
    ) -> Self {
        Self {
            inner,
            from_analysis_id,
        }
    }

    /// Returns the result of the analysis on a given module IR.
    ///
    /// If the result is not in cache, the pass manager will execute the
    /// analysis pass. Otherwise, the result is directly returned from cache.
    ///
    /// # Panics
    ///
    /// Panics if the given analysis wasn't registered (happens if the `#[analysis]`
    /// attribute wasn't used), or if this function was called within the given analysis
    /// itself.
    pub fn get_result<'a, A>(&self, module: &Module<'a>) -> &'static A::Result
    where
        A: crate::LlvmModuleAnalysis,
    {
        let id = A::id();
        assert!(
            !matches!(self.from_analysis_id, Some(n) if id == n),
            "Analysis cannot request its own result"
        );

        unsafe {
            let res =
                crate::get_module_analysis_result(self.inner, A::id(), module.get_ref().cast());
            Box::leak(Box::from_raw(res.cast()))
        }
    }
}
