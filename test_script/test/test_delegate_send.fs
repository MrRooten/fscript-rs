@async
fn yield_fn() {
    for i in 0..3 {
        println(f"send_value: {i.yield}")
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
delegate_obj.__next__()
delegate_obj.send(1)
delegate_obj.__next__()