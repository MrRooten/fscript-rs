#![allow(unused)]

#[cfg(test)]
mod backend_tests {
    use crate::backend::{base_type::{base::{FSRArgs, FSRObject}, integer::FSRInteger}, vm::vm::FSRVirtualMachine};
    use crate::backend::base_type::utils::i_to_m;
    use crate::backend::vm::runtime::FSRThreadRuntime;

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
        let mut context = FSRThreadRuntime::new();
        let code = "
        a = 1
        print(a)
        ";
        
        vm.run_code(code.as_bytes(), &mut context);
    }

    #[test]
    fn test_print() {
        let mut vm = FSRVirtualMachine::new().unwrap();
        let mut context = FSRThreadRuntime::new();
        let code = "
        b = 1
        fn abc() {
            println('this abc function')
        }

        a = [1,2,3,4]
        println(a)
        abc()
        ";
        
        vm.run_code(code.as_bytes(), &mut context);
    }

    #[test]
    fn test_while() {
        let mut vm = FSRVirtualMachine::new().unwrap();
        let mut thread = FSRThreadRuntime::new();
        let code = "
fn abc() {
    a = 1
    while a < 3 {
        a = a + 1
        c = 1
        while c < 3 {
            c = c + 1
            print(\"c: \")
            println(c)
        }
    }

    return 'abc'
}

c = abc()
println(c)
        ";
        
        vm.run_code(code.as_bytes(), &mut thread);
    }

    #[test]
    fn test_fn() {
        let mut vm = FSRVirtualMachine::new().unwrap();
        let mut thread = FSRThreadRuntime::new();
        let code = "

        fn abc() {
            if 3 > 1 {
                return 'ddc'
            }
            return 'abc'
        }

        a = abc()
        println(a)
        ";
        
        vm.run_code(code.as_bytes(), &mut thread);
    }

    #[test]
    fn test_class() {
        let mut vm = FSRVirtualMachine::new().unwrap();
        let mut thread = FSRThreadRuntime::new();
        let code = "
        class Abc {
            abc = 1

            fn test(self) {
                println('abc')
            }

            fn bbc(self) {
                println(self)
            }

            fn __new__(self) {
                self.cccc = 123 
            }

            fn t(self, cdf) {
                return cdf
            }
        }

        b = Abc()
        c = b.t('sdf')
        println(c)
        d = b.t('asfd')
        println(d)
        ";
        
        vm.run_code(code.as_bytes(), &mut thread);
    }
}