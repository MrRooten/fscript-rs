
# def fib(n):
#     if n == 1 or n == 2:
#         return 1
#     else:
#         return fib(n - 1) + fib(n - 2)

# for i in range(18000000):
#     fib(2)

def fib(n)
    return 1 if n == 1 || n == 2
    fib(n - 1) + fib(n - 2)
end

18_000_000.times do
    fib(2)
end