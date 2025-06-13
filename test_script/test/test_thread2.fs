import thread


t2 = thread::Thread(|x| {
    println("this is lambda")
    println(x)
    println(Thread::thread_id())
}, [1, 2, 3])

t2.join()