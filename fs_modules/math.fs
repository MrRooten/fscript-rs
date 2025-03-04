class Number {
    fn __new__(self, s) {
        self.store = s
        return self
    }
}

class Div {
    fn __new__(self, numerator, denominator) {
        self.numerator = numerator
        self.denominator = denominator
        return self
    }   
}


v = Div(123, 345)
dump(v)