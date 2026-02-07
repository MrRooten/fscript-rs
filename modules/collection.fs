class Counter {
    fn __new__(self, vec) {
        self.inner_map = HashMap::new()
        self.inner_map.set_default(0)

        for item in vec {
            self.inner_map[item] += 1
        }
    }

    fn __str__(self) {
        return f"Counter({self.inner_map})"
    }

    fn most_common(self, n: Integer) {
        items = self.inner_map.__iter__().as_list()
        items.sort_by(|a, b| {
            return b[1] - a[1] > 0
        })

        return items[0..n]
    }
}