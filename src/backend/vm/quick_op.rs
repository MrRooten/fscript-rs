use crate::backend::types::{base::FSRGlobalObjId, fn_def::FSRRustFn};

use super::virtual_machine::get_object_by_global_id;

const OP_LEN: usize = 20;

pub type Lookup2D = [[Option<FSRRustFn>; OP_LEN]; OP_LEN];
pub type Lookup1D = [Option<FSRRustFn>; OP_LEN];

/// Quick op lookup for first initialize type like integer or float etc.
pub struct Ops {
    pub(crate) add: Lookup2D,
    pub(crate) sub: Lookup2D,
    pub(crate) less: Lookup2D,
    pub(crate) greater: Lookup2D,
    pub(crate) getter: Lookup2D,
    pub(crate) next: Lookup1D,
    pub(crate) equal: Lookup2D,
    pub(crate) reminder: Lookup2D,
    pub(crate) set_item: Lookup2D
}

impl Ops {
    pub fn new_init() -> Self {
        let mut add = [[None; OP_LEN]; OP_LEN];
        Self::insert(
            FSRGlobalObjId::IntegerCls as usize,
            FSRGlobalObjId::IntegerCls as usize,
            &mut add,
            crate::backend::types::integer::add,
        );

        Self::insert(
            get_object_by_global_id(FSRGlobalObjId::FloatCls) as usize,
            get_object_by_global_id(FSRGlobalObjId::FloatCls) as usize,
            &mut add,
            crate::backend::types::float::add,
        );

        Self::insert(
            FSRGlobalObjId::StringCls as usize,
            FSRGlobalObjId::StringCls as usize,
            &mut add,
            crate::backend::types::string::add,
        );

        let mut sub = [[None; OP_LEN]; OP_LEN];
        Self::insert(
            FSRGlobalObjId::IntegerCls as usize,
            FSRGlobalObjId::IntegerCls as usize,
            &mut sub,
            crate::backend::types::integer::sub,
        );
        Self::insert(
            get_object_by_global_id(FSRGlobalObjId::FloatCls) as usize,
            get_object_by_global_id(FSRGlobalObjId::FloatCls) as usize,
            &mut sub,
            crate::backend::types::float::sub,
        );


        let mut less = [[None; OP_LEN]; OP_LEN];
        Self::insert(
            FSRGlobalObjId::IntegerCls as usize,
            FSRGlobalObjId::IntegerCls as usize,
            &mut less,
            crate::backend::types::integer::less,
        );

        Self::insert(
            get_object_by_global_id(FSRGlobalObjId::FloatCls) as usize,
            get_object_by_global_id(FSRGlobalObjId::FloatCls) as usize,
            &mut less,
            crate::backend::types::float::less,
        );

        let mut greater = [[None; OP_LEN]; OP_LEN];
        Self::insert(
            FSRGlobalObjId::IntegerCls as usize,
            FSRGlobalObjId::IntegerCls as usize,
            &mut greater,
            crate::backend::types::integer::greater,
        );

        Self::insert(
            get_object_by_global_id(FSRGlobalObjId::FloatCls) as usize,
            get_object_by_global_id(FSRGlobalObjId::FloatCls) as usize,
            &mut greater,
            crate::backend::types::float::greater,
        );

        let mut equal = [[None; OP_LEN]; OP_LEN];
        Self::insert(
            FSRGlobalObjId::IntegerCls as usize,
            FSRGlobalObjId::IntegerCls as usize,
            &mut equal,
            crate::backend::types::integer::equal,
        );

        Self::insert(
            get_object_by_global_id(FSRGlobalObjId::FloatCls) as usize,
            get_object_by_global_id(FSRGlobalObjId::FloatCls) as usize,
            &mut equal,
            crate::backend::types::float::equal,
        );

        Self::insert(
            FSRGlobalObjId::StringCls as usize,
            FSRGlobalObjId::StringCls as usize,
            &mut equal,
            crate::backend::types::string::equal,
        );

        let mut next = [None; OP_LEN];
        next[FSRGlobalObjId::InnerIterator as usize] =
            Some(crate::backend::types::iterator::next_obj as FSRRustFn);

        let mut getter = [[None; OP_LEN]; OP_LEN];
        getter[get_object_by_global_id(FSRGlobalObjId::ListCls) as usize][FSRGlobalObjId::IntegerCls as usize] =
            Some(crate::backend::types::list::get_item as FSRRustFn);
        getter[get_object_by_global_id(FSRGlobalObjId::HashMapCls) as usize][FSRGlobalObjId::IntegerCls as usize] =
            Some(crate::backend::types::ext::hashmap::fsr_fn_hashmap_get_reference as FSRRustFn);


        let mut reminder = [[None; OP_LEN]; OP_LEN];
        Self::insert(
            FSRGlobalObjId::IntegerCls as usize,
            FSRGlobalObjId::IntegerCls as usize,
            &mut reminder,
            crate::backend::types::integer::reminder,
        );

        let mut set_item = [[None; OP_LEN]; OP_LEN];
        set_item[get_object_by_global_id(FSRGlobalObjId::ListCls) as usize][FSRGlobalObjId::IntegerCls as usize] =
            Some(crate::backend::types::list::set_item as FSRRustFn);

        Self {
            add,
            less,
            greater,
            next,
            getter,
            equal,
            reminder,
            sub,
            set_item,
        }

        
    }

    pub fn insert(
        i: usize,
        j: usize,
        ops: &mut [[Option<FSRRustFn>; OP_LEN]; OP_LEN],
        op: FSRRustFn,
    ) {
        ops[i][j] = Some(op);
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_add(&self, i: usize, j: usize) -> Option<FSRRustFn> {
        // is the square matrix, so self.add len is the same as self.add[i].len()
        if i < OP_LEN && j < OP_LEN {
            return self.add[i][j];
        }
        None
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_less(&self, i: usize, j: usize) -> Option<FSRRustFn> {
        // is the square matrix, so self.add len is the same as self.add[i].len()
        if i < OP_LEN && j < OP_LEN {
            return self.less[i][j];
        }
        None
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_equal(&self, i: usize, j: usize) -> Option<FSRRustFn> {
        // is the square matrix, so self.add len is the same as self.add[i].len()
        if i < OP_LEN && j < OP_LEN {
            return self.equal[i][j];
        }
        None
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_greater(&self, i: usize, j: usize) -> Option<FSRRustFn> {
        // is the square matrix, so self.add len is the same as self.add[i].len()
        if i < OP_LEN && j < OP_LEN {
            return self.greater[i][j];
        }
        None
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_next(&self, i: usize) -> Option<FSRRustFn> {
        self.next.get(i).cloned()?
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_getter(&self, i: usize, j: usize) -> Option<FSRRustFn> {
        if i < OP_LEN && j < OP_LEN {
            return self.getter[i][j];
        }
        None
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_reminder(&self, i: usize, j: usize) -> Option<FSRRustFn> {
        if i < OP_LEN && j < OP_LEN {
            return self.reminder[i][j];
        }
        None
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_sub(&self, i: usize, j: usize) -> Option<FSRRustFn> {
        if i < OP_LEN && j < OP_LEN {
            return self.sub[i][j];
        }
        None
    }


    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_set_item(&self, i: usize, j: usize) -> Option<FSRRustFn> {
        if i < OP_LEN && j < OP_LEN {
            return self.set_item[i][j];
        }
        None
    }
}
