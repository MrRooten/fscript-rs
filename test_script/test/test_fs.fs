import fs

res = fs::Dir::sub_paths("./test_script/test/")
for filename in res {
    println("./test_sciprt/", filename)
}