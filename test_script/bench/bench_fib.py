import gc


# def fib(n)
#     return 1 if n == 1 || n == 2
#     fib(n - 1) + fib(n - 2)
# end

# def abc
#     result = fib(35)
#     puts result
#     gc_info
# end

# def gc_info
#     puts "GC count: #{GC.count}"
# end

# abc

def fib(n):
    if n == 1 or n == 2:
        return 1
    return fib(n - 1) + fib(n - 2)

def gc_info():
    print(f"GC count: {len(gc.get_objects())}")

def abc():
    result = fib(29)
    print(result)
    gc_info()

abc()