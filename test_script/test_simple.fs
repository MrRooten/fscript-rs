@async
fn abc() {
    1.yield
    2.yield
    return 0
}


a = abc()
v1 = a.poll_future()
println(v1)
v2 = a.poll_future()
println(v2)
v3 = a.poll_future()
println(v3)

println(a)