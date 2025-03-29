class Test {
    fn abc(self) {
        a = 1
    }
}

t = Test()
t.abc()

b = 1
while b < 3000000 {
    b = b + 1
    t.abc()
}

