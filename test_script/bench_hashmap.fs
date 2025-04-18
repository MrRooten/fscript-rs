a = HashMap::new()

for i in 0..1000000 {
    a.insert(i, i)
}

gc_info()
a = none
gc_collect()
gc_info()
for i in 0..3000000 {

}

for i in 0..1000000 {
    
}

for i in 0..1000000 {
    
}
gc_collect()
gc_info()