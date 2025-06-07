use std::{collections::HashMap, os::unix::thread};

use cranelift::{
    codegen,
    prelude::{
        settings, types, AbiParam, Block, Configurable, EntityRef, FunctionBuilder,
        FunctionBuilderContext, InstBuilder, Signature, Type, Value, Variable,
    },
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;

use crate::{
    backend::{
        compiler::{
            bytecode::{
                ArgType, BinaryOffset, Bytecode, BytecodeArg, BytecodeOperator, CompareOperator,
            },
            jit::jit_wrapper::{binary_dot_getter, clear_exp, get_current_fn_id, save_to_exp},
        },
        types::base::{FSRObject, ObjId},
        vm::thread::FSRThreadRuntime,
    },
    frontend::ast::token::{call, constant::FSROrinStr2, expr::SingleOp},
};

use super::jit_wrapper::{
    binary_op, binary_range, c_next_obj, call_fn, check_gc, compare_test, free, gc_collect,
    get_constant, get_iter_obj, get_n_args, get_obj_by_name, load_float, load_integer, load_string,
    malloc,
};

const ARGS_LEN: i64 = 512;
const CALL_ARGS_LEN: i64 = 16;

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
    constans: HashMap<u64, Variable>,
    defined_variables: HashMap<String, Variable>,
    module: &'a mut JITModule,
}

struct OperatorContext {
    exp: Vec<Value>,
    middle_value: Vec<Value>, // used to store intermediate values during operator processing, clear line
    operator: &'static str,
    loop_blocks: Vec<Block>,
    loop_exit_blocks: Vec<Block>,
    if_blocks: Vec<Block>,
    if_exit_blocks: Vec<(Block, bool)>,
    entry_block: Block,
    args_index: usize,
    ins_check_gc: bool,
    for_obj: Vec<Value>,
    for_iter_obj: Vec<Value>,
    logic_end_block: Option<Block>,
    logic_rest_bytecode_count: Option<usize>, // used to track the remaining bytecode count in a logic block
}

impl JitBuilder<'_> {
    fn load_constant(&mut self, c: u64, context: &mut OperatorContext) {
        let value = self.variables.get(&format!("{}_constant", c)).unwrap();
        let ret = self.builder.use_var(*value);
        context.exp.push(ret);
    }

    fn load_global_name(&mut self, name: Value, name_len: Value, context: &mut OperatorContext) {
        // pub extern "C" fn get_obj_by_name(name: *const u8, len: usize, thread: &mut FSRThreadRuntime) -> ObjId
        let mut get_obj_by_name_sig = self.module.make_signature();
        get_obj_by_name_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // name pointer
        get_obj_by_name_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // name length
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

    fn load_is_true(&mut self, context: &mut OperatorContext) -> Value {
        if let Some(value) = context.exp.last() {
            let true_id = self.builder.ins().iconst(
                self.module.target_config().pointer_type(),
                FSRObject::true_id() as i64,
            );
            let is_true =
                self.builder
                    .ins()
                    .icmp(codegen::ir::condcodes::IntCC::Equal, *value, true_id);
            // context.exp.push(is_true);
            return is_true;
        } else {
            panic!("IsTrue requires a value operand");
        }
    }

    fn load_is_not_false(&mut self, context: &mut OperatorContext) -> Value {
        if let Some(value) = context.exp.last() {
            let false_id = self.builder.ins().iconst(
                self.module.target_config().pointer_type(),
                FSRObject::false_id() as i64,
            );
            let is_not_false =
                self.builder
                    .ins()
                    .icmp(codegen::ir::condcodes::IntCC::NotEqual, *value, false_id);
            return is_not_false;
        } else {
            panic!("IsNotFalse requires a value operand");
        }
    }

    fn load_is_not_true(&mut self, context: &mut OperatorContext) -> Value {
        if let Some(value) = context.exp.last() {
            let true_id = self.builder.ins().iconst(
                self.module.target_config().pointer_type(),
                FSRObject::true_id() as i64,
            );
            let is_not_true =
                self.builder
                    .ins()
                    .icmp(codegen::ir::condcodes::IntCC::NotEqual, *value, true_id);
            return is_not_true;
        } else {
            panic!("IsNotTrue requires a value operand");
        }
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
            compare_test_sig.params.push(AbiParam::new(types::I32)); // compare operator type
            compare_test_sig
                .returns
                .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (boolean)
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
                self.builder.ins().iconst(types::I32, v)
            } else {
                panic!("CompareTest requires a CompareOperator argument")
            };
            let call = self
                .builder
                .ins()
                .call(func_ref, &[thread_runtime, left, right, op]);
            let result = self.builder.inst_results(call)[0];
            context.exp.push(result);
            context.middle_value.push(left);
            context.middle_value.push(right);
        } else {
            panic!("CompareTest requires both left and right operands");
        }
    }

    fn load_while(&mut self, context: &mut OperatorContext) {
        //let header_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        let exit_block = self.builder.create_block();
        let is_true = self.load_is_not_false(context);
        let condition = is_true;
        self.builder
            .ins()
            .brif(condition, body_block, &[], exit_block, &[]);

        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);
        context.loop_exit_blocks.push(exit_block);
        context.ins_check_gc = true;
    }

    fn load_while_end(&mut self, context: &mut OperatorContext) {
        self.builder
            .ins()
            .jump(context.loop_blocks.last().unwrap().clone(), &[]);

        //context.is_while = false;
        let v = context.loop_blocks.pop().unwrap();
        let exit_block = context.loop_exit_blocks.pop().unwrap();
        self.builder.seal_block(v);
        self.builder.switch_to_block(exit_block);
        self.builder.seal_block(exit_block);
        context.ins_check_gc = true;
        //self.builder.ins().iconst(self.int, 0);
    }

    fn load_for_iter(&mut self, context: &mut OperatorContext) {
        // pub extern "C" fn get_iter_obj(obj: ObjId, thread: &mut FSRThreadRuntime) -> ObjId {
        let mut get_iter_obj_sig = self.module.make_signature();
        get_iter_obj_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // object to iterate
        get_iter_obj_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
        get_iter_obj_sig
            .returns
            .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
        let fn_id = self
            .module
            .declare_function(
                "get_iter_obj",
                cranelift_module::Linkage::Import,
                &get_iter_obj_sig,
            )
            .unwrap();

        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let thread_runtime = self.builder.block_params(context.entry_block)[0];
        let for_obj = context.for_obj.pop().unwrap();
        let iter_obj = self
            .builder
            .ins()
            .call(func_ref, &[for_obj, thread_runtime]);
        let iter_obj_value = self.builder.inst_results(iter_obj)[0];

        let header_block = self.builder.create_block();
        // add param for header bloack
        let loop_var = self
            .builder
            .append_block_param(header_block, self.module.target_config().pointer_type());
        self.builder.ins().jump(header_block, &[iter_obj_value]);
        self.builder.switch_to_block(header_block);
        context.loop_blocks.push(header_block);
        context.for_iter_obj.push(iter_obj_value);

        // let header_block = self.builder.create_block();
        // self.builder.ins().jump(header_block, &[]);
        // self.builder.switch_to_block(header_block);
        // context.loop_blocks.push(header_block);
    }

    fn is_none(&mut self, value: Value, context: &mut OperatorContext) -> Value {
        self.load_none(context);
        let none_id = context.exp.pop().unwrap();

        self.builder
            .ins()
            .icmp(codegen::ir::condcodes::IntCC::NotEqual, value, none_id)
    }

    fn load_for_next(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        //pub extern "C" fn c_next_obj(obj: ObjId, thread: &mut FSRThreadRuntime) -> ObjId {
        let mut next_obj_sig = self.module.make_signature();
        next_obj_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // iterator object
        next_obj_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
        next_obj_sig
            .returns
            .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
        let fn_id = self
            .module
            .declare_function(
                "c_next_obj",
                cranelift_module::Linkage::Import,
                &next_obj_sig,
            )
            .unwrap();
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let thread_runtime = self.builder.block_params(context.entry_block)[0];
        let iter_obj = *context.for_iter_obj.last().unwrap();
        let next_obj = self
            .builder
            .ins()
            .call(func_ref, &[iter_obj, thread_runtime]);
        let next_obj_value = self.builder.inst_results(next_obj)[0];
        if let ArgType::Local((_, name, _)) = arg.get_arg() {
            let variable = self.variables.get(name).unwrap();
            self.builder.def_var(*variable, next_obj_value);
            self.defined_variables.insert(name.to_string(), *variable);

            let v = self.builder.use_var(*variable);
            let condition = self.is_none(v, context);
            let body_block = self.builder.create_block();
            let exit_block = self.builder.create_block();
            // let condition = context.exp.pop().unwrap();
            // self.builder
            //     .ins()
            //     .brif(condition, body_block, &[], exit_block, &[]);

            self.builder
                .ins()
                .brif(condition, body_block, &[], exit_block, &[]);
            self.builder.switch_to_block(body_block);
            self.builder.seal_block(body_block);
            context.loop_exit_blocks.push(exit_block);
            context.ins_check_gc = true;
        } else {
            panic!("ForNext requires a Local argument");
        }
    }

    fn load_for_end(&mut self, context: &mut OperatorContext) {
        self.builder.ins().jump(
            context.loop_blocks.last().unwrap().clone(),
            &[*context.for_iter_obj.last().unwrap()],
        );

        //context.is_while = false;
        let v = context.loop_blocks.pop().unwrap();
        let exit_block = context.loop_exit_blocks.pop().unwrap();
        self.builder.seal_block(v);
        self.builder.switch_to_block(exit_block);
        self.builder.seal_block(exit_block);
        context.for_iter_obj.pop();
        context.for_obj.pop();
        context.ins_check_gc = true;
        //self.builder.ins().iconst(self.int, 0);
    }

    fn load_if_test(&mut self, context: &mut OperatorContext) {
        //let header_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        let exit_block = self.builder.create_block();
        // let condition = context.exp.pop().unwrap();
        let is_true = self.load_is_not_false(context);
        let condition = is_true;
        self.builder
            .ins()
            .brif(condition, body_block, &[], exit_block, &[]);

        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);
        context.if_exit_blocks.push((exit_block, false));
        context.ins_check_gc = true;
    }

    fn load_if_end(&mut self, context: &mut OperatorContext) {
        if context.if_exit_blocks.last().unwrap().1 {
        } else {
            self.builder
                .ins()
                .jump(context.if_exit_blocks.last().unwrap().clone().0, &[]);
        }

        //self.builder.ins().nop();

        //context.is_while = false;
        let v = context.if_blocks.pop().unwrap();
        let exit_block = context.if_exit_blocks.pop().unwrap();
        self.builder.seal_block(v);
        self.builder.switch_to_block(exit_block.0);
        self.builder.seal_block(exit_block.0);
        context.ins_check_gc = true;
        //self.builder.ins().iconst(self.int, 0);
    }

    fn load_make_arg_list(&mut self, context: &mut OperatorContext, len: usize) -> Value {
        // let mut malloc_sig = self.module.make_signature();
        // malloc_sig
        //     .params
        //     .push(AbiParam::new(self.module.target_config().pointer_type())); // size
        // malloc_sig
        //     .returns
        //     .push(AbiParam::new(self.module.target_config().pointer_type())); // return type
        // let malloc_id = self
        //     .module
        //     .declare_function("malloc", cranelift_module::Linkage::Import, &malloc_sig)
        //     .unwrap();
        // let malloc_func_ref = self
        //     .module
        //     .declare_func_in_func(malloc_id, self.builder.func);

        let size = self
            .builder
            .ins()
            .iconst(self.module.target_config().pointer_type(), len as i64);
        // let malloc_call = self.builder.ins().call(malloc_func_ref, &[size]);
        // let malloc_ret = self.builder.inst_results(malloc_call)[0];
        let malloc_ret = self
            .builder
            .use_var(*self.variables.get("#call_args_ptr").unwrap());
        let mut rev_args = vec![];
        for i in 0..len {
            // Assuming we have a way to get the next argument value
            let arg_value = context.exp.pop().unwrap(); // This should be replaced with actual argument retrieval logic
            context.middle_value.push(arg_value);
            rev_args.push(arg_value);
        }

        rev_args.reverse();

        for (i, arg) in rev_args.into_iter().enumerate() {
            let offset = self
                .builder
                .ins()
                .iconst(types::I64, i as i64 * std::mem::size_of::<ObjId>() as i64); // Replace with actual offset calculation
            let ptr = self.builder.ins().iadd(malloc_ret, offset);
            self.builder
                .ins()
                .store(cranelift::codegen::ir::MemFlags::new(), arg, ptr, 0);
        }

        malloc_ret
    }

    fn load_make_middle_v_list_save(&mut self, context: &mut OperatorContext) -> Value {
        // let mut malloc_sig = self.module.make_signature();
        // malloc_sig
        //     .params
        //     .push(AbiParam::new(self.module.target_config().pointer_type())); // size
        // malloc_sig
        //     .returns
        //     .push(AbiParam::new(self.module.target_config().pointer_type())); // return type
        // let malloc_id = self
        //     .module
        //     .declare_function("malloc", cranelift_module::Linkage::Import, &malloc_sig)
        //     .unwrap();
        // let malloc_func_ref = self
        //     .module
        //     .declare_func_in_func(malloc_id, self.builder.func);

        // let size = self
        //     .builder
        //     .ins()
        //     .iconst(self.module.target_config().pointer_type(), context.middle_value.len() as i64);
        // let malloc_call = self.builder.ins().call(malloc_func_ref, &[size]);
        // let malloc_ret = self.builder.inst_results(malloc_call)[0];
        let mut malloc_ret = self
            .builder
            .use_var(*self.variables.get("#args_ptr").unwrap());
        let mut rev_args = vec![];
        for i in &context.middle_value {
            // Assuming we have a way to get the next argument value
            rev_args.push(*i);
        }

        rev_args.reverse();

        for (i, arg) in rev_args.into_iter().enumerate() {
            let offset = self
                .builder
                .ins()
                .iconst(types::I64, i as i64 * std::mem::size_of::<ObjId>() as i64); // Replace with actual offset calculation
            let ptr = self.builder.ins().iadd(malloc_ret, offset);
            self.builder
                .ins()
                .store(cranelift::codegen::ir::MemFlags::new(), arg, ptr, 0);
        }

        malloc_ret
    }

    fn load_free_arg_list(&mut self, list_ptr: Value, context: &mut OperatorContext, len: i64) {
        // pub extern "C" fn free(ptr: *mut Vec<ObjId>, size: usize)
        let mut free_sig = self.module.make_signature();
        free_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // pointer to the list
        free_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // size of the list
                                                                              //free_sig.returns.push(AbiParam::new(types::I32)); // return type (void)
        let free_id = self
            .module
            .declare_function("free", cranelift_module::Linkage::Import, &free_sig)
            .unwrap();
        let free_func_ref = self.module.declare_func_in_func(free_id, self.builder.func);
        let size = self
            .builder
            .ins()
            .iconst(self.module.target_config().pointer_type(), len);
        let free_call = self.builder.ins().call(free_func_ref, &[list_ptr, size]);
        let _ = self.builder.inst_results(free_call); // We don't need the return value, just ensure the call is made
    }

    fn load_gc_collect(&mut self, context: &mut OperatorContext) {
        let ptr_type = self.module.target_config().pointer_type();
        let var_count =
            self.defined_variables.len() + context.for_iter_obj.len() + context.for_obj.len();
        let size = self.builder.ins().iconst(ptr_type, var_count as i64); // usize

        // let mut malloc_sig = self.module.make_signature();
        // malloc_sig.params.push(AbiParam::new(types::I64));
        // malloc_sig.returns.push(AbiParam::new(ptr_type));
        // let malloc_id = self
        //     .module
        //     .declare_function("malloc", cranelift_module::Linkage::Import, &malloc_sig)
        //     .unwrap();
        // let malloc_func_ref = self
        //     .module
        //     .declare_func_in_func(malloc_id, self.builder.func);
        // let malloc_call = self.builder.ins().call(malloc_func_ref, &[size]);
        // let arr_ptr = self.builder.inst_results(malloc_call)[0];
        let arr_ptr = self
            .builder
            .use_var(*self.variables.get("#args_ptr").unwrap());
        let mut i = 0;
        for var in self.defined_variables.values() {
            let value = self.builder.use_var(*var);
            let offset = self
                .builder
                .ins()
                .iconst(types::I64, (i * std::mem::size_of::<ObjId>()) as i64);
            let ptr = self.builder.ins().iadd(arr_ptr, offset);
            self.builder
                .ins()
                .store(cranelift::codegen::ir::MemFlags::new(), value, ptr, 0);
            i += 1;
        }

        for var in &context.for_iter_obj {
            let value = *var;
            let offset = self
                .builder
                .ins()
                .iconst(types::I64, (i * std::mem::size_of::<ObjId>()) as i64);
            let ptr = self.builder.ins().iadd(arr_ptr, offset);
            self.builder
                .ins()
                .store(cranelift::codegen::ir::MemFlags::new(), value, ptr, 0);
            i += 1;
        }

        for var in &context.for_obj {
            let value = *var;
            let offset = self
                .builder
                .ins()
                .iconst(types::I64, (i * std::mem::size_of::<ObjId>()) as i64);
            let ptr = self.builder.ins().iadd(arr_ptr, offset);
            self.builder
                .ins()
                .store(cranelift::codegen::ir::MemFlags::new(), value, ptr, 0);
            i += 1;
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
                                                       //self.load_free_arg_list(arr_ptr, context, var_count as i64);
    }

    fn load_check_gc(&mut self, context: &mut OperatorContext) -> Value {
        let mut check_gc_sig = self.module.make_signature();
        check_gc_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
        check_gc_sig.returns.push(AbiParam::new(types::I8)); // return type (boolean)

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

    fn make_call_fn(&self) -> Signature {
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
        call_fn_sig
    }

    fn load_call(&mut self, arg: &BytecodeArg, context: &mut OperatorContext) {
        if let ArgType::CallArgsNumber(v) = arg.get_arg() {
            //let variable = self.variables.get(v.2.as_str()).unwrap();
            // context.left = Some(self.builder.use_var(*variable));
            //let fn_obj_id = self.builder.use_var(*variable);

            // call_fn(args: *const ObjId, len: usize, fn_id: ObjId, thread: &mut FSRThreadRuntime, code: ObjId) -> ObjId

            let call_fn_sig = self.make_call_fn();
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
            self.save_middle_value(context);
            self.save_object_to_exp(context);
            let call = self.builder.ins().call(
                func_ref,
                &[list_ptr, len, fn_obj_id, thread_runtime, code_object],
            );
            //self.clear_middle_value(context);

            let ret = self.builder.inst_results(call)[0];

            // Free the argument list after the call
            //self.load_free_arg_list(list_ptr, context, *v as i64);

            context.exp.push(ret);
            context.middle_value.push(ret);
        } else {
            unimplemented!()
        }
    }

    fn load_none(&mut self, context: &mut OperatorContext) {
        // pub extern "C" fn get_none() -> ObjId
        // let mut get_none_sig = self.module.make_signature();
        // get_none_sig
        //     .returns
        //     .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
        // let fn_id = self
        //     .module
        //     .declare_function("get_none", cranelift_module::Linkage::Import, &get_none_sig)
        //     .unwrap();
        // let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        // let call = self.builder.ins().call(func_ref, &[]);
        let none_id = FSRObject::none_id();
        let none_value = self
            .builder
            .ins()
            .iconst(self.module.target_config().pointer_type(), none_id as i64);
        // let ret = self.builder.inst_results(call)[0];
        context.exp.push(none_value);
    }

    fn make_binary_op(&self) -> Signature {
        let mut binary_op_sig = self.module.make_signature();
        binary_op_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // left value
        binary_op_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // right value
        binary_op_sig.params.push(AbiParam::new(types::I32)); // operator type
                                                              // operator_name_sig
                                                              //     .params
                                                              //     .push(AbiParam::new(self.module.target_config().pointer_type())); // runtime context
                                                              // code: ObjId,
        binary_op_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // code object
        binary_op_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
        binary_op_sig
            .returns
            .push(AbiParam::new(self.module.target_config().pointer_type()));
        binary_op_sig
    }

    fn load_binary_op(&mut self, context: &mut OperatorContext, op: BinaryOffset) {
        if let (Some(right), Some(left)) = (context.exp.pop(), context.exp.pop()) {
            self.save_middle_value(context);
            self.save_object_to_exp(context);
            let binary_op_sig = self.make_binary_op();

            let fn_id = self
                .module
                .declare_function(
                    "binary_op",
                    cranelift_module::Linkage::Import,
                    &binary_op_sig,
                )
                .unwrap();
            let thread = self.builder.block_params(context.entry_block)[0];
            let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
            let add_t = self.builder.ins().iconst(types::I32, op as i64);
            let code_object = self.builder.block_params(context.entry_block)[1];
            let call = self
                .builder
                .ins()
                .call(func_ref, &[left, right, add_t, code_object, thread]);
            let ret = self.builder.inst_results(call)[0];
            context.exp.push(ret);
            context.middle_value.push(ret);
            context.middle_value.push(right);
            context.middle_value.push(left);
            // self.clear_middle_value(context);
        } else {
            unimplemented!("BinaryAdd requires both left and right operands");
        }
    }

    fn save_object_to_exp(&mut self, context: &mut OperatorContext) {
        let arr_ptr = self
            .builder
            .use_var(*self.variables.get("#args_ptr").unwrap());
        let mut i = 0;
        for var in self.defined_variables.values() {
            let value = self.builder.use_var(*var);
            let offset = self
                .builder
                .ins()
                .iconst(types::I64, (i * std::mem::size_of::<ObjId>()) as i64);
            let ptr = self.builder.ins().iadd(arr_ptr, offset);
            self.builder
                .ins()
                .store(cranelift::codegen::ir::MemFlags::new(), value, ptr, 0);
            i += 1;
        }

        for var in &context.for_iter_obj {
            let value = *var;
            let offset = self
                .builder
                .ins()
                .iconst(types::I64, (i * std::mem::size_of::<ObjId>()) as i64);
            let ptr = self.builder.ins().iadd(arr_ptr, offset);
            self.builder
                .ins()
                .store(cranelift::codegen::ir::MemFlags::new(), value, ptr, 0);
            i += 1;
        }

        for var in &context.for_obj {
            let value = *var;
            let offset = self
                .builder
                .ins()
                .iconst(types::I64, (i * std::mem::size_of::<ObjId>()) as i64);
            let ptr = self.builder.ins().iadd(arr_ptr, offset);
            self.builder
                .ins()
                .store(cranelift::codegen::ir::MemFlags::new(), value, ptr, 0);
            i += 1;
        }

        let mut save_to_exp_sig = self.module.make_signature();
        save_to_exp_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // args pointer
        save_to_exp_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // length of the args
        save_to_exp_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime

        let fn_id = self
            .module
            .declare_function(
                "save_to_exp",
                cranelift_module::Linkage::Import,
                &save_to_exp_sig,
            )
            .unwrap();
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let len = self.builder.ins().iconst(types::I64, i as i64);
        let thread_runtime = self.builder.block_params(context.entry_block)[0];
        let call = self
            .builder
            .ins()
            .call(func_ref, &[arr_ptr, len, thread_runtime]);

        // self.load_free_arg_list(arr_ptr, context, i as i64);
    }

    fn save_middle_value(&mut self, context: &mut OperatorContext) {
        let mut save_to_exp_sig = self.module.make_signature();
        save_to_exp_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // args pointer
        save_to_exp_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // length of the args
        save_to_exp_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime

        let fn_id = self
            .module
            .declare_function(
                "save_to_exp",
                cranelift_module::Linkage::Import,
                &save_to_exp_sig,
            )
            .unwrap();
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let list_ptr = self.load_make_middle_v_list_save(context);
        let len = self
            .builder
            .ins()
            .iconst(types::I64, context.middle_value.len() as i64);
        let thread_runtime = self.builder.block_params(context.entry_block)[0];
        let call = self
            .builder
            .ins()
            .call(func_ref, &[list_ptr, len, thread_runtime]);

        //self.load_free_arg_list(list_ptr, context, context.middle_value.len() as i64);
    }

    fn clear_middle_value(&mut self, context: &mut OperatorContext) {
        // pub extern "C" fn clear_exp(thread: &mut FSRThreadRuntime)
        let mut clear_exp_sig = self.module.make_signature();
        clear_exp_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime

        let fn_id = self
            .module
            .declare_function(
                "clear_exp",
                cranelift_module::Linkage::Import,
                &clear_exp_sig,
            )
            .unwrap();
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let thread_runtime = self.builder.block_params(context.entry_block)[0];
        let call = self.builder.ins().call(func_ref, &[thread_runtime]);
        let _ = self.builder.inst_results(call); // We don't need the return value, just ensure the call is made
    }

    fn load_binary_range(&mut self, context: &mut OperatorContext) {
        if let (Some(right), Some(left)) = (context.exp.pop(), context.exp.pop()) {
            // pub extern "C" fn binary_range(left: ObjId, right: ObjId, thread: &mut FSRThreadRuntime) -> ObjId
            let mut binary_range_sig = self.module.make_signature();
            binary_range_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // left operand
            binary_range_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // right operand
            binary_range_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
            binary_range_sig
                .returns
                .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
            let fn_id = self
                .module
                .declare_function(
                    "binary_range",
                    cranelift_module::Linkage::Import,
                    &binary_range_sig,
                )
                .unwrap();
            let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
            let thread_runtime = self.builder.block_params(context.entry_block)[0];
            let call = self
                .builder
                .ins()
                .call(func_ref, &[left, right, thread_runtime]);
            let ret = self.builder.inst_results(call)[0];
            context.exp.push(ret);
            context.middle_value.push(ret);
            context.middle_value.push(right);
            context.middle_value.push(left);
        } else {
            panic!("BinaryRange requires both left and right operands");
        }
    }

    fn load_args(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        let index = context.args_index;
        // pub extern "C" fn get_n_args(thread: &mut FSRThreadRuntime, index: i32) -> ObjId
        let mut get_n_args_sig = self.module.make_signature();
        get_n_args_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
        get_n_args_sig.params.push(AbiParam::new(types::I32)); // index of the argument
        get_n_args_sig
            .returns
            .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
        let fn_id = self
            .module
            .declare_function(
                "get_n_args",
                cranelift_module::Linkage::Import,
                &get_n_args_sig,
            )
            .unwrap();
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let thread_runtime = self.builder.block_params(context.entry_block)[0];
        let index_value = self.builder.ins().iconst(types::I32, index as i64);
        let call = self
            .builder
            .ins()
            .call(func_ref, &[thread_runtime, index_value]);
        let ret = self.builder.inst_results(call)[0];

        if let ArgType::Local(v) = arg.get_arg() {
            let variable = self.variables.get(v.1.as_str()).unwrap();
            self.builder.def_var(*variable, ret);
            self.defined_variables.insert(v.1.to_string(), *variable);
        } else {
            panic!("GetArgs requires a Local argument");
        }

        context.args_index += 1;
    }

    fn load_or_jump(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        // process or logic like a or b
        //let last_ssa_value = *context.exp.last().unwrap();
        let last_ssa_value = self.load_is_not_false(context);
        //let last_ssa_value = context.exp.pop().unwrap();
        let b_block = self.builder.create_block();
        let end_block = self.builder.create_block();
        // add param for end block
        self.builder
            .append_block_param(end_block, self.module.target_config().pointer_type());
        self.builder.ins().brif(
            last_ssa_value,
            end_block,
            &[*context.exp.last().unwrap()],
            b_block,
            &[],
        );

        self.builder.switch_to_block(b_block);
        self.builder.seal_block(b_block);
        context.logic_end_block = Some(end_block);
        if let ArgType::AddOffset(offset) = arg.get_arg() {
            context.logic_rest_bytecode_count = Some(*offset);
        } else {
            panic!("OrJump requires an AddOffset argument");
        }
    }

    fn load_and_jump(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        //let last_ssa_value = *context.exp.last().unwrap();
        let last_ssa_value = self.load_is_not_true(context);
        //let last_ssa_value = context.exp.pop().unwrap();
        let b_block = self.builder.create_block();
        let end_block = self.builder.create_block();
        // add param for end block
        self.builder
            .append_block_param(end_block, self.module.target_config().pointer_type());
        self.builder.ins().brif(
            last_ssa_value,
            end_block,
            &[*context.exp.last().unwrap()],
            b_block,
            &[],
        );

        self.builder.switch_to_block(b_block);
        self.builder.seal_block(b_block);
        context.logic_end_block = Some(end_block);
        if let ArgType::AddOffset(offset) = arg.get_arg() {
            context.logic_rest_bytecode_count = Some(*offset);
        } else {
            panic!("OrJump requires an AddOffset argument");
        }
    }

    fn load_init_integer(&mut self, arg: &BytecodeArg, context: &mut OperatorContext) {
        // pub extern "C" fn load_integer(
        //     value: i64,
        //     thread: &mut FSRThreadRuntime,
        // ) -> ObjId {
        if let ArgType::ConstInteger(id, s, op) = arg.get_arg() {
            let mut load_integer_sig = self.module.make_signature();
            load_integer_sig.params.push(AbiParam::new(types::I64)); // value
            load_integer_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
            load_integer_sig
                .returns
                .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
            let fn_id = self
                .module
                .declare_function(
                    "load_integer",
                    cranelift_module::Linkage::Import,
                    &load_integer_sig,
                )
                .unwrap();
            let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
            let v = match op {
                Some(SingleOp::Minus) => -s.parse::<i64>().unwrap(),
                None => s.parse::<i64>().unwrap(),
                _ => panic!("Unsupported single operation for constant integer"),
            };

            let value = self.builder.ins().iconst(types::I64, v);
            let thread_runtime = self.builder.block_params(context.entry_block)[0];
            let call = self.builder.ins().call(func_ref, &[value, thread_runtime]);
            let ret = self.builder.inst_results(call)[0];

            let name = format!("{}_constant", id);

            let variable = self.variables.get(&name).unwrap();
            self.builder.def_var(*variable, ret);
            self.defined_variables.insert(name.to_string(), *variable);
        }
    }

    fn load_init_float(&mut self, arg: &BytecodeArg, context: &mut OperatorContext) {
        if let ArgType::ConstFloat(id, f, op) = arg.get_arg() {
            // pub extern "C" fn load_float(
            //     value: f64,
            //     thread: &mut FSRThreadRuntime,
            // ) -> ObjId {
            let mut load_float_sig = self.module.make_signature();
            load_float_sig.params.push(AbiParam::new(types::F64)); // value
            load_float_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
            load_float_sig
                .returns
                .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
            let fn_id = self
                .module
                .declare_function(
                    "load_float",
                    cranelift_module::Linkage::Import,
                    &load_float_sig,
                )
                .unwrap();
            let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
            let v = match op {
                Some(SingleOp::Minus) => -f.parse::<f64>().unwrap(),
                None => f.parse::<f64>().unwrap(),
                _ => panic!("Unsupported single operation for constant float"),
            };

            let value = self.builder.ins().f64const(v);
            let thread_runtime = self.builder.block_params(context.entry_block)[0];
            let call = self.builder.ins().call(func_ref, &[value, thread_runtime]);
            let ret = self.builder.inst_results(call)[0];

            let name = format!("{}_constant", id);

            let variable = self.variables.get(&name).unwrap();
            self.builder.def_var(*variable, ret);
            self.defined_variables.insert(name.to_string(), *variable);
        }
    }

    fn load_init_string(&mut self, arg: &BytecodeArg, context: &mut OperatorContext) {
        if let ArgType::ConstString(id, s) = arg.get_arg() {
            // pub extern "C" fn load_string(
            //     value: *const u8,
            //     len: usize,
            //     thread: &mut FSRThreadRuntime,
            // ) -> ObjId {
            let mut load_string_sig = self.module.make_signature();
            load_string_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // value pointer
            load_string_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // length of the string
            load_string_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
            load_string_sig
                .returns
                .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
            let fn_id = self
                .module
                .declare_function(
                    "load_string",
                    cranelift_module::Linkage::Import,
                    &load_string_sig,
                )
                .unwrap();
            let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
            let value_ptr = self.builder.ins().iconst(
                self.module.target_config().pointer_type(),
                s.as_ptr() as i64,
            );
            let value_len = self
                .builder
                .ins()
                .iconst(self.module.target_config().pointer_type(), s.len() as i64);
            let thread_runtime = self.builder.block_params(context.entry_block)[0];
            let call = self
                .builder
                .ins()
                .call(func_ref, &[value_ptr, value_len, thread_runtime]);
            let ret = self.builder.inst_results(call)[0];
            let name = format!("{}_constant", id);
            let variable = self.variables.get(&name).unwrap();
            self.builder.def_var(*variable, ret);
            self.defined_variables.insert(name.to_string(), *variable);
        }
    }

    fn load_init_constants(&mut self, arg: &BytecodeArg, context: &mut OperatorContext) {
        if let ArgType::ConstInteger(id, s, op) = arg.get_arg() {
            self.load_init_integer(arg, context);
        } else if let ArgType::ConstFloat(id, f, op) = arg.get_arg() {
            self.load_init_float(arg, context);
        } else if let ArgType::ConstString(id, s) = arg.get_arg() {
            self.load_init_string(arg, context);
        }
    }

    fn get_current_fn_id(&mut self, context: &mut OperatorContext) -> Value {
        // pub extern "C" fn get_current_fn_id(thread: &mut FSRThreadRuntime) -> ObjId
        let mut get_current_fn_sig = self.module.make_signature();
        get_current_fn_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
        get_current_fn_sig
            .returns
            .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
        let fn_id = self
            .module
            .declare_function(
                "get_current_fn_id",
                cranelift_module::Linkage::Import,
                &get_current_fn_sig,
            )
            .unwrap();
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let thread_runtime = self.builder.block_params(context.entry_block)[0];
        let call = self.builder.ins().call(func_ref, &[thread_runtime]);
        self.builder.inst_results(call)[0]
    }

    fn binary_dot_process(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        if let ArgType::Attr(id, name) = arg.get_arg() {
            // pub extern "C" fn binary_dot_getter(
            //     father: ObjId,
            //     name: *const u8,
            //     len: usize,
            //     thread: &mut FSRThreadRuntime,
            // ) -> ObjId {
            let mut binary_dot_sig = self.module.make_signature();
            binary_dot_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // father object
            binary_dot_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // name pointer
            binary_dot_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // name length
            binary_dot_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime

            binary_dot_sig
                .returns
                .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
            let fn_id = self
                .module
                .declare_function(
                    "binary_dot_getter",
                    cranelift_module::Linkage::Import,
                    &binary_dot_sig,
                )
                .unwrap();
            let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
            let father = context.exp.pop().unwrap();
            let name_ptr = self.builder.ins().iconst(
                self.module.target_config().pointer_type(),
                name.as_ptr() as i64,
            );
            let name_len = self.builder.ins().iconst(
                self.module.target_config().pointer_type(),
                name.len() as i64,
            );
            let thread_runtime = self.builder.block_params(context.entry_block)[0];
            let call = self
                .builder
                .ins()
                .call(func_ref, &[father, name_ptr, name_len, thread_runtime]);
            let ret = self.builder.inst_results(call)[0];
            context.exp.push(ret);
        } else {
            panic!("BinaryDot requires an Attr argument");
        }
    }

    fn malloc_args(&mut self, context: &mut OperatorContext) {
        let mut malloc_sig = self.module.make_signature();
        malloc_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // size
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

        let size = self
            .builder
            .ins()
            .iconst(self.module.target_config().pointer_type(), ARGS_LEN);
        let malloc_call = self.builder.ins().call(malloc_func_ref, &[size]);
        let malloc_ret = self.builder.inst_results(malloc_call)[0];

        let var = self.variables.get("#args_ptr").unwrap();
        self.builder.def_var(*var, malloc_ret);
    }

    fn malloc_call_args(&mut self, context: &mut OperatorContext) {
        let mut malloc_sig = self.module.make_signature();
        malloc_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // size
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

        let size = self
            .builder
            .ins()
            .iconst(self.module.target_config().pointer_type(), CALL_ARGS_LEN);
        let malloc_call = self.builder.ins().call(malloc_func_ref, &[size]);
        let malloc_ret = self.builder.inst_results(malloc_call)[0];

        let var = self.variables.get("#call_args_ptr").unwrap();
        self.builder.def_var(*var, malloc_ret);
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
            context.loop_blocks.push(header_block);
        }

        if expr.last().unwrap().get_operator() == &BytecodeOperator::IfTest {
            //context.is_if += 1;
            let header_block = self.builder.create_block();
            self.builder.ins().jump(header_block, &[]);
            self.builder.switch_to_block(header_block);
            context.if_blocks.push(header_block);
        }

        if expr.last().unwrap().get_operator() == &BytecodeOperator::LoadForIter {
            //context.is_for += 1;
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
                        let name_len = self.builder.ins().iconst(
                            self.module.target_config().pointer_type(),
                            name.len() as i64,
                        );
                        self.load_global_name(name_ptr, name_len, context);
                    } else if let ArgType::LoadTrue = arg.get_arg() {
                        let true_id = FSRObject::true_id();
                        let true_value = self
                            .builder
                            .ins()
                            .iconst(self.module.target_config().pointer_type(), true_id as i64);
                        context.exp.push(true_value);
                    } else if let ArgType::LoadFalse = arg.get_arg() {
                        let false_id = FSRObject::false_id();
                        let false_value = self
                            .builder
                            .ins()
                            .iconst(self.module.target_config().pointer_type(), false_id as i64);
                        context.exp.push(false_value);
                    } else if let ArgType::LoadNone = arg.get_arg() {
                        self.load_none(context);
                    } else if let ArgType::CurrentFn = arg.get_arg() {
                        let fn_id = self.get_current_fn_id(context);
                        context.exp.push(fn_id);
                    } else if let ArgType::ClosureVar(v) = arg.get_arg() {
                        // same as load global for now
                        let name_ptr = self.builder.ins().iconst(
                            self.module.target_config().pointer_type(),
                            v.1.as_ptr() as i64,
                        );
                        let name_len = self.builder.ins().iconst(
                            self.module.target_config().pointer_type(),
                            v.1.len() as i64,
                        );
                        self.load_global_name(name_ptr, name_len, context);
                    }
                    else {
                        panic!("Load requires a variable or constant argument");
                    }

                    //unimplemented!()
                }
                BytecodeOperator::Assign => {
                    if let ArgType::Local(v) = arg.get_arg() {
                        let variable = self.variables.get(v.1.as_str()).unwrap();
                        let var = context.exp.pop().unwrap();
                        context.middle_value.push(var);
                        self.builder.def_var(*variable, var);
                        self.defined_variables.insert(v.1.to_string(), *variable);
                    } else {
                        panic!("not supported assign type: {:?}", arg.get_arg());
                    }
                }
                BytecodeOperator::BinaryAdd => {
                    self.load_binary_op(context, BinaryOffset::Add);
                }
                BytecodeOperator::BinarySub => {
                    self.load_binary_op(context, BinaryOffset::Sub);
                }
                BytecodeOperator::BinaryMul => {
                    self.load_binary_op(context, BinaryOffset::Mul);
                }
                BytecodeOperator::BinaryDiv => {
                    self.load_binary_op(context, BinaryOffset::Div);
                }
                BytecodeOperator::BinaryReminder => {
                    self.load_binary_op(context, BinaryOffset::Reminder);
                }
                BytecodeOperator::AssignArgs => {
                    self.load_args(context, arg);
                    context.ins_check_gc = true;
                }

                BytecodeOperator::EndFn => {
                    // let null_value = self.builder.ins().iconst(self.int, 0);
                    // self.builder.ins().return_(&[null_value]);
                    // let end_bloack = self.builder.create_block();
                    // self.builder.switch_to_block(end_bloack);
                    // self.builder.seal_block(end_bloack);
                    // let ptr = self.module.target_config().pointer_type();
                    // let null_value = self.builder.ins().iconst(ptr, 0);

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
                BytecodeOperator::Empty => {
                    // Do nothing for empty operators
                    //self.builder.ins().nop();
                }
                BytecodeOperator::ReturnValue => {
                    let args_ptr = self
                        .builder
                        .use_var(*self.variables.get("#args_ptr").unwrap());
                    let call_args_ptr = self
                        .builder
                        .use_var(*self.variables.get("#call_args_ptr").unwrap());
                    self.load_free_arg_list(args_ptr, context, ARGS_LEN);
                    self.load_free_arg_list(call_args_ptr, context, CALL_ARGS_LEN);
                    if let Some(s) = context.if_exit_blocks.last_mut() {
                        s.1 = true; // Mark the last if block as having a return value
                    }
                    if let Some(value) = context.exp.pop() {
                        context.middle_value.push(value);
                        self.builder.ins().return_(&[value]);
                    } else {
                        self.load_none(context);
                        let value = context.exp.pop().unwrap();
                        self.builder.ins().return_(&[value]);
                    }
                }
                BytecodeOperator::IfTest => {
                    self.load_if_test(context);
                }
                BytecodeOperator::IfBlockEnd => {
                    self.load_if_end(context);
                }
                BytecodeOperator::LoadConst => {
                    self.load_init_constants(arg, context);
                }
                BytecodeOperator::ForBlockRefAdd => {
                    context.for_obj.push(context.exp.pop().unwrap());
                }
                BytecodeOperator::SpecialLoadFor => {
                    self.load_for_next(context, arg);
                }
                BytecodeOperator::LoadForIter => {
                    self.load_for_iter(context);
                }
                BytecodeOperator::ForBlockEnd => {
                    self.load_for_end(context);
                }
                BytecodeOperator::BinaryRange => {
                    self.load_binary_range(context);
                }
                BytecodeOperator::OrJump => {
                    self.load_or_jump(context, arg);
                }
                BytecodeOperator::AndJump => {
                    self.load_and_jump(context, arg);
                }
                BytecodeOperator::BinaryDot => {
                    self.binary_dot_process(context, arg);
                }
                _ => {
                    unimplemented!("Compile operator: {:?} not support now", arg.get_operator())
                }
            }

            if let Some(s) = &mut context.logic_rest_bytecode_count {
                if *s == 0 {
                    if let Some(end_block) = context.logic_end_block.take() {
                        self.builder
                            .ins()
                            .jump(end_block, &[context.exp.pop().unwrap()]);
                        self.builder.switch_to_block(end_block);
                        self.builder.seal_block(end_block);
                        context.exp.push(self.builder.block_params(end_block)[0]);
                    }
                    context.logic_rest_bytecode_count = None;
                }
            }

            if let Some(s) = &mut context.logic_rest_bytecode_count {
                *s -= 1;
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
    module: &JITModule,
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
        // let val = builder.block_params(entry_block)[i];
        let val = builder
            .ins()
            .iconst(module.target_config().pointer_type(), 0);
        let var = declare_variable(int, builder, &mut variables, &mut index, name);
        builder.def_var(var, val);
    }

    // for c in constans {
    //     let val = builder.block_params(entry_block)[i];
    //     let var = declare_variable(
    //         int,
    //         builder,
    //         &mut variables,
    //         &mut index,
    //         &format!("{}_constant", c),
    //     );
    //     builder.declare_var(var, int);
    // }

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

fn declare_constants(
    int: types::Type,
    builder: &mut FunctionBuilder,
    constants: &HashMap<u64, Variable>,
) -> HashMap<u64, Variable> {
    let mut constans = HashMap::new();
    for (c, variable) in constants {
        if !constans.contains_key(c) {
            constans.insert(*c, *variable);
            builder.declare_var(*variable, int);
        }
    }
    constans
}

impl CraneLiftJitBackend {
    fn init_builder(builder: &mut JITBuilder) {
        builder.symbol("binary_op", binary_op as *const u8);
        builder.symbol("get_constant", get_constant as *const u8);
        builder.symbol("call_fn", call_fn as *const u8);
        builder.symbol("malloc", malloc as *const u8);
        builder.symbol("free", free as *const u8);
        builder.symbol("get_obj_by_name", get_obj_by_name as *const u8);
        builder.symbol("check_gc", check_gc as *const u8);
        builder.symbol("gc_collect", gc_collect as *const u8);
        builder.symbol("compare_test", compare_test as *const u8);
        builder.symbol("get_n_args", get_n_args as *const u8);
        builder.symbol("load_integer", load_integer as *const u8);
        builder.symbol("load_string", load_string as *const u8);
        builder.symbol("load_float", load_float as *const u8);
        builder.symbol("get_iter_obj", get_iter_obj as *const u8);
        builder.symbol("c_next_obj", c_next_obj as *const u8);
        builder.symbol("binary_range", binary_range as *const u8);
        builder.symbol("get_current_fn_id", get_current_fn_id as *const u8);
        builder.symbol("save_to_exp", save_to_exp as *const u8);
        builder.symbol("clear_exp", clear_exp as *const u8);
        builder.symbol("binary_dot_getter", binary_dot_getter as *const u8);
    }

    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        //flag_builder.set("opt_level", "speed").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        Self::init_builder(&mut builder);

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
        let mut variables = code
            .var_map
            .var_map
            .keys()
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        let constans = code
            .var_map
            .const_map
            .values()
            .map(|x| format!("{}_constant", x))
            .collect::<Vec<_>>();

        let args_ptr = "#args_ptr";
        let call_args_ptr = "#call_args_ptr";
        variables.extend(constans);
        variables.push(args_ptr.to_string());
        variables.push(call_args_ptr.to_string());

        let entry_block = builder.create_block();

        let mut context = OperatorContext {
            exp: vec![],
            operator: "",
            loop_blocks: vec![],
            loop_exit_blocks: vec![],
            entry_block: entry_block.clone(),
            if_blocks: vec![],
            if_exit_blocks: vec![],
            args_index: 0,
            ins_check_gc: false,
            for_obj: vec![],
            for_iter_obj: vec![],
            logic_end_block: None,
            logic_rest_bytecode_count: None,
            middle_value: vec![],
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
        let variables = declare_variables(&self.module, ptr, &mut builder, &variables, entry_block);

        let mut trans = JitBuilder {
            int: ptr,
            builder,
            variables,
            module: &mut self.module,
            defined_variables: HashMap::new(),
            constans: HashMap::new(),
        };
        let mut i = 0;

        trans.malloc_args(&mut context);
        trans.malloc_call_args(&mut context);
        for expr in &code.bytecode {
            if i % 20 == 0 || context.ins_check_gc {
                trans.load_check_gc(&mut context);
                context.ins_check_gc = false;
            }

            trans.compile_expr(expr, &mut context);
            context.exp.clear();
            context.middle_value.clear();

            i += 1;
        }

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
        println!("{}", self.ctx.func.display());
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
