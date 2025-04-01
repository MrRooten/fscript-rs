i = 0
while i < 30000 {
    i = i + 1
}

gc_info()
gc_collect()
gc_info()