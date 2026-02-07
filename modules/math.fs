class Number {
    fn __new__(self, s) {
        self.store = s
    }

    fn __add__(self, other) {
        return Number(self.store + other.store)
    }

    fn __str__(self) {
        return self.store
    }
}

class Div {
    fn __new__(self, numerator, denominator) {
        self.numerator = numerator
        self.denominator = denominator
    }

    fn dump(self) {
        println("numerator: ")
        println(self.numerator)
        println("denominator: ")
        println(self.denominator)
    }
}

