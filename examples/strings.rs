// See https://github.com/tsarpaul/llvm-string-obfuscator
// for a more detailed explanation.

use llvm_plugin::inkwell::basic_block::BasicBlock;
use llvm_plugin::inkwell::module::{Linkage, Module};
use llvm_plugin::inkwell::values::{BasicValueEnum, FunctionValue, GlobalValue};
use llvm_plugin::inkwell::{AddressSpace, IntPredicate};
use llvm_plugin::{
    LlvmModulePass, ModuleAnalysisManager, PassBuilder, PipelineParsing, PreservedAnalyses,
};

#[llvm_plugin::plugin(name = "StringObfuscatorPass", version = "v0.1")]
fn plugin_registrar(builder: &mut PassBuilder) {
    builder.add_module_pipeline_parsing_callback(|name, pass_manager| {
        if name == "string-obfuscator-pass" {
            pass_manager.add_pass(StringObfuscatorModPass);
            PipelineParsing::Parsed
        } else {
            PipelineParsing::NotParsed
        }
    });
}

struct StringObfuscatorModPass;
impl LlvmModulePass for StringObfuscatorModPass {
    fn run_pass(&self, module: &mut Module, _manager: &ModuleAnalysisManager) -> PreservedAnalyses {
        // transform the strings
        let global_strings = encode_global_strings(module);

        // inject functions
        let decode_fn = create_decode_fn(module);
        let decode_stub = create_decode_stub(module, global_strings, decode_fn);

        // inject a call to decode_stub from main
        if let Some(instr) = module
            .get_function("main")
            .and_then(FunctionValue::get_first_basic_block)
            .and_then(BasicBlock::get_first_instruction)
        {
            let cx = module.get_context();
            let builder = cx.create_builder();
            builder.position_before(&instr);
            builder.build_call(decode_stub, &[], "").unwrap();
        };

        PreservedAnalyses::None
    }
}

enum GlobalString<'a> {
    Array(GlobalValue<'a>, u32),
    Struct(GlobalValue<'a>, u32, u32),
}

fn encode_global_strings<'a>(module: &mut Module<'a>) -> Vec<GlobalString<'a>> {
    let mut global_strings = Vec::new();
    let cx = module.get_context();

    for global in module.get_globals() {
        // ignore external globals
        if matches!(global.get_linkage(), Linkage::External) {
            continue;
        }

        match global.get_initializer() {
            // C-like strings
            Some(BasicValueEnum::ArrayValue(arr)) if arr.is_const_string() => {
                let encoded_str = match arr.get_string_constant() {
                    Some(s) => s
                        .to_bytes_with_nul()
                        .iter()
                        .map(|c| *c + 1)
                        .collect::<Vec<_>>(),
                    None => continue,
                };
                let new_const = cx.const_string(&encoded_str, false);
                global.set_initializer(&new_const);
                global.set_constant(false);
                global_strings.push(GlobalString::Array(global, encoded_str.len() as u32));
            }
            // Rust-like strings
            Some(BasicValueEnum::StructValue(stru)) if stru.count_fields() <= 1 => {
                let arr = match stru.get_field_at_index(0) {
                    Some(BasicValueEnum::ArrayValue(arr)) if arr.is_const_string() => arr,
                    _ => continue,
                };
                let encoded_str = match arr.get_string_constant() {
                    Some(s) => s
                        .to_bytes_with_nul()
                        .iter()
                        .map(|c| *c + 1)
                        .collect::<Vec<_>>(),
                    None => continue,
                };
                let new_const = cx.const_string(&encoded_str, false);
                stru.set_field_at_index(0, new_const);
                global.set_constant(false);
                global_strings.push(GlobalString::Struct(global, 0, encoded_str.len() as u32));
            }
            _ => (),
        }
    }

    global_strings
}

fn create_decode_fn<'a>(module: &mut Module<'a>) -> FunctionValue<'a> {
    let cx = module.get_context();

    // create type `void decode(int8*, int32)`
    let arg1_ty = cx.i8_type().ptr_type(AddressSpace::default());
    let arg2_ty = cx.i32_type();
    let fn_ty = cx
        .void_type()
        .fn_type(&[arg1_ty.into(), arg2_ty.into()], false);

    let decode_fn = module.add_function("decode", fn_ty, None);

    let start_bb = cx.append_basic_block(decode_fn, "");
    let builder = cx.create_builder();
    builder.position_at_end(start_bb);

    let arg1 = decode_fn.get_nth_param(0).unwrap();
    let arg2 = decode_fn.get_nth_param(1).unwrap();

    let var3 = builder
        .build_is_not_null(arg1.into_pointer_value(), "")
        .unwrap();
    let var4 = builder
        .build_int_compare(
            IntPredicate::SGT,
            arg2.into_int_value(),
            cx.i32_type().const_zero(),
            "",
        )
        .unwrap();
    let var5 = builder.build_and(var4, var3, "").unwrap();

    let loop_body_bb = cx.append_basic_block(decode_fn, "");
    let end_bb = cx.append_basic_block(decode_fn, "");
    builder
        .build_conditional_branch(var5, loop_body_bb, end_bb)
        .unwrap();

    builder.position_at_end(loop_body_bb);
    let phi1 = builder
        .build_phi(cx.i8_type().ptr_type(AddressSpace::default()), "")
        .unwrap();
    let phi2 = builder.build_phi(cx.i32_type(), "").unwrap();
    let var9 = builder
        .build_int_nsw_add(
            phi2.as_basic_value().into_int_value(),
            cx.i32_type().const_all_ones(),
            "",
        )
        .unwrap();
    #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0")))]
    let var10 = unsafe {
        builder.build_gep(
            phi1.as_basic_value().into_pointer_value(),
            &[cx.i64_type().const_int(1, false)],
            "",
        )
    }
    .unwrap();
    #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0"))]
    let var10 = unsafe {
        builder.build_gep(
            cx.i8_type(),
            phi1.as_basic_value().into_pointer_value(),
            &[cx.i64_type().const_int(1, false)],
            "",
        )
    }
    .unwrap();
    #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0")))]
    let var11 = builder.build_load(phi1.as_basic_value().into_pointer_value(), "");
    #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0"))]
    let var11 = builder
        .build_load(cx.i8_type(), phi1.as_basic_value().into_pointer_value(), "")
        .unwrap();
    let var12 = builder
        .build_int_add(var11.into_int_value(), cx.i8_type().const_all_ones(), "")
        .unwrap();
    builder
        .build_store(phi1.as_basic_value().into_pointer_value(), var12)
        .unwrap();
    let var13 = builder
        .build_int_compare(
            IntPredicate::SGT,
            phi2.as_basic_value().into_int_value(),
            cx.i32_type().const_int(1, false),
            "",
        )
        .unwrap();

    builder
        .build_conditional_branch(var13, loop_body_bb, end_bb)
        .unwrap();

    builder.position_at_end(end_bb);
    builder.build_return(None).unwrap();
    phi1.add_incoming(&[(&var10, loop_body_bb), (&arg1, start_bb)]);
    phi2.add_incoming(&[(&var9, loop_body_bb), (&arg2, start_bb)]);

    decode_fn
}

fn create_decode_stub<'a>(
    module: &mut Module<'a>,
    global_strings: Vec<GlobalString<'a>>,
    decode_fn: FunctionValue<'a>,
) -> FunctionValue<'a> {
    let cx = module.get_context();

    let decode_stub = module.add_function("decode_stub", cx.void_type().fn_type(&[], false), None);

    let start_bb = cx.append_basic_block(decode_stub, "");
    let builder = cx.create_builder();
    builder.position_at_end(start_bb);

    for globstr in global_strings {
        let (s, len) = match globstr {
            GlobalString::Array(gs, len) => {
                let s = builder
                    .build_pointer_cast(
                        gs.as_pointer_value(),
                        cx.i8_type().ptr_type(AddressSpace::default()),
                        "",
                    )
                    .unwrap();
                (s, len)
            }
            GlobalString::Struct(gs, id, len) => {
                #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0")))]
                let s = builder
                    .build_struct_gep(gs.as_pointer_value(), id, "")
                    .unwrap();
                #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0"))]
                let s = {
                    let i8_ty_ptr = cx.i8_type().ptr_type(AddressSpace::default());
                    let struct_ty = cx.struct_type(&[i8_ty_ptr.into()], false);
                    builder
                        .build_struct_gep(struct_ty, gs.as_pointer_value(), id, "")
                        .unwrap()
                };
                let s = builder
                    .build_pointer_cast(s, cx.i8_type().ptr_type(AddressSpace::default()), "")
                    .unwrap();
                (s, len)
            }
        };
        let len = cx.i32_type().const_int(len as u64, false);
        builder
            .build_call(decode_fn, &[s.into(), len.into()], "")
            .unwrap();
    }

    builder.build_return(None).unwrap();
    decode_stub
}
