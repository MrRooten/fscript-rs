use crate::backend::base_type::bool::FSRBool;
use std::collections::HashMap;
use std::fmt::Error;

use crate::backend::vm::module::FSRRuntimeModule;
use crate::backend::vm::vm::FSRVirtualMachine;
use crate::utils::error::{FSRRuntimeError, FSRRuntimeType};

use super::function::FSRFn;
use super::integer::FSRInteger;
use super::list::FSRList;
use super::none::FSRNone;
use super::string::FSRString;
use super::utils::i_to_m;

#[derive(Debug)]
pub enum FSRObjectType {
    Function,
    Class,
    Integer,
    String,
    Bytes,
    Object,
    MetaClass,
    None,
}

#[derive(Debug)]
pub enum FSRValue<'a> {
    Function(FSRFn<'a>),
    Integer(FSRInteger),
    String(FSRString),
    List(FSRList),
    Bool(FSRBool),
    None,
}

impl FSRValue<'_> {}

#[derive(Debug)]
pub struct FSRClass {
    name: String,
    attrs: HashMap<String, u64>,
}

impl FSRClass {
    pub fn new<S>(name: S) -> FSRClass
    where
        S: ToString,
    {
        FSRClass {
            name: name.to_string(),
            attrs: HashMap::new(),
        }
    }

    pub fn init_cls(&mut self) {}

    pub fn get_name(&self) -> &str {
        return &self.name;
    }

    pub fn register_obj(&mut self, name: &str, obj_id: u64) {
        self.attrs.insert(name.to_string(), obj_id);
    }

    pub fn get_id_by_name(&self, name: &str) -> Option<&u64> {
        return self.attrs.get(name);
    }
}

#[derive(Debug)]
pub struct FSRObject<'a> {
    id: u64,
    obj_type: FSRObjectType,
    cls: Option<&'a FSRClass>,
    ref_count: u64,
    value: FSRValue<'a>,
    attrs: HashMap<&'a str, u64>,
}

impl<'a> FSRObject<'a> {
    pub fn get_id(&self) -> u64 {
        return self.id;
    }

    pub fn set_cls(&mut self, cls: &'a FSRClass) {
        self.cls = Some(cls);
    }

    pub fn get_cls_name(&self) -> &str {
        if let Some(s) = self.cls {
            return &s.name;
        }

        return "Object";
    }

    pub fn get_value(&self) -> &FSRValue {
        return &self.value;
    }

    pub fn new(vm: &'a FSRVirtualMachine<'a>) -> &'a mut FSRObject<'a> {
        let obj = FSRObject {
            obj_type: FSRObjectType::Object,
            cls: None,
            ref_count: 0,
            value: FSRValue::None,
            attrs: HashMap::new(),
            id: i_to_m(vm).new_id(),
        };
        let id = obj.id;
        i_to_m(vm).register_obj(obj);

        return i_to_m(vm).get_mut_obj_by_id(&id).unwrap();
    }

    pub fn new_with_id(vm: &'a FSRVirtualMachine<'a>, id: u64) -> &'a mut FSRObject<'a> {
        let obj = FSRObject {
            obj_type: FSRObjectType::Object,
            cls: None,
            ref_count: 0,
            value: FSRValue::None,
            attrs: HashMap::new(),
            id,
        };
        let id = obj.id;
        i_to_m(vm).register_obj(obj);

        return i_to_m(vm).get_mut_obj_by_id(&id).unwrap();
    }

    pub fn register(&mut self, name: &'a str, value: u64) {
        self.set_attr(name, value)
    }

    pub fn set_value(&mut self, v: FSRValue<'a>) {
        self.value = v;
    }

    pub fn deref_object(&mut self) {
        self.ref_count -= 1;
    }

    pub fn ref_object(&mut self) {
        self.ref_count += 1;
    }

    pub fn get_attr(&self, name: &str, vm: &'a FSRVirtualMachine<'a>) -> Option<&u64> {
        if let Some(s) = self.attrs.get(name) {
            return Some(s);
        }

        let fn_id = match self.cls {
            Some(s) => {
                let cls = vm.get_cls(s.get_name()).unwrap();
                cls.get_id_by_name(name)
            }
            None => {
                let cls = vm.get_cls("none").unwrap();
                cls.get_id_by_name(name)
            }
        };

        return fn_id;
    }

    pub fn invoke_method(
        &self,
        fn_name: &str,
        vm: &'a FSRVirtualMachine<'a>,
        rt: &'a FSRRuntimeModule<'a>,
    ) -> Result<u64, FSRRuntimeError> {
        let fn_id = match self.cls {
            Some(s) => {
                let cls = vm.get_cls(s.get_name()).unwrap();
                cls.get_id_by_name(fn_name)
            }
            None => {
                let cls = vm.get_cls("none").unwrap();
                cls.get_id_by_name(fn_name)
            }
        };

        let fn_obj = match fn_id {
            Some(s) => vm.get_obj_by_id(s),
            None => {
                let err = FSRRuntimeError::new(
                    rt.get_call_stack(),
                    FSRRuntimeType::NotFoundObject,
                    format!("Not found object id, {:?}", fn_id),
                    rt.get_cur_meta(),
                );
                return Err(err);
            }
        };

        let fn_obj = match fn_obj {
            Some(s) => s,
            None => {
                let err = FSRRuntimeError::new(
                    rt.get_call_stack(),
                    FSRRuntimeType::NotFoundObject,
                    format!("Not found object id, {:?}", fn_id),
                    rt.get_cur_meta(),
                );
                return Err(err);
            },
        };

        if let FSRValue::Function(f) = &fn_obj.value {
            i_to_m(rt).assign_variable("self", self.get_id(), vm)?;
            let v = f.invoke(vm, i_to_m(rt)).unwrap();
            return Ok(v);
        }

        let err = FSRRuntimeError::new(
            rt.get_call_stack(),
            FSRRuntimeType::TypeNotMatch,
            format!("{}::{} is not method or function", self.get_cls_name(), fn_name),
            rt.get_cur_meta(),
        );
        return Err(err);
    }

    pub fn set_attr(&mut self, name: &'a str, value: u64) {
        self.attrs.insert(name, value);
    }

    pub fn get_type(&self) -> &FSRObjectType {
        return &self.obj_type;
    }

    pub fn get_function(&self) -> Result<&FSRFn, Error> {
        if let FSRValue::Function(f) = &self.value {
            return Ok(f);
        }
        unimplemented!()
    }

    pub fn get_integer(&self) -> Result<&FSRInteger, Error> {
        if let FSRValue::Integer(i) = &self.value {
            return Ok(i);
        }

        unimplemented!()
    }

    pub fn get_string(&self) -> Result<&FSRString, Error> {
        if let FSRValue::String(s) = &self.value {
            return Ok(&s);
        }

        unimplemented!()
    }

    pub fn is_function(&self) -> bool {
        if let FSRValue::Function(_) = &self.value {
            return true;
        }

        return false;
    }
}

impl IFSRObject for FSRObject<'_> {
    fn get_class_name() -> &'static str {
        "Object"
    }

    fn get_class(_: &FSRVirtualMachine) -> FSRClass {
        let cls = FSRClass::new("Object");
        return cls;
    }

    fn init(&mut self) {}
}

pub type FSRAttrs<'a> = HashMap<&'a str, u64>;

pub type FSRArgs<'a> = Vec<(&'a str, u64)>;

#[derive(Debug)]
pub struct FSRObjectManager<'a> {
    objs: HashMap<u64, FSRObject<'a>>,
    cls_maps: HashMap<&'a str, FSRAttrs<'a>>,
}

impl<'a> FSRObjectManager<'a> {
    pub fn get_cls_maps(&mut self) -> &mut HashMap<&'a str, FSRAttrs<'a>> {
        return &mut self.cls_maps;
    }

    pub fn init_manager(&mut self) {}

    pub fn new() -> Self {
        let mut vm = Self {
            objs: HashMap::new(),
            cls_maps: HashMap::new(),
        };

        vm.init_manager();

        return vm;
    }

    pub fn get_mut_obj_by_id(&mut self, id: &u64) -> Option<&mut FSRObject<'a>> {
        match self.objs.get_mut(id) {
            Some(s) => {
                return Some(s);
            }
            None => {
                return None;
            }
        }
    }

    pub fn get_obj_by_id(&self, id: &u64) -> Option<&FSRObject<'a>> {
        match self.objs.get(id) {
            Some(s) => {
                return Some(s);
            }
            None => {
                return None;
            }
        }
    }

    pub fn register_obj(&mut self, id: u64, obj: FSRObject<'a>) {
        self.objs.insert(id, obj);
    }
}

#[derive(Debug)]
pub struct FSRVMClsMgr {
    cls_map: HashMap<String, FSRClass>,
}

impl FSRVMClsMgr {
    pub fn new(vm: &FSRVirtualMachine) -> FSRVMClsMgr {
        let mut mgr = Self {
            cls_map: HashMap::new(),
        };

        mgr.cls_map.insert(
            FSRObject::get_class_name().to_string(),
            FSRObject::get_class(vm),
        );
        mgr.cls_map.insert(
            FSRNone::get_class_name().to_string(),
            FSRNone::get_class(vm),
        );
        mgr.cls_map.insert(
            FSRBool::get_class_name().to_string(),
            FSRBool::get_class(vm),
        );
        mgr.cls_map.insert(
            FSRInteger::get_class_name().to_string(),
            FSRInteger::get_class(vm),
        );
        mgr.cls_map.insert(
            FSRString::get_class_name().to_string(),
            FSRString::get_class(vm),
        );
        mgr.cls_map.insert(
            FSRList::get_class_name().to_string(), 
            FSRList::get_class(vm)
        );
        return mgr;
    }

    pub fn get_cls(&self, name: &str) -> Option<&FSRClass> {
        return self.cls_map.get(name);
    }
}

pub trait IFSRObject {
    fn init(&mut self);

    fn get_class_name() -> &'static str;

    fn get_class(vm: &FSRVirtualMachine) -> FSRClass;
}
