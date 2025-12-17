fn full_to_short(full_name) {
    short_name = ""
    for sub in full_name.split("_") {
        short_name = short_name + sub[0]
    }

    return short_name
}

class ArgOption {
    fn __new__(self, name) {
        self.name = name
        self.follow_value = false
        self.value = none

        short_name = full_to_short(self.name)
        self.short_name = short_name
        self.help_message = "not setting helper message "
    }

    fn __str__(self) {
        return "ArgOption { name:{}, follow_value:{}, value: {}}".format(self.name, 
        self.follow_value, 
        self.value)
    }

    fn set_follow(self) {
        self.follow_value = true
    }

    fn set_helper(self, message) {
        self.help_message = message
    }

    fn set_default(self, default_value) {
        self.value = default_value
    }
}

class ArgParser {
    fn __new__(self, args) {
        self.args = args
        self.options = []
    }

    fn get_args(self) {
        return self.args
    }

    fn get_full_name(self, short_name) {
        for opt in self.options {
            if opt.short_name == short_name {
                return opt.name
            }
        }
        return none
    }

    fn parse(self) {
        full_map = HashMap::new()
        for option in self.options {
            full_map[option.name] = option
        }

        short_map = HashMap::new()
        for option in self.options {
            short_map[option.short_name] = option
        }


        res_map = HashMap::new()

        for opt in self.options {
            if opt.value != none {
                res_map[opt.name] = opt.value
            }
        }

        index = 0
        while index < self.args.len() {
            if self.args[index].starts_with("--") {
                # full name
                full_name_len = self.args[index].len()
                full_name = self.args[index][2..full_name_len]
                option = full_map[full_name]

                if option == none {
                    index = index + 1
                    continue
                }

                println("option.follow_value: {}".format(option.follow_value))
                if option.follow_value == true {
                    index = index + 1
                    value = self.args[index]
                    res_map[full_name] = value
                } else {
                    res_map[full_name] = true
                }
                index = index + 1
                continue
            }

            else if self.args[index].starts_with("-") {
                println(self.args[index])
                short_name_len = self.args[index].len()
                short_name = self.args[index][1..short_name_len]
                option = short_map[short_name]
                if option == none {
                    index = index + 1
                    continue
                }

                full_name = self.get_full_name(short_name)
                if option.follow_value == true {
                    index = index + 1
                    value = self.args[index]
                    res_map[full_name] = value
                }
            }
            index = index + 1
        }

        return res_map
    }

    fn add_option(self, option: ArgOption) {
        if is_class(option, get_class([])) {
            for opt in option {
                self.options.push(opt)
            }

            return
        }
        self.options.push(option)
    }

    fn __str__(self) {
        return "{}: {}".format(self.args, self.options)
    }

    fn helper(self) {
        for option in self.options {
            println(" -{}, --{}:".format(option.short_name, option.name))
            println("    ", option.help_message)
        }
    }
}