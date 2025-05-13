class Ddc {
    fn __new__(self) {
        self.ddc = 123 + 1
        return self
    }
}

class Abc {
    fn __new__(self) {
        self.abc = Ddc()
        return self
    }
    fn __str__(self) {
        return 'return string'
    }
}



a = Abc()

a.__str__()