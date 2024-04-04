use crate::{backend::{base_type::{function::FSRFn, module::FSRModule}, vm::{runtime::FSRThreadRuntime, vm::FSRVirtualMachine}}, utils::error::FSRRuntimeError};


fn test<'a>(
    vm: &'a FSRVirtualMachine<'a>,
    rt: &'a mut FSRThreadRuntime<'a>,
) -> Result<u64, FSRRuntimeError<'a>> {
    println!("slkfjskldfjlskdfj");
    return Ok(vm.get_none_id());
}

pub fn register_path<'a>(vm: &'a FSRVirtualMachine<'a>) -> FSRModule {
    let mut module = FSRModule::new("path");
    let fn_obj = FSRFn::from_func(test, vm, vec!["p"]);
    module.register_obj("test", fn_obj.get_id());
    return module;
}