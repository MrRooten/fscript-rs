class Thread {
    fn __new__(self, f) {
        self.f = f
        return self
    }

    fn start(self) {
        __new_thread(self.f)
    }
}