struct String {
    chars: Ptr[u8]
    len: u64

    @static
    fn new() -> Ptr[String] {
        s: Ptr[String] = String.alloc
        s.chars = null
        s.len = 0
        return s
    }   
}