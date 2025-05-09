# a = {}

# for i in range(1000000):
#     a[i] = i

# for i in range(1000000):
#     b = a[i]
#     if b != i:
#         raise AssertionError("Value mismatch")

a = {}

(0...1_000_000).each do |i|
    a[i] = i
end

(0...1_000_000).each do |i|
    b = a[i]
    raise "Value mismatch" if b != i
end