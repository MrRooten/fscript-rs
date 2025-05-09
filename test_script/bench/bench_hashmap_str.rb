# import gc


# t = {}
# for i in range(1000000):
#     a = str(i)
#     t[a] = i

# for i in range(1000000):
#     a = str(i)
#     v = t.get(a)

# gc.collect()
# print(gc.get_stats())

require 'objspace'

t = {}
1_000_000.times do |i|
    a = i.to_s
    t[a] = i
end

1_000_000.times do |i|
    a = i.to_s
    v = t[a]
end

GC.start
puts ObjectSpace.count_objects