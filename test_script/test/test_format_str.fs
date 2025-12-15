a = "abc"
c = f"hello {1 + 1}"
assert(c == "hello 2")

b = f"hello {f"abc{1}"}"
assert(b == "hello abc1")

c = f"{1}"
assert(c == "1")

d = f"{1}, {2}"
assert(d == "1, 2")

e = f"{3}{4}"
assert(e == "34")