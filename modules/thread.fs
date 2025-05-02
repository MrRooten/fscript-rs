class Thread {
    fn __new__(self, f) {
        self.handle = __new_thread(f)
        return self
    }

    fn join(self) {
        self.handle.join()
    }

    fn thread_id() {
        return __get_cur_thread_id()
    }
}