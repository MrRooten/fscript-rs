import thread

fn abc() {
    id = __get_cur_thread_id()
    print("abc thread id: ")
    println(id)
    i = 0
    while i < 3000000 {
        i = i + 1
    }
}

id = __get_cur_thread_id()
print("__main__ thread id: ")
println(id)

th = thread::Thread(abc)
th.join()