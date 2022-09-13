// See https://github.com/banach-space/llvm-tutor/blob/main/lib/OpcodeCounter.cpp
// for a more detailed explanation.

#[llvm_plugin::plugin(name = "OpcodeCounter", version = "0.1")]
mod plugin {
    use std::collections::HashMap;

    use llvm_plugin::inkwell::values::{FunctionValue, InstructionOpcode};
    use llvm_plugin::utils::InstructionIterator;
    use llvm_plugin::{
        FunctionAnalysisManager, LlvmFunctionAnalysis, LlvmFunctionPass, PreservedAnalyses,
    };

    #[derive(Default)]
    struct OpcodeCounterPrinterPass;

    #[pass(name = "opcode-counter-printer")]
    impl LlvmFunctionPass for OpcodeCounterPrinterPass {
        fn run_pass(
            &self,
            function: &mut FunctionValue,
            manager: &FunctionAnalysisManager,
        ) -> PreservedAnalyses {
            let opcode_map = manager.get_result::<OpcodeCounterAnalysis>(function);

            println!(
                "Printing analysis 'OpcodeCounter Pass` for function {:?}:",
                function.get_name()
            );

            print_opcode_counter_result(opcode_map);
            PreservedAnalyses::All
        }
    }

    #[derive(Default)]
    struct OpcodeCounterAnalysis;

    #[analysis]
    impl LlvmFunctionAnalysis for OpcodeCounterAnalysis {
        fn run_analysis(
            &self,
            function: &FunctionValue,
            _manager: &FunctionAnalysisManager,
        ) -> HashMap<InstructionOpcode, usize> {
            let mut opcode_map = HashMap::new();

            for bb in function.get_basic_blocks() {
                for instr in InstructionIterator::new(&bb) {
                    opcode_map
                        .entry(instr.get_opcode())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }
            }

            opcode_map
        }
    }

    fn print_opcode_counter_result(opcode_map: &HashMap<InstructionOpcode, usize>) {
        println!("=================================================");
        println!("LLVM-TUTOR: OpcodeCounter results");
        println!("=================================================");
        println!("{:<20} {:<10}", "OPCODE", "#TIMES USED");
        println!("----------------------------------------------------");

        for (opcode, count) in opcode_map {
            let name = format!("{:?}", opcode);
            println!("{:<20} {:<10}", name, count);
        }

        println!("----------------------------------------------------\n");
    }
}
