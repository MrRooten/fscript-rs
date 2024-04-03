#[allow(invalid_reference_casting)]
pub fn i_to_m<T>(reference: &T) -> &mut T {
    unsafe {
        let const_ptr = reference as *const T;
        let mut_ptr = const_ptr as *mut T;
        &mut *mut_ptr
    }
}

pub fn m_to_i<T>(reference: &mut T) -> &T {
    unsafe {
        let const_ptr = reference as *mut T;
        let mut_ptr = const_ptr as *const T;
        &*mut_ptr
    }
}