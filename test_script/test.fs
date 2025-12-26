fn normal() {
    // ....
}

@static
fn static_jit() {
    i: cint8 = 1
    while i < 300 {
        i = i + 1
    }

    // can call dynamic function
    v: Object = normal()
    s: cstr = v.as_str()
    // when in dynamic function will auto convert to String 
    // when in static function just return value
    return s 
}