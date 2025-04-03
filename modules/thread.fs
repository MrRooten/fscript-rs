class Thread {
    fn __new__(self, f) {
        self.handle = __new_thread(f)
        return self
    }

    fn join(self) {
        self.handle.join()
    }
}