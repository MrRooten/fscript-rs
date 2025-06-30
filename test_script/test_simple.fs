@async
fn abc() {
    for i in 0..3000000 {
        i.yield
    }
    println("done")
}


a = abc()
i = 0
while a.poll_future() != none {

}