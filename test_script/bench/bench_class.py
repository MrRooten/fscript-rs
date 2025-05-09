class Test:
    def abc(self):
        a = 1
        return a
    
    def __add__(self, other):
        return 1
    
    def __init__(self):
        self.ddc = 0

t = Test()
t.abc()
t.index = 1
b = 1
while t.index < 3000000:
    t.index = t.index + t.abc()
    t.ddc = 123 + 1
    c = t + 1

    
