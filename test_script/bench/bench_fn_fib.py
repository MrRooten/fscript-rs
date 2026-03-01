# fn abc() {
#     fn fib(n) {
#         if n == 1 or n == 2 {
#             return 1
#         } else {
#             return fib(n - 1) + fib(n - 2)
#         }
#     }

#     for i in 0..18000000 {
#     	fib(2)
#     }
# }

# abc()
from itertools import repeat
def fib(n):
    return 1

for i in repeat(None, 3000000):
    fib(2)