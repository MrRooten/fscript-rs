class Test:
    def abc(self):
        pass
    
    def __add__(self, other):
        return 1
    
    def __init__(self):
        self.ddc = 0

t = Test()
t.abc()
t.index = 1
b = 1
for i in range(3000000):
    c = t.abc()

    
