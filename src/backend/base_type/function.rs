

use crate::{backend::{base_type::base::FSRValue, vm::{runtime::FSRThreadRuntime, vm::FSRVirtualMachine}}, frontend::ast::token::{base::FSRToken, function_def::FSRFnDef}, utils::error::FSRRuntimeError};

use super::base::FSRObject;


type FSRFuncType = for<'a> fn(manager: &'a FSRVirtualMachine<'a>, rt: &'a mut FSRThreadRuntime<'a>) -> Result<u64, FSRRuntimeError<'a>>;


enum FSRFnValue<'a> {
    RustImpl(FSRFuncType),
    FSRImpl(&'a FSRFnDef<'a>)
}

pub struct FSRFn<'a> {
    args        : Vec<&'a str>,
    value       : FSRFnValue<'a>,
    identify    : u32
}

impl std::fmt::Debug for FSRFn<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FSRFunction").field("value", &self.identify).finish()
    }
}

impl<'a> FSRFn<'a> {

    pub fn from_func(func: FSRFuncType, vm: &'a FSRVirtualMachine<'a>, args: Vec<&'a str>) -> &'a FSRObject<'a> {
        let v = Self {
            value: FSRFnValue::RustImpl(func),
            args: args,
            identify: 0, 
        };
        let obj = FSRObject::new(vm);
        obj.set_value(FSRValue::Function(v));
        return obj;
    }

    pub fn from_ast(fn_def: &'a FSRFnDef<'a>, vm: &'a FSRVirtualMachine<'a>) -> &'a FSRObject<'a> {
        let args = fn_def.get_args();
        let mut fn_args = vec![];
        for arg in args {
            if let FSRToken::Variable(v) = arg {
                fn_args.push(v.get_name());
            } else if let FSRToken::Assign(a) = arg {
                fn_args.push(a.get_name());
            } else {
                unimplemented!()
            }
        }
        let v = Self {
            value: FSRFnValue::FSRImpl(fn_def),
            args: fn_args,
            identify: 0
        };
        let obj = FSRObject::new(vm);
        obj.set_value(FSRValue::Function(v));

        return obj;
    }


    pub fn invoke(&self, vm: &'a FSRVirtualMachine<'a>, 
        module: &'a mut FSRThreadRuntime<'a>) -> Result<u64, FSRRuntimeError> {
        if let FSRFnValue::RustImpl(v) = &self.value {
            let v = (*v)(vm, module);
            return v;
        }

        if let FSRFnValue::FSRImpl(v) = &self.value {
            return Ok(module.run_ast_fn(v, vm).unwrap());
        }

        unimplemented!()
    }

    pub fn get_args(&self) -> &Vec<&'a str> {
        return &self.args;
    }
}

pub struct FSRMethod {

}