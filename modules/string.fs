struct String {
    chars: Ptr[u8]
    len: u64

    @static
    fn init(self: Ptr[String]) {
        self.chars = u8.alloc(100)
        self.len = 0
    }
}

@entry
fn test() -> u64 {
    
}