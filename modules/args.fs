class ArgOption {
    fn __new__(self, name) {
        self.name = name
        self.follow_value = false
        self.value = none
        return self
    }

    fn __str__(self) {
        return "ArgOption { name:{}, follow_value:{}, value: {} }".format(self.name, 
        self.follow_value, 
        self.value)
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
        
    }

    fn set_option(self, option: ArgOption) {
        self.options.push(option)
    }

    fn __str__(self) {
        return "{}: {}".format(self.args, self.options)
    }
}