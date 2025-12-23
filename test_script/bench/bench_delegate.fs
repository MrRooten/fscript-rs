@async
fn yield_fn() {
    for i in 0..3000000 {
        i.yield
    }
}

@async
fn delegate_test() {
    yield_fn().delegate

    yield_fn().delegate

    yield_fn().delegate
}

delegate_obj = delegate_test()

for i in delegate_obj {
    #println(i)
}

dump(delegate_obj)