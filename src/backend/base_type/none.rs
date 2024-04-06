use crate::backend::base_type::base::{FSRBaseType, FSRObject, FSRValue, IFSRObject};
use crate::backend::vm::vm::FSRVirtualMachine;

#[derive(Debug)]
pub struct FSRNone {

}

impl FSRNone {
    pub fn new<'a>(vm: &'a FSRVirtualMachine<'a>) -> &'a FSRObject<'a> {
        let obj = FSRObject::new_with_id(vm, 0);
        obj.set_cls(vm.get_cls("none").unwrap());
        obj.set_value(FSRValue::None);
        return obj;
    }
}

impl IFSRObject for FSRNone {

    fn get_class_name() -> &'static str {
        "none"
    }

    fn get_class(_: &FSRVirtualMachine) -> FSRBaseType {
        let cls = FSRBaseType::new("none");
        return cls;
    }
    
    fn init(&mut self) {
        todo!()
    }
}