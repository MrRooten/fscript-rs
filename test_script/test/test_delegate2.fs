
@async
fn yield_fn() {
    println("yield_fn")
    for i in 0..3 {
        i.yield
    }
}

@async
fn delegate_inner() {
    println("delegate_inner")
    yield_fn().delegate

}

@async
fn delegate_test() {
    println("delegate_outer")
    res = delegate_inner()

    res.delegate
    println("delegate_outer2")
    res2 = delegate_inner()

    res2.delegate
}

delegate_obj = delegate_test()

for i in delegate_obj {
    println(i)
}

dump(delegate_obj)