use crate::{backend::{base_type::base::FSRValue, vm::{runtime::FSRThreadRuntime, vm::FSRVirtualMachine}}, utils::error::FSRRuntimeError};

fn export_obj<'a>(
    vm: &'a FSRVirtualMachine<'a>,
    rt: &'a mut FSRThreadRuntime<'a>,
) -> Result<u64, FSRRuntimeError<'a>> {
    
    return Ok(0);
}