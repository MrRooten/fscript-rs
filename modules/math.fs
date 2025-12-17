class Number {
    fn __new__(self, s) {
        self.store = s
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

