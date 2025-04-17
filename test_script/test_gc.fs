i = 0
while i < 30000 {
    i = i + 1
}

gc_info()
gc_collect()
gc_info()

c = 1 + 2

v = gc_referers()
for i in v {
    println(i)
}