// See https://github.com/banach-space/llvm-tutor/blob/main/lib/StaticCallCounter.cpp
// for a more detailed explanation.

#[llvm_plugin::plugin(name = "static-cc", version = "0.1")]
mod plugin {
    use std::collections::HashMap;

    use either::Either;

    use llvm_plugin::inkwell::module::Module;
    use llvm_plugin::inkwell::values::{BasicValueEnum, InstructionOpcode};
    use llvm_plugin::utils::InstructionIterator;
    use llvm_plugin::{
        LlvmModuleAnalysis, LlvmModulePass, ModuleAnalysisManager, PreservedAnalyses,
    };

    #[derive(Default)]
    struct StaticCallCounterPrinterPass;

    #[pass(name = "static-cc-printer")]
    impl LlvmModulePass for StaticCallCounterPrinterPass {
        fn run_pass(
            &self,
            module: &mut Module,
            manager: &ModuleAnalysisManager,
        ) -> PreservedAnalyses {
            let call_map = manager.get_result::<StaticCallCounterAnalysis>(module);
            print_static_counter_result(call_map);
            PreservedAnalyses::All
        }
    }

    #[derive(Default)]
    struct StaticCallCounterAnalysis;

    #[analysis]
    impl LlvmModuleAnalysis for StaticCallCounterAnalysis {
        fn run_analysis(
            &self,
            module: &Module,
            _manager: &ModuleAnalysisManager,
        ) -> HashMap<String, usize> {
            let mut call_map = HashMap::new();

            for func in module.get_functions() {
                for bb in func.get_basic_blocks() {
                    for instr in InstructionIterator::new(&bb) {
                        if !matches!(instr.get_opcode(), InstructionOpcode::Call) {
                            continue;
                        }

                        let ptr = match instr.get_operand(1) {
                            Some(Either::Left(BasicValueEnum::PointerValue(ptr))) => ptr,
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
}
