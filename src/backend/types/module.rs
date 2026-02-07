use std::{collections::HashMap, fmt::Debug, ptr::addr_of, sync::{Arc, atomic::AtomicUsize}};

use ahash::AHashMap;

use crate::backend::{compiler::bytecode::{FSRSType, FSRSTypeInfo, FSRStruct, FnCallSig}, vm::{thread::FSRThreadRuntime, virtual_machine::gid}};

use super::{base::{AtomicObjId, GlobalObj, FSRObject, FSRValue, ObjId}, class::FSRClass};


pub type NewModuleFn = fn(&mut FSRThreadRuntime) -> FSRValue<'static>;

pub struct FSRModule<'a> {
    name: String,
    fn_map: HashMap<String, FSRObject<'a>>,
    pub(crate) object_map: AHashMap<String, AtomicObjId>,
    pub(crate) jit_code_map: Vec<(Option<Arc<FSRSType>>, String, AtomicUsize)>, // JITed code address map
    pub(crate) type_info: FSRSTypeInfo
    // pub(crate) const_table: Vec<Option<ObjId>>,
}

impl Debug for FSRModule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fn_map_debug = HashMap::new();
        for v in self.fn_map.iter() {
            let addr = addr_of!(self.fn_map) as usize;
            fn_map_debug.insert(v.0.as_str(), addr);
        }
        f.debug_struct("FSRModule")
            .field("name", &self.name)
            .field("fn_map", &fn_map_debug)
            .field("object_map", &self.object_map)
            .finish()
    }
}

impl<'a> FSRModule<'a> {

    pub fn get_jit_code_map(&self, s: Option<Arc<FSRSType>>, name: &str) -> Option<&AtomicUsize> {
        for (struct_opt, fn_name, addr) in &self.jit_code_map {
            if struct_opt.is_none() && s.is_none() && fn_name == name {
                return Some(addr);
            }
            if let (Some(st1), Some(st2)) = (struct_opt, &s) {
                if st1.eq(st2) && fn_name == name {
                    return Some(addr);
                }

                if let FSRSType::Ptr(inner) = st2.as_ref() {
                    if st1.eq(inner) && fn_name == name {
                        return Some(addr);
                    }
                }
            }
        }
        None
    }

    /// Get the JITed function address pointer by name
    /// # Arguments
    /// * `name` - The name of the function
    /// # Returns
    /// * `Option<usize>` - The address pointer of the JITed function
    pub fn get_fn_addr_ptr(&self, s: Option<Arc<FSRSType>>, name: &str) -> Option<usize> {
        // Use for lazy JIT function address retrieval
        // let v = self.jit_code_map.get(name).and_then(|x| Some(x as *const AtomicUsize as usize));
        // match v {
        //     Some(addr) => addr,
        //     None => 0,
        // }
        match self.get_jit_code_map(s, name) {
            Some(addr) => Some(addr as *const AtomicUsize as usize),
            None => None,
        }
        
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn as_string(&self) -> String {
        format!("Module: {}", self.name)
    }

    pub fn new_object(name: &str) -> FSRObject<'a> {
        let module = FSRModule {
            name: name.to_string(),
            fn_map: HashMap::new(),
            object_map: AHashMap::new(),
            jit_code_map: vec![],
            type_info: FSRSTypeInfo::new(),
            // const_table: vec![],
        };
        let mut object = FSRObject::new();
        object.value = FSRValue::Module(Box::new(module));
        object.cls = FSRObject::id_to_obj(gid(GlobalObj::ModuleCls)).as_class();
        object
    }

    pub fn new_value(name: &str) -> FSRValue<'a> {
        let module = FSRModule {
            name: name.to_string(),
            fn_map: HashMap::new(),
            object_map: AHashMap::new(),
            jit_code_map: vec![],
            type_info: FSRSTypeInfo::new(),
            // const_table: vec![],
        };
        FSRValue::Module(Box::new(module))
    }

    pub fn new_module(name: &str) -> FSRModule<'a> {
        FSRModule {
            name: name.to_string(),
            fn_map: HashMap::new(),
            object_map: AHashMap::new(),
            jit_code_map: vec![],
            type_info: FSRSTypeInfo::new(),
            // const_table: vec![],
        }
    }
    

    pub fn init_fn_map(&mut self, fn_map: (HashMap<String, FSRObject<'a>>, FSRSTypeInfo)) {
        self.fn_map = fn_map.0;
        self.type_info = fn_map.1;
    }

    pub fn get_class() -> FSRClass {
        FSRClass::new("FSRModule")
    }

    pub fn get_fn(&self, name: &str) -> Option<&FSRObject<'a>> {
        self.fn_map.get(name)
    }

    pub fn iter_fn(&self) -> impl Iterator<Item = (&String, &FSRObject<'a>)> {
        self.fn_map.iter()
    }

    pub fn register_object(&mut self, name: &'a str, obj_id: ObjId) {
        self.object_map
            .insert(name.to_string(), AtomicObjId::new(obj_id));
    }

    pub fn get_object(&self, name: &str) -> Option<&AtomicObjId> {
        self.object_map.get(name)
    }

    // pub fn insert_const(&mut self, const_index: usize, obj: ObjId) {
    //     if const_index >= self.const_table.len() {
    //         self.const_table.resize(const_index + 1, None);
    //     }
    //     self.const_table[const_index] = Some(obj);
    // }

    // #[inline(always)]
    // pub fn get_const(&self, const_index: usize) -> Option<ObjId> {
    //     if const_index < self.const_table.len() {
    //         self.const_table[const_index]
    //     } else {
    //         None
    //     }
    // }
}