true = 1 == 1

assert(true)

false = 1 != 1

assert(!false, "false is not false")

a = -1 * 10
assert(a == -10, "a is not -10")

b = 10 + -1 * 10
assert(b == 0, "b is not 0")

assert(true and not false, "true and not false is not true")

assert(true or false, "true or false is not true")

assert(not (true and false), "not (true and false) is not true")