class Ddc {
    fn __new__(self) {
        self.ddc = 123 + 1
        return self
    }
}

class Abc {
    fn __new__(self) {
        self.abc = Ddc()
        self.c = 3
        return self
    }

    fn __str__(self) {
        return 'return string'
    }

    fn test(self) {
        return 323
    }
}

fn abc() {
    a = Abc()

    a = 3
}

abc()
