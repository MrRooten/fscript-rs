use crate::{backend::{
    base_type::{
        base::FSRValue,
        function::FSRFn,
    },
    vm::{runtime::FSRThreadRuntime, vm::FSRVirtualMachine},
}, utils::error::FSRRuntimeError};

fn io_print<'a>(
    vm: &'a FSRVirtualMachine<'a>,
    rt: &'a mut FSRThreadRuntime<'a>,
) -> Result<u64, FSRRuntimeError<'a>> {
    unimplemented!()
}

fn io_println<'a>(
    vm: &'a FSRVirtualMachine<'a>,
    rt: &'a mut FSRThreadRuntime<'a>,
) -> Result<u64, FSRRuntimeError<'a>> {
    unimplemented!()
}

fn io_dump_obj<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
    unimplemented!()
}

pub fn register_io(vm: &mut FSRVirtualMachine) {
    let fn_obj = FSRFn::from_func(io_print, vm, vec!["value"]);
    vm.register_global_with_name("print", fn_obj.get_id())
        .unwrap();
    let fn_obj = FSRFn::from_func(io_println, vm, vec!["value"]);
    vm.register_global_with_name("println", fn_obj.get_id())
        .unwrap();

    let fn_obj = FSRFn::from_func(io_dump_obj, vm, vec!["value"]);
    vm.register_global_with_name("dump_obj", fn_obj.get_id())
        .unwrap();
}
