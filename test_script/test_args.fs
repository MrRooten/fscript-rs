import args

opt = args::ArgOption("name")
opt.set_follow()
opt.set_helper("test name value")
opt2 = args::ArgOption("file")
opt2.set_follow()
opt2.set_helper("test file value")
default_opt = args::ArgOption("default")
default_opt.set_default("default_value")
side_opt = args::ArgOption("side_file")
side_opt.set_follow()
parser = args::ArgParser(["--name", "abc", "-f", "value2", "-sf", "test"])

parser.add_option([opt, opt2, default_opt, side_opt])
res = parser.parse()
println(res)

parser.helper()