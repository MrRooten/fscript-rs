class SortItem {
    fn __new__(self, value) {
        self.value = value
        return self
    }

    fn __gt__(self, other) {
        return self.value > other.value
    }

    fn __str__(self) {
        return "SortItem" + "(" + str(self.value) + ")"
    }
}

a1 = [SortItem(3), SortItem(4), SortItem(1), SortItem(2), SortItem(5)]

