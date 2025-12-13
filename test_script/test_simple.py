class Test:
    def abc(self) -> int:
        a = 1
        return a

    def __add__(self, other: "Test") -> int:
        return 1


t: Test = Test()
t.abc()

for i in range(3_000_000):
    t.abc()