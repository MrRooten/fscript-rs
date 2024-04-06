use fscript_rs::backend::vm::{runtime::FSRThreadRuntime, vm::FSRVirtualMachine};

fn main() {
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

            fn __new__(self, cdf) {
                self.cccc = cdf 
            }

            fn t(self, cdf) {
                return cdf
            }

            fn __str__(self) {
                return 'this is __str__'
            }
        }

        b = 1 + 1
        println(b)

        c = Abc('')
        println(c)

        f = c.t('this is t func')
        println(f)
        a = 1
        while a < 300000 {
            a += 1
        }
        ";
        
        vm.run_code(code.as_bytes(), &mut thread);
}
