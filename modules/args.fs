class ArgOption {
    fn __new__(self, name) {
        self.name = name
        self.follow_value = false
        self.value = none

        short_name = name[0]
        self.short_name = short_name
        return self
    }

    fn __str__(self) {
        return "ArgOption { name:{}, follow_value:{}, value: {}, short_name: {} }".format(self.name, 
        self.follow_value, 
        self.value,
        self.short_name)
    }
}

class ArgParser {
    fn __new__(self, args) {
        self.args = args
        self.options = []
        return self
    }

    fn get_args(self) {
        return self.args
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

        index = 0
        while index < self.args.len() {
            if self.args[index].starts_with("--") {
                # full name
                full_name_len = self.args[index].len()
                full_name = self.args[index][2..full_name_len]
                println(full_name)
                index = index + 1
                continue
            }

            if self.args[index].starts_with("-") {
                println(self.args[index])
            }
            index = index + 1
        }

        println(short_map)
    }

    fn add_option(self, option: ArgOption) {
        self.options.push(option)
    }

    fn __str__(self) {
        return "{}: {}".format(self.args, self.options)
    }
}