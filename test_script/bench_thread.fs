import thread

fn abc() {
    import thread
    id = thread::Thread::thread_id()
    println("thread id: ", id)
    i = 0
    while i < 3000000 {
        i = i + 1
    }
}

th = thread::Thread(abc)
th2 = thread::Thread(abc)
th3 = thread::Thread(abc)
th.join()
th2.join()
th3.join()
