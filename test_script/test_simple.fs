import args

opt = args::ArgOption("name")
opt.set_follow()
opt.set_helper("test name value")
opt2 = args::ArgOption("file")
opt2.set_follow()
opt2.set_helper("test file value")
parser = args::ArgParser(["--name", "abc", "-f", "value2"])

parser.add_option([opt, opt2])
res = parser.parse()
println(res)

parser.helper()