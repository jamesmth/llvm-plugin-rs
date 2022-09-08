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

    /// Returns the result of the analysis on a given function IR.
    ///
    /// If the result is not in cache, `None` is returned. Otherwise,
    /// the result is directly returned from cache.
    ///
    /// This function never triggers the execution of an analysis.
    ///
    /// # Panics
    ///
    /// Panics if the given analysis wasn't registered (happens if the `#[analysis]`
    /// attribute wasn't used), or if this function was called within the given analysis
    /// itself.
    pub fn get_cached_result<'a, A>(&self, function: &FunctionValue<'a>) -> Option<&A::Result>
    where
        A: crate::LlvmFunctionAnalysis,
    {
        let id = A::id();
        assert!(
            !matches!(self.from_analysis_id, Some(n) if id == n),
            "Analysis cannot request its own result"
        );

        unsafe {
            let res = crate::get_function_analysis_cached_result(
                self.inner,
                id,
                function.as_value_ref().cast(),
            );
            (!res.is_null()).then_some(Box::leak(Box::from_raw(res.cast())))
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
    pub fn get_result<'a, A>(&self, module: &Module<'a>) -> &A::Result
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

    /// Returns the result of the analysis on a given module IR.
    ///
    /// If the result is not in cache, `None` is returned. Otherwise,
    /// the result is directly returned from cache.
    ///
    /// This function never triggers the execution of an analysis.
    ///
    /// # Panics
    ///
    /// Panics if the given analysis wasn't registered (happens if the `#[analysis]`
    /// attribute wasn't used), or if this function was called within the given analysis
    /// itself.
    pub fn get_cached_result<'a, A>(&self, module: &Module<'a>) -> Option<&A::Result>
    where
        A: crate::LlvmModuleAnalysis,
    {
        let id = A::id();
        assert!(
            !matches!(self.from_analysis_id, Some(n) if id == n),
            "Analysis cannot request its own result"
        );

        unsafe {
            let res = crate::get_module_analysis_cached_result(
                self.inner,
                A::id(),
                module.get_ref().cast(),
            );
            (!res.is_null()).then_some(Box::leak(Box::from_raw(res.cast())))
        }
    }

    /// Returns a [`FunctionAnalysisManagerProxy`], which is essentially an interface
    /// allowing management of analyses at the function level.
    pub fn get_function_analysis_manager_proxy<'a>(
        &self,
        module: &Module<'a>,
    ) -> FunctionAnalysisManagerProxy {
        let proxy = crate::get_function_analysis_manager_module_proxy(self.inner, unsafe {
            module.get_ref().cast()
        });
        FunctionAnalysisManagerProxy { inner: proxy }
    }
}

/// Struct allowing to make queries to the pass manager about function-level
/// analyses.
///
/// The main use-case of such interface is to give the ability for module-level
/// passes to trigger/query function-level analyses.
pub struct FunctionAnalysisManagerProxy {
    inner: *mut c_void,
}

impl FunctionAnalysisManagerProxy {
    /// Returns the inner [`FunctionAnalysisManager`].
    pub fn get_manager(&self) -> FunctionAnalysisManager {
        let manager = crate::get_function_analysis_manager(self.inner);
        FunctionAnalysisManager {
            inner: manager,
            from_analysis_id: None,
        }
    }
}
