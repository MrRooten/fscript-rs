a = "abc"
c = f"hello {1 + 1}"
assert(c == "hello 2")

b = f"hello {f"abc{1}"}"
assert(b == "hello abc1")