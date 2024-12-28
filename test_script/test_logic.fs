true = 1 == 1
false = 1 == 2

test_true = (true and false) or true

assert(test_true)

test_not = not false


assert(test_not)