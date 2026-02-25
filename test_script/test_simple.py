def test():
    a = [1, 2, None, 3]
    for i in a:
        yield i


for i in test():
    print(i)