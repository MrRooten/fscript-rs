import fs


f = fs::File.open("./.gitignore")
out = f.read_all()
for line in out.split("\n") {
    println("------ new line ------")
    println(line)
}