use std::ffi::c_void;

use super::{
    FunctionAnalysisManager, FunctionPassManager, ModuleAnalysisManager, ModulePassManager,
};

/// Main struct for registering callbacks.
pub struct PassBuilder {
    inner: *mut c_void,
}

impl PassBuilder {
    #[doc(hidden)]
    pub unsafe fn from_raw(pass_builder: *mut c_void) -> Self {
        Self {
            inner: pass_builder,
        }
    }

    /// Register a new pipeline parsing callback.
    ///
    /// These callbacks can be used to parse a single pass name, and populate
    /// the given [ModulePassManager] accordingly.
    pub fn add_module_pipeline_parsing_callback<T>(&mut self, cb: T)
    where
        T: Fn(&str, &mut ModulePassManager) -> PipelineParsing,
    {
        let cb = Box::new(cb);

        extern "C" fn callback_deleter<T>(cb: *const c_void) {
            drop(unsafe { Box::<T>::from_raw(cb as *mut _) })
        }

        extern "C" fn callback_entrypoint<T>(
            cb: *const c_void,
            name_ptr: *const u8,
            name_len: usize,
            manager: *mut c_void,
        ) -> bool
        where
            T: Fn(&str, &mut ModulePassManager) -> PipelineParsing,
        {
            let cb = unsafe { Box::<T>::from_raw(cb as *mut _) };
            let name = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
            let name = unsafe { std::str::from_utf8_unchecked(name) };
            let mut manager = unsafe { ModulePassManager::from_raw(manager) };

            let res = cb(name, &mut manager);

            Box::into_raw(cb);
            matches!(res, PipelineParsing::Parsed)
        }

        unsafe {
            super::passBuilderAddModulePipelineParsingCallback(
                self.inner,
                Box::into_raw(cb).cast(),
                callback_deleter::<T>,
                callback_entrypoint::<T>,
            )
        }
    }

    /// Register a new pipeline parsing callback.
    ///
    /// These callbacks can be used to parse a single pass name, and populate
    /// the given [FunctionPassManager] accordingly.
    pub fn add_function_pipeline_parsing_callback<T>(&mut self, cb: T)
    where
        T: Fn(&str, &mut FunctionPassManager) -> PipelineParsing,
    {
        let cb = Box::new(cb);

        extern "C" fn callback_deleter<T>(cb: *const c_void) {
            drop(unsafe { Box::<T>::from_raw(cb as *mut _) })
        }

        extern "C" fn callback_entrypoint<T>(
            cb: *const c_void,
            name_ptr: *const u8,
            name_len: usize,
            manager: *mut c_void,
        ) -> bool
        where
            T: Fn(&str, &mut FunctionPassManager) -> PipelineParsing,
        {
            let cb = unsafe { Box::<T>::from_raw(cb as *mut _) };
            let name = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
            let name = unsafe { std::str::from_utf8_unchecked(name) };
            let mut manager = unsafe { FunctionPassManager::from_raw(manager) };

            let res = cb(name, &mut manager);

            Box::into_raw(cb);
            matches!(res, PipelineParsing::Parsed)
        }

        unsafe {
            super::passBuilderAddFunctionPipelineParsingCallback(
                self.inner,
                Box::into_raw(cb).cast(),
                callback_deleter::<T>,
                callback_entrypoint::<T>,
            )
        }
    }

    /// Register a new callback for analysis registration.
    ///
    /// These callbacks can be used to register custom analyses with the given
    /// [ModuleAnalysisManager].
    pub fn add_module_analysis_registration_callback<T>(&mut self, cb: T)
    where
        T: Fn(&mut ModuleAnalysisManager),
    {
        let cb = Box::new(cb);

        extern "C" fn callback_deleter<T>(cb: *const c_void) {
            drop(unsafe { Box::<T>::from_raw(cb as *mut _) })
        }

        extern "C" fn callback_entrypoint<T>(cb: *const c_void, manager: *mut c_void)
        where
            T: Fn(&mut ModuleAnalysisManager),
        {
            let cb = unsafe { Box::<T>::from_raw(cb as *mut _) };
            let mut manager = unsafe { ModuleAnalysisManager::from_raw(manager, None) };

            cb(&mut manager);

            Box::into_raw(cb);
        }

        unsafe {
            super::passBuilderAddModuleAnalysisRegistrationCallback(
                self.inner,
                Box::into_raw(cb).cast(),
                callback_deleter::<T>,
                callback_entrypoint::<T>,
            )
        }
    }

    /// Register a new callback for analysis registration.
    ///
    /// These callbacks can be used to register custom analyses with the given
    /// [FunctionAnalysisManager].
    pub fn add_function_analysis_registration_callback<T>(&mut self, cb: T)
    where
        T: Fn(&mut FunctionAnalysisManager),
    {
        let cb = Box::new(cb);

        extern "C" fn callback_deleter<T>(cb: *const c_void) {
            drop(unsafe { Box::<T>::from_raw(cb as *mut _) })
        }

        extern "C" fn callback_entrypoint<T>(cb: *const c_void, manager: *mut c_void)
        where
            T: Fn(&mut FunctionAnalysisManager),
        {
            let cb = unsafe { Box::<T>::from_raw(cb as *mut _) };
            let mut manager = unsafe { FunctionAnalysisManager::from_raw(manager, None) };

            cb(&mut manager);

            Box::into_raw(cb);
        }

        unsafe {
            super::passBuilderAddFunctionAnalysisRegistrationCallback(
                self.inner,
                Box::into_raw(cb).cast(),
                callback_deleter::<T>,
                callback_entrypoint::<T>,
            )
        }
    }
}

/// Enum describing whether a pipeline parsing callback
/// successfully parsed its given pipeline element.
#[derive(Clone, Copy)]
pub enum PipelineParsing {
    /// The pipeline element was successfully parsed.
    Parsed,

    /// The pipeline element wasn't parsed.
    NotParsed,
}