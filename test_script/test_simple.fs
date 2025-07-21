import args

opt = args::ArgOption("name")
parser = args::ArgParser(["--n"])
parser.add_option(opt)

parser.parse()