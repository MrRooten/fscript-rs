class Test {
    fn abc(self) {
        a = ((1 + 3 + 1 ) * 3 + 4 + 2) * 4
    }
}

t = Test()
t.abc()

b = 1
while b < 3000000 {
    b = b + 1
    t.abc()
}