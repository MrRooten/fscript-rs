import thread

fn abc() {
    i = 0
    while i < 3000000 {
        i = i + 1
    }
}

th = thread::Thread(abc)
th.join()