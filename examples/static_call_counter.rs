// See https://github.com/banach-space/llvm-tutor/blob/main/lib/StaticCallCounter.cpp
// for a more detailed explanation.

use std::collections::HashMap;

use inkwell::values::Operand;
use llvm_plugin::inkwell::module::Module;
use llvm_plugin::inkwell::values::{BasicValueEnum, InstructionOpcode};
use llvm_plugin::{
    AnalysisKey, LlvmModuleAnalysis, LlvmModulePass, ModuleAnalysisManager, PassBuilder,
    PipelineParsing, PreservedAnalyses,
};

#[llvm_plugin::plugin(name = "static-cc", version = "0.1")]
fn plugin_registrar(builder: &mut PassBuilder) {
    builder.add_module_pipeline_parsing_callback(|name, pass_manager| {
        if name == "static-cc-printer" {
            pass_manager.add_pass(StaticCallCounterPrinterPass);
            PipelineParsing::Parsed
        } else {
            PipelineParsing::NotParsed
        }
    });

    builder.add_module_analysis_registration_callback(|manager| {
        manager.register_pass(StaticCallCounterAnalysis);
    });
}

struct StaticCallCounterPrinterPass;
impl LlvmModulePass for StaticCallCounterPrinterPass {
    fn run_pass(&self, module: &mut Module, manager: &ModuleAnalysisManager) -> PreservedAnalyses {
        let call_map = manager.get_result::<StaticCallCounterAnalysis>(module);
        print_static_counter_result(call_map);
        PreservedAnalyses::All
    }
}

struct StaticCallCounterAnalysis;
impl LlvmModuleAnalysis for StaticCallCounterAnalysis {
    type Result = HashMap<String, usize>;

    fn run_analysis(&self, module: &Module, _manager: &ModuleAnalysisManager) -> Self::Result {
        let mut call_map = HashMap::new();

        for func in module.get_functions() {
            for bb in func.get_basic_blocks() {
                for instr in bb.get_instructions() {
                    if !matches!(instr.get_opcode(), InstructionOpcode::Call) {
                        continue;
                    }

                    let ptr = match instr.get_operand(1) {
                        Some(Operand::Value(BasicValueEnum::PointerValue(ptr))) => ptr,
                        _ => continue,
                    };

                    let name = ptr.get_name().to_bytes();
                    if !name.is_empty() {
                        call_map
                            .entry(String::from_utf8_lossy(name).into_owned())
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                    }
                }
            }
        }

        call_map
    }

    fn id() -> AnalysisKey {
        1 as AnalysisKey
    }
}

fn print_static_counter_result(call_map: &HashMap<String, usize>) {
    println!("=================================================");
    println!("LLVM-TUTOR: static analysis results");
    println!("=================================================");
    println!("{:<20} {:<10}", "NAME", "#N DIRECT CALLS");
    println!("----------------------------------------------------");

    for (name, count) in call_map {
        println!("{:<20} {:<10}", name, count);
    }

    println!("----------------------------------------------------\n");
}
