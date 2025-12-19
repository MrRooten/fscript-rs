# fn abc() {
#     fn fib(n) {
#         if n == 1 or n == 2 {
#             return 1
#         } else {
#             return fib(n - 1) + fib(n - 2)
#         }
#     }
#     result = fib(35)
#     println(result)
#     gc_info()
# }

# abc()

def fib(n)
    return 1 if n == 1 || n == 2
    fib(n - 1) + fib(n - 2)
end

def abc
    result = fib(29)
    puts result
    gc_info
end

def gc_info
    puts "GC count: #{GC.count}"
end

abc