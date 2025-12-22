@async
fn yield_fn() {
    for i in 0..3 {
        i.yield
    }
}

@async
fn delegate_test() {
    res = yield_fn()

    res.delegate

    res2 = yield_fn()

    res2.delegate
}

delegate_obj = delegate_test()

for i in delegate_obj {
    println(i)
}

dump(delegate_obj)