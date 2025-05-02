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

c = thread::Thread(abc)
c.join()