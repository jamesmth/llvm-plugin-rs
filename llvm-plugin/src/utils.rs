use inkwell::basic_block::BasicBlock;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, GlobalValue, InstructionValue};

/// Convenient iterator for enumerating functions.
pub struct FunctionIterator<'a, 'ctx> {
    module: &'a Module<'ctx>,
    func: Option<FunctionValue<'ctx>>,
}

impl<'a, 'ctx> FunctionIterator<'a, 'ctx> {
    /// Create a new [`FunctionIterator`] from a [`Module`].
    pub fn new(module: &'a Module<'ctx>) -> Self {
        Self { module, func: None }
    }
}

impl<'a, 'ctx> Iterator for FunctionIterator<'a, 'ctx> {
    type Item = FunctionValue<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(func) = self.func {
            self.func = func.get_next_function();
            return self.func;
        }

        self.func = Some(self.module.get_first_function()?);
        self.func
    }
}

/// Convenient iterator for enumerating globals.
pub struct GlobalIterator<'a, 'ctx> {
    module: &'a Module<'ctx>,
    global: Option<GlobalValue<'ctx>>,
}

impl<'a, 'ctx> GlobalIterator<'a, 'ctx> {
    /// Create a new [`GlobalIterator`] from a [`Module`].
    pub fn new(module: &'a Module<'ctx>) -> Self {
        Self {
            module,
            global: None,
        }
    }
}

impl<'a, 'ctx> Iterator for GlobalIterator<'a, 'ctx> {
    type Item = GlobalValue<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(global) = self.global {
            self.global = global.get_next_global();
            return self.global;
        }

        self.global = Some(self.module.get_first_global()?);
        self.global
    }
}

/// Convenient iterator for enumerating globals.
pub struct InstructionIterator<'a, 'ctx> {
    bb: &'a BasicBlock<'ctx>,
    instr: Option<InstructionValue<'ctx>>,
}

impl<'a, 'ctx> InstructionIterator<'a, 'ctx> {
    /// Create a new [`InstructionIterator`] from a [`BasicBlock`].
    pub fn new(bb: &'a BasicBlock<'ctx>) -> Self {
        Self { bb, instr: None }
    }
}

impl<'a, 'ctx> Iterator for InstructionIterator<'a, 'ctx> {
    type Item = InstructionValue<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(instr) = self.instr {
            self.instr = instr.get_next_instruction();
            return self.instr;
        }

        self.instr = Some(self.bb.get_first_instruction()?);
        self.instr
    }
}
