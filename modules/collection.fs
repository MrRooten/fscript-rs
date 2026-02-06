class Counter {
    fn __new__(self, vec) {
        self.inner_map = HashMap::new()
        for item in vec {
            self.inner_map.insert(item, 0)
        }

        for item in vec {
            self.inner_map[item] += 1
        }
    }
}