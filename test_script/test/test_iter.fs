class TestIter {
    fn __new__(self) {
        self.list = [1, 2, 3, 4, 5]
        self.index = 0
        return self
    }

    fn __next__(self) {
        if self.index < self.list.len() {
            self.index = self.index + 1
            return self.list[self.index - 1]
        } else {
            return none
        }
    }
}

class Test {
    fn __iter__(self) {
        a = TestIter()
        return a
    }
}

a = Test()

for i in a.__iter__() {
    println(i)
}

assert(i == 5)

for c in "abcd".__iter__() {
    println(c)
}


a = [0, 1, 2, 3].__iter__().any(|x| { return x == 2 })
assert(a == true)