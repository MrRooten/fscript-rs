class Ddc {
    fn __new__(self) {
        self.ddc = 123 + 1
        return self
    }
}

class Abc {
    fn __new__(self) {
        self.abc = Ddc()
        dump(self.abc)
        return self
    }

    fn __str__(self) {
        return 'return string'
    }

    fn test(self) {
        return 323
    }
}

a = Abc()

dump(a)

dump(a)