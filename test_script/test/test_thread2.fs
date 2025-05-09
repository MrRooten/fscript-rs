fn ddc() {
    println("ddc")
    id = __get_cur_thread_id()
    print("ddc thread id: ")
    println(id)
    println("sleep 10s not exist")
    sleep(10000)
}


out = __new_thread(|| {
    println("abc")
})

out.join()