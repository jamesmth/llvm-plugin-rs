use std::ffi::c_void;

type PassEntrypointFn = extern "C" fn(*mut c_void, *mut c_void) -> crate::PreservedAnalyses;
type AnalysisEntrypointFn =
    extern "C" fn(*mut c_void, *mut c_void, *mut *mut c_void, *mut extern "C" fn(*mut c_void));

pub type AnalysisKey = *mut u8;

#[link(name = "llvm-plugin-cpp")]
extern "C" {
    pub fn getFunctionAnalysisResult(
        manager: *mut c_void,
        id: AnalysisKey,
        function: *mut c_void,
    ) -> *mut c_void;

    pub fn getModuleAnalysisResult(
        manager: *mut c_void,
        id: AnalysisKey,
        module: *mut c_void,
    ) -> *mut c_void;

    fn registerModulePass(name: *const u8, name_len: usize, entrypoint: PassEntrypointFn);

    fn registerFunctionPass(name: *const u8, name_len: usize, entrypoint: PassEntrypointFn);

    fn registerModuleAnalysis(id: AnalysisKey, entrypoint: AnalysisEntrypointFn);

    fn registerFunctionAnalysis(id: AnalysisKey, entrypoint: AnalysisEntrypointFn);

    fn llvmPluginApiVersion() -> u32;

    fn llvmPluginRegistrar() -> unsafe extern "C" fn(*mut c_void);
}

pub(super) fn get_module_analysis_result(
    manager: *mut c_void,
    id: AnalysisKey,
    module: *mut c_void,
) -> *mut c_void {
    unsafe { getModuleAnalysisResult(manager, id, module) }
}

pub(super) fn get_function_analysis_result(
    manager: *mut c_void,
    id: AnalysisKey,
    function: *mut c_void,
) -> *mut c_void {
    unsafe { getFunctionAnalysisResult(manager, id, function) }
}

#[doc(hidden)]
pub fn register_module_pass__(name: &str, entrypoint: PassEntrypointFn) {
    unsafe { registerModulePass(name.as_ptr(), name.len(), entrypoint) };
}

#[doc(hidden)]
pub fn register_function_pass__(name: &str, entrypoint: PassEntrypointFn) {
    unsafe { registerFunctionPass(name.as_ptr(), name.len(), entrypoint) };
}

#[doc(hidden)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn register_module_analysis__(id: AnalysisKey, entrypoint: AnalysisEntrypointFn) {
    unsafe { registerModuleAnalysis(id, entrypoint) };
}

#[doc(hidden)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn register_function_analysis__(id: AnalysisKey, entrypoint: AnalysisEntrypointFn) {
    unsafe { registerFunctionAnalysis(id, entrypoint) };
}

#[doc(hidden)]
pub fn get_llvm_plugin_api_version__() -> u32 {
    unsafe { llvmPluginApiVersion() }
}

#[doc(hidden)]
pub fn get_llvm_plugin_registrar__() -> unsafe extern "C" fn(*mut c_void) {
    unsafe { llvmPluginRegistrar() }
}
