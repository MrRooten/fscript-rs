use std::collections::{HashMap, btree_set::Intersection};

use fscript_rs::backend::base_type::{integer::FSRInteger, base::FSRObjectManager, utils::{m_to_i, i_to_m}};

fn main() {
    let mut vm = FSRObjectManager::new();
    let i_vm = m_to_i(&mut vm);
    let i1 = FSRInteger::from_u32(3);
    let i1_id = i_to_m(i_vm).register_object(i1);
    let i2 = FSRInteger::from_u32(4);
    let i2_id = i_to_m(i_vm).register_object(i2);

    let integer1 = i_to_m(i_vm).get_mut_obj_by_id(&i1_id).unwrap();
    let integer2 = i_vm.get_obj_by_id(&i2_id).unwrap();
    
    let args = &HashMap::from([("other", i2_id),("self", i1_id)]);
    let obj = i_to_m(i_vm).call_object_method(&integer1, "add", &args).unwrap();
    println!("{:?}", obj);

}
 