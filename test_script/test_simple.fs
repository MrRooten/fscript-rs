i = 0
while i < 3000 {
    i = i + 1
}

gc_info()
gc_collect()
gc_info()

for i in 0..2 {
    if i == 1 {
        gc_info()
    }
}

gc_collect()
gc_info()