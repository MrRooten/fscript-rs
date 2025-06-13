import thread

fn abc() {
    println("abc")
    id = __get_cur_thread_id()
    print("abc thread id: ")
    println(id)
}

t2 = thread::Thread(|| {
    println("this is lambda")
}, [])

t = thread::Thread(abc, [])

fn ddc() {
    println("ddc")
    id = __get_cur_thread_id()
    print("ddc thread id: ")
    println(id)
}

fn thread_with_args(i) {
    println(i)
}


__new_thread(ddc)

__new_thread(thread_with_args, 4)

id = __get_cur_thread_id()
println("main")
print("main thread id: ")
println(id)
println("sleep 10s")
sleep(10000)

