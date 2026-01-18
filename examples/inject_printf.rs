// See https://github.com/banach-space/llvm-tutor/blob/main/lib/InjectFuncCall.cpp
// for a more detailed explanation.

use llvm_plugin::inkwell::basic_block::BasicBlock;
use llvm_plugin::inkwell::module::Module;
use llvm_plugin::inkwell::AddressSpace;
use llvm_plugin::{
    LlvmModulePass, ModuleAnalysisManager, PassBuilder, PipelineParsing, PreservedAnalyses,
};

#[cfg(not(any(
    feature = "llvm15-0",
    feature = "llvm16-0",
    feature = "llvm17-0",
    feature = "llvm18-1",
    feature = "llvm19-1",
    feature = "llvm20-1",
    feature = "llvm21-1",
)))]
macro_rules! ptr_type {
    ($cx:ident, $ty:ident) => {
        $cx.$ty().ptr_type(AddressSpace::default())
    };
}
#[cfg(any(
    feature = "llvm15-0",
    feature = "llvm16-0",
    feature = "llvm17-0",
    feature = "llvm18-1",
    feature = "llvm19-1",
    feature = "llvm20-1",
    feature = "llvm21-1",
))]
macro_rules! ptr_type {
    ($cx:ident, $ty:ident) => {
        $cx.ptr_type(AddressSpace::default())
    };
}

#[llvm_plugin::plugin(name = "inject-func-call", version = "0.1")]
fn plugin_registrar(builder: &mut PassBuilder) {
    builder.add_module_pipeline_parsing_callback(|name, pass_manager| {
        if name == "inject-func-call" {
            pass_manager.add_pass(InjectFuncCallPass);
            PipelineParsing::Parsed
        } else {
            PipelineParsing::NotParsed
        }
    });
}

struct InjectFuncCallPass;
impl LlvmModulePass for InjectFuncCallPass {
    fn run_pass(&self, module: &mut Module, _manager: &ModuleAnalysisManager) -> PreservedAnalyses {
        let cx = module.get_context();

        let printf = match module.get_function("printf") {
            Some(func) => func,
            None => {
                // create type `int32 printf(int8*, ...)`
                let arg_ty = ptr_type!(cx, i8_type);
                let func_ty = cx.i32_type().fn_type(&[arg_ty.into()], true);
                module.add_function("printf", func_ty, None)
            }
        };

        // create format string global
        const FORMAT_STR: &[u8] =
            b"(llvm-tutor) Hello from: %s\n(llvm-tutor)   number of arguments: %d\n";

        let format_str_g = module.add_global(
            cx.i8_type().array_type(FORMAT_STR.len() as u32 + 1),
            None,
            "",
        );

        let format_str = cx.const_string(FORMAT_STR, true);
        format_str_g.set_initializer(&format_str);
        format_str_g.set_constant(true);

        let mut inserted_one_printf = false;
        for func in module.get_functions() {
            if func.is_undef() {
                continue;
            }

            let builder = cx.create_builder();

            match func
                .get_first_basic_block()
                .and_then(BasicBlock::get_first_instruction)
            {
                Some(instr) => builder.position_before(&instr),
                None => continue,
            };

            // create printf args
            let func_name = func.get_name().to_str().unwrap();
            let func_name_g = builder.build_global_string_ptr(&func_name, "").unwrap();
            let func_argc = cx.i32_type().const_int(func.count_params() as u64, false);

            eprintln!(" Injecting call to printf inside {}", func_name);

            let format_str_g = builder
                .build_pointer_cast(format_str_g.as_pointer_value(), ptr_type!(cx, i8_type), "")
                .unwrap();

            builder
                .build_call(
                    printf,
                    &[
                        format_str_g.into(),
                        func_name_g.as_pointer_value().into(),
                        func_argc.into(),
                    ],
                    "",
                )
                .unwrap();

            inserted_one_printf = true;
        }

        inserted_one_printf
            .then_some(PreservedAnalyses::None)
            .unwrap_or(PreservedAnalyses::All)
    }
}
