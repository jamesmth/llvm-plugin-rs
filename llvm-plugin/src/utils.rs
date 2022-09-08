use inkwell::module::Module;
use inkwell::values::FunctionValue;

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
