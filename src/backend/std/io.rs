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
    let s = rt.find_symbol("value", vm, None)?;
    let obj = vm.get_obj_by_id(&s).unwrap();
    let str_object = obj.invoke_method("to_string", vm, rt)?;
    let obj = vm.get_obj_by_id(&str_object).unwrap();
    if let FSRValue::String(s) = obj.get_value() {
        print!("{}", s.get_string());
    }
    return Ok(vm.get_none_id());
}

fn io_println<'a>(
    vm: &'a FSRVirtualMachine<'a>,
    rt: &'a mut FSRThreadRuntime<'a>,
) -> Result<u64, FSRRuntimeError<'a>> {
    let s = rt.find_symbol("value", vm, None).unwrap();
    let obj = vm.get_obj_by_id(&s).unwrap();
    let str_object = obj.invoke_method("to_string", vm, rt)?;
    let obj = vm.get_obj_by_id(&str_object).unwrap();
    if let FSRValue::String(s) = obj.get_value() {
        println!("{}", s.get_string());
    }
    return Ok(vm.get_none_id());
}

fn io_dump_obj<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
    let s = rt.find_symbol("value", vm, None).unwrap();
    let obj = vm.get_obj_by_id(&s).unwrap();
    println!("{:#?}", obj);
    return Ok(vm.get_none_id());
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
