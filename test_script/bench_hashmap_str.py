import gc

# t = HashMap::new()
# for i in 0..1000000 {
#     a = str(i)
#     t.insert(a, i)
# }

# for i in 0..1000000 {
#     a = str(i)
#     v = t.get(a)
# }

# gc_info()

t = {}
for i in range(1000000):
    a = str(i)
    t.update({a: i})

for i in range(1000000):
    a = str(i)
    v = t.get(a)

gc.collect()
print(gc.get_stats())