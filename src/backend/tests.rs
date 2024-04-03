#![allow(unused)]

#[cfg(test)]
mod backend_tests {
    use crate::backend::{base_type::{base::{FSRArgs, FSRObject}, integer::FSRInteger}, vm::vm::FSRVirtualMachine};
    use crate::backend::base_type::utils::i_to_m;
    use crate::backend::vm::module::FSRRuntimeModule;

    struct Test {
        s: String
    }

    impl Test {
        pub fn test(&mut self, s: &mut String) {

        }

        pub fn test1(&mut self, s: &mut String) {

        }
    }

    #[test]
    fn test_string() {
        let mut vm = FSRVirtualMachine::new().unwrap();
        let mut context = FSRRuntimeModule::new();
        let code = "
        fn abc(b) {
            c = b + 3
            if c > 2 {
                println('ok')
                dump_obj(c)
            }
            println(c)
            println(b)
        }
        a = 1

        abc(a)
        ";
        
        vm.run_code(code.as_bytes(), &mut context);
    }

    #[test]
    fn test_print() {
        let mut vm = FSRVirtualMachine::new().unwrap();
        let mut context = FSRRuntimeModule::new();
        let code = "
        fn abc() {
            println('this abc function')
        }
        a = [1 + 1, 2, 3]
        println(a)

        b = 34
        if b > 5 {
            println('b bigger than 5')
        }

        abc()
        ";
        
        vm.run_code(code.as_bytes(), &mut context);
    }
}