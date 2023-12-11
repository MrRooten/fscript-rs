use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::fmt::Error;
use std::ops::Deref;

use super::function::FSRFunction;
use super::integer::FSRInteger;
use super::string::FSRString;

#[derive(Debug)]
pub enum FSRObjectType {
    Function,
    Class,
    Integer,
    String,
    Bytes,
    Object,
    MetaClass,
}

#[derive(Debug)]
pub enum FSRValue {
    Function(FSRFunction),
    Integer(FSRInteger),
    String(FSRString),
    None
}

impl FSRValue {
    
}

#[derive(Debug)]
pub struct FSRClass<'a> {
    name        : &'a str,
}

impl<'a> FSRClass<'a> {
    pub const fn new(name: &'static str) -> FSRClass<'a> {
        FSRClass { name }
    }
}

#[derive(Debug)]
pub struct FSRObject<'a> {
    obj_type    : FSRObjectType,
    cls         : &'static FSRClass<'static>,
    ref_count   : u64,
    value       : FSRValue,
    attrs       : HashMap<&'a str, *mut FSRObject<'a>>

}


const OBJECT_CLASS: FSRClass = FSRClass::new("object");

impl<'a> FSRObject<'a> {
    pub fn set_cls(&mut self, cls: &'static FSRClass) {
        self.cls = cls;
    }

    pub fn get_cls_name(&self) -> &str {
        return self.cls.name;
    }

    pub fn new() -> FSRObject<'a> {
        FSRObject { obj_type: FSRObjectType::Object, 
            cls: &OBJECT_CLASS, 
            ref_count: 0, 
            value: FSRValue::None, 
            attrs: HashMap::new() 
        }
    }

    pub fn register(&mut self, name: &'a str, value: *mut FSRObject<'a>) {
        self.set_attr(name, value)
    }

    pub fn set_value(&mut self, v: FSRValue) {
        self.value = v;
    }

    fn deref_object(&mut self) {
        self.ref_count -= 1;
    }

    pub fn get_attr(&self, name: &str) -> Option<&mut FSRObject<'a>> {
        match self.attrs.get(name) {
            Some(s) =>  {
                let p = unsafe { &mut **s };
                return Some(p);
            },
            None => {
                None
            }
        }
    }

    pub fn call(&self, name: &str, args: &HashMap<&str, &FSRObject>) {
        unimplemented!()
    }

    pub fn set_attr(&mut self, name: &'a str, value: *mut FSRObject<'a>) {
        if let Some(s) = self.attrs.get_mut(name) {
            (unsafe { &mut **s }).deref_object();
        }

        self.attrs.insert(name, value);
        
        
    }

    pub fn get_type(&self) -> &FSRObjectType {
        return &self.obj_type;
    }

    pub fn get_function(&self) -> Result<&FSRFunction, Error> {
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
}

impl Drop for FSRObject<'_> {
    fn drop(&mut self) {
        for item in &mut self.attrs {
            unsafe { &mut **item.1 }.deref_object()
        }
    }
}

pub type FSRAttrs<'a> = HashMap<&'a str, FSRObject<'a>>;

pub type FSRArgs<'a> = HashMap<&'a str, u64>;

pub struct FSRObjectManager<'a> {
    objs    : HashMap<u64, FSRObject<'a>>,
    max_id  : u64,
    cls_maps    : HashMap<&'static str, FSRAttrs<'static>>
}

impl<'a> FSRObjectManager<'a> {
    pub fn get_cls_maps(&mut self) -> &mut HashMap<&'static str, FSRAttrs<'static>> {
        return &mut self.cls_maps;
    }

    fn init_internal_classes(&mut self) {
        self.cls_maps.insert(FSRInteger::get_class_name(), FSRInteger::get_attrs());
    }

    pub fn init_manager(&mut self) {

    }

    pub fn new() -> Self {
        let mut vm = Self {
            objs: HashMap::new(),
            max_id: 0,
            cls_maps: HashMap::new(),
        };

        vm.init_internal_classes();
        return vm;
    }

    pub fn register_object(&mut self, obj: FSRObject<'a>) -> u64 {
        let mut id = 0;
        match self.max_id.checked_add(1) {
            Some(s) => {
                self.objs.insert(self.max_id + 1, obj);
                id = self.max_id + 1;
            }, 
            None => {
                self.max_id = 0;
                self.objs.insert(self.max_id + 1, obj);
                id = self.max_id + 1;
            }
        };

        self.max_id += 1;
        return id;
        
    }

    pub fn get_mut_obj_by_id(&mut self, id: &u64) -> Option<&mut FSRObject<'a>> {
        match self.objs.get_mut(id) {
            Some(s) => {
                return Some(s);
            },
            None => {
                return None;
            }
        }
    }

    pub fn get_obj_by_id(&self, id: &u64) -> Option<&FSRObject<'a>> {
        match self.objs.get(id) {
            Some(s) => {
                return Some(s);
            },
            None => {
                return None;
            }
        }
    }

    pub fn call_object_method(&self, object: &FSRObject, fn_name: &str, args: &FSRArgs) -> Result<FSRObject<'static>, Error> {
        let name = object.get_cls_name();
        let attrs = self.cls_maps.get(name).unwrap();
        let func_obj: &FSRObject<'_> = attrs.get(fn_name).unwrap();
        let func = func_obj.get_function().unwrap();
        func.invoke(args, self)
    }

    pub fn new_object(&mut self, cls_name: &str, args: &FSRArgs) -> Result<&FSRObject, Error>{
        unimplemented!()
    }

    
}


pub trait FSRClassRegister {
    fn get_class_name() -> &'static str;
    fn register_attrs(manager: &mut FSRObjectManager);
    fn get_attrs() -> FSRAttrs<'static>;
    fn get_cls_name(&self) -> &'static str;
}