use std::{collections::HashMap, os::unix::thread};

use cranelift::{
    codegen,
    prelude::{
        settings, types, AbiParam, Block, Configurable, EntityRef, FunctionBuilder,
        FunctionBuilderContext, InstBuilder, Type, Value, Variable,
    },
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;

use crate::backend::{
    compiler::bytecode::{
        ArgType, BinaryOffset, Bytecode, BytecodeArg, BytecodeOperator, CompareOperator,
    },
    types::base::{FSRObject, ObjId},
    vm::thread::FSRThreadRuntime,
};

use super::jit_wrapper::{
    binary_op, call_fn, check_gc, compare_test, free, gc_collect, get_constant, get_obj_by_name,
    is_false, malloc,
};

struct BuildContext {}

pub struct CraneLiftJitBackend {
    ctx: codegen::Context,
    builder_context: FunctionBuilderContext,
    //variable: HashMap<String, Variable>,
    module: JITModule,
}

struct JitBuilder<'a> {
    int: types::Type,
    builder: FunctionBuilder<'a>,
    variables: HashMap<String, Variable>,
    module: &'a mut JITModule,
}

struct OperatorContext {
    exp: Vec<Value>,
    operator: &'static str,
    while_blocks: Vec<Block>,
    exit_blocks: Vec<Block>,
    entry_block: Block,
}

impl JitBuilder<'_> {
    fn load_constant(&mut self, c: u64, context: &mut OperatorContext) {
        let mut get_constant_sig = self.module.make_signature();
        get_constant_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // code object
        get_constant_sig.params.push(AbiParam::new(types::I64)); // constant index
        get_constant_sig
            .returns
            .push(AbiParam::new(self.module.target_config().pointer_type())); // return type
        let fn_id = self
            .module
            .declare_function(
                "get_constant",
                cranelift_module::Linkage::Import,
                &get_constant_sig,
            )
            .unwrap();
        println!("get_constant: {:?}", fn_id);
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let code = self.builder.block_params(context.entry_block)[1];
        let index = self.builder.ins().iconst(types::I64, c as i64);
        let call = self.builder.ins().call(func_ref, &[code, index]);
        let ret = self.builder.inst_results(call)[0];
        context.exp.push(ret);
    }

    fn load_global_name(&mut self, name: Value, name_len: Value, context: &mut OperatorContext) {
        // pub extern "C" fn get_obj_by_name(name: *const u8, len: usize, thread: &mut FSRThreadRuntime) -> ObjId
        let mut get_obj_by_name_sig = self.module.make_signature();
        get_obj_by_name_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // name pointer
        get_obj_by_name_sig.params.push(AbiParam::new(types::I64)); // name length
        get_obj_by_name_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
        get_obj_by_name_sig
            .returns
            .push(AbiParam::new(self.module.target_config().pointer_type())); // return type
        let fn_id = self
            .module
            .declare_function(
                "get_obj_by_name",
                cranelift_module::Linkage::Import,
                &get_obj_by_name_sig,
            )
            .unwrap();
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let thread_rt = self.builder.block_params(context.entry_block)[0];
        let get_global_name = self
            .builder
            .ins()
            .call(func_ref, &[name, name_len, thread_rt]);
        let global_obj = self.builder.inst_results(get_global_name)[0];
        context.exp.push(global_obj);
    }

    fn is_true(&mut self, value: Value) -> Value {
        let mut is_true_sig = self.module.make_signature();
        is_true_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // value to check
        is_true_sig.returns.push(AbiParam::new(types::I32)); // return type (boolean)

        let fn_id = self
            .module
            .declare_function("is_true", cranelift_module::Linkage::Import, &is_true_sig)
            .unwrap();
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let ret = self.builder.ins().call(func_ref, &[value]);
        let result = self.builder.inst_results(ret)[0];
        result
    }

    fn load_compare(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        if let (Some(right), Some(left)) = (context.exp.pop(), context.exp.pop()) {
            // pub extern "C" fn compare_test(thread: &mut FSRThreadRuntime, left: ObjId, right: ObjId, op: CompareOperator)

            let mut compare_test_sig = self.module.make_signature();
            compare_test_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
            compare_test_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // left operand
            compare_test_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // right operand
            compare_test_sig.params.push(AbiParam::new(types::I64)); // compare operator type
            compare_test_sig.returns.push(AbiParam::new(types::I32)); // return type (boolean)
            let fn_id = self
                .module
                .declare_function(
                    "compare_test",
                    cranelift_module::Linkage::Import,
                    &compare_test_sig,
                )
                .unwrap();
            let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
            let thread_runtime = self.builder.block_params(context.entry_block)[0];
            // let op = self.builder.ins().iconst(types::I32, 0); // Replace with actual operator type
            // let op = CompareOperator::new_from_str(context.operator).unwrap() as i32;
            let op = if let ArgType::Compare(op) = arg.get_arg() {
                let v = *op as i64;
                self.builder.ins().iconst(types::I64, v)
            } else {
                panic!("CompareTest requires a CompareOperator argument")
            };
            let call = self
                .builder
                .ins()
                .call(func_ref, &[thread_runtime, left, right, op]);
            let result = self.builder.inst_results(call)[0];
            context.exp.push(result);
        } else {
            panic!("CompareTest requires both left and right operands");
        }
    }

    fn load_if_test(&mut self, context: &mut OperatorContext) {
        let condition = context.exp.pop().unwrap();
        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        let exit_block = self.builder.create_block();

        self.builder
            .ins()
            .brif(condition, then_block, &[], else_block, &[]);

        self.builder.switch_to_block(then_block);
        //context.exit_blocks.push(exit_block);
        //context.while_blocks.push(else_block);
    }

    fn load_while(&mut self, context: &mut OperatorContext) {
        //let header_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        let exit_block = self.builder.create_block();
        let condition = context.exp.pop().unwrap();
        self.builder
            .ins()
            .brif(condition, body_block, &[], exit_block, &[]);

        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);
        context.exit_blocks.push(exit_block);
    }

    fn load_while_end(&mut self, context: &mut OperatorContext) {
        self.builder
            .ins()
            .jump(context.while_blocks.last().unwrap().clone(), &[]);

        //context.is_while = false;
        let v = context.while_blocks.pop().unwrap();
        let exit_block = context.exit_blocks.pop().unwrap();
        self.builder.seal_block(v);
        self.builder.switch_to_block(exit_block);
        self.builder.seal_block(exit_block);
        //self.builder.ins().iconst(self.int, 0);
    }

    fn load_make_arg_list(&mut self, context: &mut OperatorContext, len: usize) -> Value {
        let mut malloc_sig = self.module.make_signature();
        malloc_sig.params.push(AbiParam::new(types::I64)); // size
        malloc_sig
            .returns
            .push(AbiParam::new(self.module.target_config().pointer_type())); // return type
        let malloc_id = self
            .module
            .declare_function("malloc", cranelift_module::Linkage::Import, &malloc_sig)
            .unwrap();
        let malloc_func_ref = self
            .module
            .declare_func_in_func(malloc_id, self.builder.func);

        let size = self.builder.ins().iconst(types::I64, len as i64);
        let malloc_call = self.builder.ins().call(malloc_func_ref, &[size]);
        let malloc_ret = self.builder.inst_results(malloc_call)[0];

        for i in 0..len {
            // Assuming we have a way to get the next argument value
            let arg_value = context.exp.pop().unwrap(); // This should be replaced with actual argument retrieval logic
            let offset = self
                .builder
                .ins()
                .iconst(types::I64, i as i64 * std::mem::size_of::<ObjId>() as i64); // Replace with actual offset calculation
            let ptr = self.builder.ins().iadd(malloc_ret, offset);
            self.builder
                .ins()
                .store(cranelift::codegen::ir::MemFlags::new(), arg_value, ptr, 0);
        }
        malloc_ret
    }

    fn load_free_arg_list(&mut self, list_ptr: Value, context: &mut OperatorContext, len: i64) {
        // pub extern "C" fn free(ptr: *mut Vec<ObjId>, size: usize)
        let mut free_sig = self.module.make_signature();
        free_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // pointer to the list
        free_sig.params.push(AbiParam::new(types::I64)); // size of the list
        free_sig.returns.push(AbiParam::new(types::I32)); // return type (void)
        let free_id = self
            .module
            .declare_function("free", cranelift_module::Linkage::Import, &free_sig)
            .unwrap();
        let free_func_ref = self.module.declare_func_in_func(free_id, self.builder.func);
        let size = self.builder.ins().iconst(types::I64, len);
        let free_call = self.builder.ins().call(free_func_ref, &[list_ptr, size]);
        let _ = self.builder.inst_results(free_call)[0]; // We don't need the return value, just ensure the call is made
    }

    fn load_gc_collect(&mut self, context: &mut OperatorContext) {
        let ptr_type = self.module.target_config().pointer_type();
        let var_count = self.variables.len();
        let size = self.builder.ins().iconst(types::I64, var_count as i64);

        let mut malloc_sig = self.module.make_signature();
        malloc_sig.params.push(AbiParam::new(types::I64));
        malloc_sig.returns.push(AbiParam::new(ptr_type));
        let malloc_id = self
            .module
            .declare_function("malloc", cranelift_module::Linkage::Import, &malloc_sig)
            .unwrap();
        let malloc_func_ref = self
            .module
            .declare_func_in_func(malloc_id, self.builder.func);
        let malloc_call = self.builder.ins().call(malloc_func_ref, &[size]);
        let arr_ptr = self.builder.inst_results(malloc_call)[0];

        for (i, var) in self.variables.values().enumerate() {
            let value = self.builder.use_var(*var);
            let offset = self
                .builder
                .ins()
                .iconst(types::I64, (i * std::mem::size_of::<ObjId>()) as i64);
            let ptr = self.builder.ins().iadd(arr_ptr, offset);
            self.builder
                .ins()
                .store(cranelift::codegen::ir::MemFlags::new(), value, ptr, 0);
        }

        let mut gc_collect_sig = self.module.make_signature();
        // pub extern "C" fn gc_collect(thread: &mut FSRThreadRuntime, list_obj: *const ObjId, len: usize)
        gc_collect_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
        gc_collect_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // list pointer
        gc_collect_sig.params.push(AbiParam::new(types::I64)); // length of the list

        gc_collect_sig
            .returns
            .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (void)
        let gc_collect_id = self
            .module
            .declare_function(
                "gc_collect",
                cranelift_module::Linkage::Import,
                &gc_collect_sig,
            )
            .unwrap();

        let gc_collect_func_ref = self
            .module
            .declare_func_in_func(gc_collect_id, self.builder.func);
        let thread_runtime = self.builder.block_params(context.entry_block)[0];
        let len = self.builder.ins().iconst(types::I64, var_count as i64);

        let gc_call = self
            .builder
            .ins()
            .call(gc_collect_func_ref, &[thread_runtime, arr_ptr, len]);
        let _ = self.builder.inst_results(gc_call)[0]; // We don't need the return value, just ensure the call is made
                                                       // Free the allocated array after the GC call
        self.load_free_arg_list(arr_ptr, context, var_count as i64);
    }

    fn load_check_gc(&mut self, context: &mut OperatorContext) -> Value {
        let mut check_gc_sig = self.module.make_signature();
        check_gc_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
        check_gc_sig.returns.push(AbiParam::new(types::I32)); // return type (boolean)

        let fn_id = self
            .module
            .declare_function("check_gc", cranelift_module::Linkage::Import, &check_gc_sig)
            .unwrap();
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let thread_runtime = self.builder.block_params(context.entry_block)[0];
        let ret = self.builder.ins().call(func_ref, &[thread_runtime]);
        let condition = self.builder.inst_results(ret)[0];

        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        self.builder
            .ins()
            .brif(condition, then_block, &[], else_block, &[]);

        self.builder.switch_to_block(then_block);
        self.builder.seal_block(then_block);
        self.load_gc_collect(context);
        self.builder.ins().jump(else_block, &[]);
        self.builder.switch_to_block(else_block);
        self.builder.seal_block(else_block);
        condition
    }

    fn load_call(&mut self, arg: &BytecodeArg, context: &mut OperatorContext) {
        if let ArgType::CallArgsNumber(v) = arg.get_arg() {
            //let variable = self.variables.get(v.2.as_str()).unwrap();
            // context.left = Some(self.builder.use_var(*variable));
            //let fn_obj_id = self.builder.use_var(*variable);

            // call_fn(args: *const ObjId, len: usize, fn_id: ObjId, thread: &mut FSRThreadRuntime, code: ObjId) -> ObjId
            let mut call_fn_sig = self.module.make_signature();
            call_fn_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // args
            call_fn_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // len
            call_fn_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // fn_obj_id
            call_fn_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
            call_fn_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // code object
            call_fn_sig
                .returns
                .push(AbiParam::new(self.module.target_config().pointer_type()));

            let fn_id = self
                .module
                .declare_function("call_fn", cranelift_module::Linkage::Import, &call_fn_sig)
                .unwrap();
            let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
            let list_ptr = self.load_make_arg_list(context, *v);
            let len = self.builder.ins().iconst(types::I64, *v as i64);
            let thread_runtime = self.builder.block_params(context.entry_block)[0];
            let code_object = self.builder.block_params(context.entry_block)[1];

            let fn_obj_id = context.exp.pop().unwrap();
            let call = self.builder.ins().call(
                func_ref,
                &[list_ptr, len, fn_obj_id, thread_runtime, code_object],
            );

            let ret = self.builder.inst_results(call)[0];

            // Free the argument list after the call
            self.load_free_arg_list(list_ptr, context, *v as i64);

            context.exp.push(ret);
        } else {
            unimplemented!()
        }
    }

    fn compile_expr(&mut self, expr: &[BytecodeArg], context: &mut OperatorContext) {
        if expr.last().is_none() {
            return;
        }

        if expr.last().unwrap().get_operator() == &BytecodeOperator::WhileTest {
            //context.is_while += 1;
            let header_block = self.builder.create_block();
            self.builder.ins().jump(header_block, &[]);
            self.builder.switch_to_block(header_block);
            context.while_blocks.push(header_block);
        }

        if expr.last().unwrap().get_operator() == &BytecodeOperator::IfTest {
            //context.is_if += 1;
            let header_block = self.builder.create_block();
            self.builder.ins().jump(header_block, &[]);
            self.builder.switch_to_block(header_block);
        }

        for arg in expr {
            match arg.get_operator() {
                BytecodeOperator::Load => {
                    if let ArgType::Local(v) = arg.get_arg() {
                        let variable = self.variables.get(v.1.as_str()).unwrap();
                        // context.left = Some(self.builder.use_var(*variable));
                        let value = self.builder.use_var(*variable);
                        context.exp.push(value);
                    } else if let ArgType::Const(c) = arg.get_arg() {
                        self.load_constant(*c, context);
                    } else if let ArgType::Global(name) = arg.get_arg() {
                        // let data_id = self
                        //     .module
                        //     .declare_data(name, cranelift_module::Linkage::Export, false, false)
                        //     .unwrap();
                        // let local_id = self.module.declare_data_in_func(data_id, self.builder.func);
                        // let name_ptr = self.builder.ins().symbol_value(self.module.target_config().pointer_type(), local_id);
                        let name_ptr = self.builder.ins().iconst(
                            self.module.target_config().pointer_type(),
                            name.as_ptr() as i64,
                        );
                        let name_len = self.builder.ins().iconst(types::I64, name.len() as i64);
                        self.load_global_name(name_ptr, name_len, context);
                    } else {
                        panic!("Load requires a variable or constant argument");
                    }

                    //unimplemented!()
                }
                BytecodeOperator::Assign => {
                    if let ArgType::Local(v) = arg.get_arg() {
                        let variable = self.variables.get(v.1.as_str()).unwrap();
                        self.builder.def_var(*variable, context.exp.pop().unwrap());
                    } else {
                        panic!("not supported assign type: {:?}", arg.get_arg());
                    }
                }
                BytecodeOperator::BinaryAdd => {
                    if let (Some(right), Some(left)) = (context.exp.pop(), context.exp.pop()) {
                        // let result = self.builder.ins().iadd(left, right);
                        // context.left = Some(result);
                        let mut operator_name_sig = self.module.make_signature();
                        operator_name_sig
                            .params
                            .push(AbiParam::new(self.module.target_config().pointer_type())); // left value
                        operator_name_sig
                            .params
                            .push(AbiParam::new(self.module.target_config().pointer_type())); // right value
                        operator_name_sig.params.push(AbiParam::new(types::I32)); // operator type
                                                                                  // operator_name_sig
                                                                                  //     .params
                                                                                  //     .push(AbiParam::new(self.module.target_config().pointer_type())); // runtime context
                        operator_name_sig
                            .params
                            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
                        operator_name_sig
                            .returns
                            .push(AbiParam::new(self.module.target_config().pointer_type()));

                        //let builder = FunctionBuilder::new(&mut ctx.func, build_context);

                        let fn_id = self
                            .module
                            .declare_function(
                                "binary_op",
                                cranelift_module::Linkage::Import,
                                &operator_name_sig,
                            )
                            .unwrap();
                        let thread = self.builder.block_params(context.entry_block)[0];
                        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
                        let add_t = self
                            .builder
                            .ins()
                            .iconst(types::I32, BinaryOffset::Add as i64);
                        let call = self
                            .builder
                            .ins()
                            .call(func_ref, &[left, right, add_t, thread]);
                        let ret = self.builder.inst_results(call)[0];
                        context.exp.push(ret);
                    } else {
                        unimplemented!("BinaryAdd requires both left and right operands");
                    }
                }

                BytecodeOperator::AssignArgs => {}

                BytecodeOperator::EndFn => {
                    // let null_value = self.builder.ins().iconst(self.int, 0);
                    // self.builder.ins().return_(&[null_value]);
                }
                BytecodeOperator::WhileTest => {
                    self.load_while(context);
                }
                BytecodeOperator::WhileBlockEnd => {
                    self.load_while_end(context);
                }
                BytecodeOperator::Call => {
                    self.load_call(arg, context);
                }
                BytecodeOperator::CompareTest => {
                    self.load_compare(context, arg);
                }
                _ => {
                    unimplemented!("Compile operator: {:?}", arg.get_operator())
                }
            }
        }
    }
}

fn declare_variable(
    int: types::Type,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
    index: &mut usize,
    name: &str,
) -> Variable {
    let var = Variable::new(*index);
    if !variables.contains_key(name) {
        variables.insert(name.into(), var);
        builder.declare_var(var, int);
        *index += 1;
    }
    var
}

fn declare_variables(
    int: types::Type,
    builder: &mut FunctionBuilder,
    params: &[String],
    //the_return: &str,
    //stmts: &[Expr],
    entry_block: Block,
) -> HashMap<String, Variable> {
    let mut variables = HashMap::new();
    let mut index = 0;

    for (i, name) in params.iter().enumerate() {
        // TODO: cranelift_frontend should really have an API to make it easy to set
        // up param variables.
        let val = builder.block_params(entry_block)[i];
        let var = declare_variable(int, builder, &mut variables, &mut index, name);
        builder.def_var(var, val);
    }
    let zero = builder.ins().iconst(int, 0);
    // let return_variable = declare_variable(
    //     int,
    //     builder,
    //     &mut variables,
    //     &mut index,
    //     "retrun_xksdfjklsdjf",
    // );
    // builder.def_var(return_variable, zero);
    // for expr in stmts {
    //     declare_variables_in_stmt(int, builder, &mut variables, &mut index, expr);
    // }

    variables
}

impl CraneLiftJitBackend {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        builder.symbol("binary_op", binary_op as *const u8);
        builder.symbol("get_constant", get_constant as *const u8);
        builder.symbol("call_fn", call_fn as *const u8);
        builder.symbol("malloc", malloc as *const u8);
        builder.symbol("free", free as *const u8);
        builder.symbol("get_obj_by_name", get_obj_by_name as *const u8);
        builder.symbol("is_false", is_false as *const u8);
        builder.symbol("check_gc", check_gc as *const u8);
        builder.symbol("gc_collect", gc_collect as *const u8);
        builder.symbol("compare_test", compare_test as *const u8);

        let module = JITModule::new(builder);

        CraneLiftJitBackend {
            ctx: codegen::Context::new(),
            builder_context: FunctionBuilderContext::new(),
            //variable: HashMap::new(),
            module,
        }
    }

    pub fn compile(&mut self, code: &Bytecode) -> Result<*const u8, String> {
        let ptr = self.module.target_config().pointer_type();

        self.ctx.func.signature.params.push(AbiParam::new(ptr)); // Add a parameter for the thread runtime.
        self.ctx.func.signature.params.push(AbiParam::new(ptr)); // Add a parameter for the code object.
        self.ctx.func.signature.params.push(AbiParam::new(ptr)); // Add a parameter for list of arguments.
        self.ctx
            .func
            .signature
            .params
            .push(AbiParam::new(types::I32)); // Add a parameter for the number of arguments.
        self.ctx.func.signature.returns.push(AbiParam::new(ptr)); // Add a return type for the function.

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let variables = code
            .var_map
            .var_map
            .keys()
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        let entry_block = builder.create_block();

        let mut context = OperatorContext {
            exp: vec![],
            operator: "",
            while_blocks: vec![],
            exit_blocks: vec![],
            entry_block: entry_block.clone(),
        };
        // Since this is the entry block, add block parameters corresponding to
        // the function's parameters.
        //
        // TODO: Streamline the API here.
        builder.append_block_params_for_function_params(entry_block);

        // Tell the builder to emit code in this block.
        builder.switch_to_block(entry_block);

        // And, tell the builder that this block will have no further
        // predecessors. Since it's the entry block, it won't have any
        // predecessors.
        builder.seal_block(entry_block);
        let variables = declare_variables(ptr, &mut builder, &variables, entry_block);

        let mut trans = JitBuilder {
            int: ptr,
            builder,
            variables,
            module: &mut self.module,
        };

        for expr in &code.bytecode {
            trans.compile_expr(expr, &mut context);
            trans.load_check_gc(&mut context);
        }

        // let end_block = trans.builder.create_block();
        // trans.builder.seal_block(end_block);
        let null_value = trans.builder.ins().iconst(ptr, 0);
        trans.builder.ins().return_(&[null_value]);

        trans.builder.finalize();

        let fn_name = code.name.as_str();

        let id = self
            .module
            .declare_function(
                fn_name,
                cranelift_module::Linkage::Export,
                &self.ctx.func.signature,
            )
            .unwrap();
        println!("Cranelift JIT compiled function: {}", fn_name);
        //println!("{}", self.ctx.func.display());
        self.module.define_function(id, &mut self.ctx).unwrap();

        self.module.clear_context(&mut self.ctx);
        // Tell the builder we're done with this function.

        self.module.finalize_definitions().unwrap();

        // We can now retrieve a pointer to the machine code.
        let code = self.module.get_finalized_function(id);
        Ok(code)
    }
}

mod test {
    use crate::backend::{
        types::{
            base::{FSRObject, ObjId},
            code::FSRCode,
            module::FSRModule,
        },
        vm::{thread::FSRThreadRuntime, virtual_machine::FSRVM},
    };
}
