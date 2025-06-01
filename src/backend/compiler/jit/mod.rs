pub mod cranelift;
pub mod jit_wrapper;
use crate::backend::types::code::FSRCode;

use super::bytecode::{Bytecode, BytecodeArg, BytecodeOperator};

pub struct FSRJitCompiler {

}

impl FSRJitCompiler {
    pub fn new_jit() -> Self {
        Self {}
    }

    fn compile_bytecode_arg(&self, arg: &BytecodeArg) -> Result<(), String> {
        
        Ok(())
    }

    fn iter_variable(&self, code_list: &Bytecode) -> Result<(), String> {
        let vars = &code_list.var_map;
        Ok(())
    }

    pub fn compile(&mut self, code: &FSRCode) -> Result<(), String> {
        // Here you would implement the JIT compilation logic
        // For now, we just return Ok to indicate success

        let code_list = &code.get_bytecode().bytecode;

        for expr in code_list {
            
        }

        Ok(())
    }
}