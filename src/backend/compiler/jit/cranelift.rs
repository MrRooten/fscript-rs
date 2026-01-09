use std::{collections::HashMap, os::unix::thread, sync::Arc};

use anyhow::{Context, Ok, Result};
use cranelift::{
    codegen::{self, ir},
    prelude::{
        AbiParam, Block, Configurable, EntityRef, FunctionBuilder, FunctionBuilderContext, InstBuilder, Signature, StackSlotData, StackSlotKind, Type, Value, Variable, settings, types
    },
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;

use crate::{
    backend::{
        compiler::{
            bytecode::{
                ArgType, BinaryOffset, Bytecode, BytecodeArg, BytecodeOperator, CompareOperator,
                FSRSType, FnCallSig, OpAssign,
            },
            jit::jit_wrapper::{
                binary_dot_getter, c_println, clear_exp, get_current_fn_id, get_obj_method,
                load_list, save_to_exp,
            },
        },
        types::base::{FSRObject, ObjId},
        vm::thread::FSRThreadRuntime,
    },
    frontend::ast::token::{call, constant::FSROrinStr2, expr::SingleOp, variable},
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
    var_index: usize,
    self_call_sig: Arc<FnCallSig>,
}

struct OperatorContext {
    exp: Vec<Value>,
    middle_value: Vec<Value>, // used to store intermediate values during operator processing, clear line
    operator: &'static str,
    loop_blocks: Vec<Block>,
    loop_exit_blocks: Vec<Block>,
    if_header_blocks: Vec<Block>,
    if_body_blocks: Vec<Block>,
    if_exit_blocks: Vec<(Block, bool)>,
    entry_block: Block,
    args_index: usize,
    //ins_check_gc: bool,
    for_obj: Vec<Value>,
    for_iter_obj: Vec<Value>,
    logic_end_block: Option<Block>,
    is_body_jump: bool,
    logic_rest_bytecode_count: Option<usize>, // used to track the remaining bytecode count in a logic block
                                              //if_body_line: Option<usize>,
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
        // if let Some(value) = context.exp.last() {
        //     let true_id = self.builder.ins().iconst(
        //         self.module.target_config().pointer_type(),
        //         FSRObject::true_id() as i64,
        //     );
        //     let is_true =
        //         self.builder
        //             .ins()
        //             .icmp(codegen::ir::condcodes::IntCC::Equal, *value, true_id);
        //     // context.exp.push(is_true);
        //     is_true
        // } else {
        //     panic!("IsTrue requires a value operand");
        // }
        context.exp.last().unwrap().clone()
    }

    // fn load_is_not_false(&mut self, context: &mut OperatorContext) -> Value {
    //     if let Some(value) = context.exp.last() {
    //         let false_id = self.builder.ins().iconst(
    //             self.module.target_config().pointer_type(),
    //             FSRObject::false_id() as i64,
    //         );
    //         let is_not_false =
    //             self.builder
    //                 .ins()
    //                 .icmp(codegen::ir::condcodes::IntCC::NotEqual, *value, false_id);
    //         is_not_false
    //     } else {
    //         panic!("IsNotFalse requires a value operand");
    //     }
    // }

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
            is_not_true
        } else {
            panic!("IsNotTrue requires a value operand");
        }
    }

    fn load_compare(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        if let (Some(right), Some(left)) = (context.exp.pop(), context.exp.pop()) {
            // pub extern "C" fn compare_test(thread: &mut FSRThreadRuntime, left: ObjId, right: ObjId, op: CompareOperator)

            // let mut compare_test_sig = self.module.make_signature();
            // compare_test_sig
            //     .params
            //     .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
            // compare_test_sig
            //     .params
            //     .push(AbiParam::new(self.module.target_config().pointer_type())); // left operand
            // compare_test_sig
            //     .params
            //     .push(AbiParam::new(self.module.target_config().pointer_type())); // right operand
            // compare_test_sig.params.push(AbiParam::new(types::I32)); // compare operator type
            // compare_test_sig
            //     .returns
            //     .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (boolean)
            // let fn_id = self
            //     .module
            //     .declare_function(
            //         "compare_test",
            //         cranelift_module::Linkage::Import,
            //         &compare_test_sig,
            //     )
            //     .unwrap();
            // let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
            // let thread_runtime = self.builder.block_params(context.entry_block)[0];
            // // let op = self.builder.ins().iconst(types::I32, 0); // Replace with actual operator type
            // // let op = CompareOperator::new_from_str(context.operator).unwrap() as i32;
            // let op = if let ArgType::Compare(op) = arg.get_arg() {
            //     let v = *op as i64;
            //     self.builder.ins().iconst(types::I32, v)
            // } else {
            //     panic!("CompareTest requires a CompareOperator argument")
            // };
            // let call = self
            //     .builder
            //     .ins()
            //     .call(func_ref, &[thread_runtime, left, right, op]);
            // let result = self.builder.inst_results(call)[0];

            let op = if let ArgType::Compare(op) = arg.get_arg() {
                op
            } else {
                panic!("......not op")
            };

            let result = match op {
                CompareOperator::Equal => {
                    self.builder
                        .ins()
                        .icmp(codegen::ir::condcodes::IntCC::Equal, left, right)
                }
                CompareOperator::NotEqual => {
                    self.builder
                        .ins()
                        .icmp(codegen::ir::condcodes::IntCC::NotEqual, left, right)
                }
                CompareOperator::Greater => self.builder.ins().icmp(
                    codegen::ir::condcodes::IntCC::SignedGreaterThan,
                    left,
                    right,
                ),
                CompareOperator::GreaterEqual => self.builder.ins().icmp(
                    codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual,
                    left,
                    right,
                ),
                CompareOperator::Less => self.builder.ins().icmp(
                    codegen::ir::condcodes::IntCC::SignedLessThan,
                    left,
                    right,
                ),
                CompareOperator::LessEqual => self.builder.ins().icmp(
                    codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                    left,
                    right,
                ),
            };

            context.exp.push(result);
        } else {
            panic!("CompareTest requires both left and right operands");
        }
    }

    fn load_while(&mut self, context: &mut OperatorContext) {
        //let header_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        let exit_block = self.builder.create_block();
        let is_true = self.load_is_true(context);
        let condition = is_true;
        self.builder
            .ins()
            .brif(condition, body_block, &[], exit_block, &[]);

        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);
        context.loop_exit_blocks.push(exit_block);
        //context.ins_check_gc = true;
    }

    fn load_while_end(&mut self, context: &mut OperatorContext) -> Result<()> {
        context.is_body_jump = false;
        self.builder
            .ins()
            .jump(*context.loop_blocks.last().unwrap(), &[]);

        //context.is_while = false;
        // unwrap to error
        let v = context
            .loop_blocks
            .pop()
            .with_context(|| "Failed to pop loop block in load_while_end".to_string())?;
        let exit_block = context
            .loop_exit_blocks
            .pop()
            .with_context(|| "Failed to pop loop exit block in load_while_end".to_string())?;
        self.builder.seal_block(v);
        self.builder.switch_to_block(exit_block);
        self.builder.seal_block(exit_block);
        //context.ins_check_gc = true;
        Ok(())
        //self.builder.ins().iconst(self.int, 0);
    }

    fn load_continue(&mut self, context: &mut OperatorContext) -> Result<()> {
        self.builder
            .ins()
            .jump(*context.loop_blocks.last().unwrap(), &[]);
        context.is_body_jump = true;
        Ok(())
    }

    fn load_break(&mut self, context: &mut OperatorContext) -> Result<()> {
        self.builder
            .ins()
            .jump(*context.loop_exit_blocks.last().unwrap(), &[]);
        context.is_body_jump = true;
        Ok(())
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
        if let ArgType::Local(var) = arg.get_arg() {
            let variable = self.variables.get(&var.name).unwrap();
            self.builder.def_var(*variable, next_obj_value);
            self.defined_variables
                .insert(var.name.to_string(), *variable);

            let v = self.builder.use_var(*variable);
            let condition = self.is_none(v, context);
            let body_block = self.builder.create_block();
            let exit_block = self.builder.create_block();

            self.builder
                .ins()
                .brif(condition, body_block, &[], exit_block, &[]);
            self.builder.switch_to_block(body_block);
            self.builder.seal_block(body_block);
            context.loop_exit_blocks.push(exit_block);
            //context.ins_check_gc = true;
        } else {
            panic!("ForNext requires a Local argument");
        }
    }

    fn load_for_end(&mut self, context: &mut OperatorContext) {
        self.builder.ins().jump(
            *context.loop_blocks.last().unwrap(),
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
        //context.ins_check_gc = true;
        //self.builder.ins().iconst(self.int, 0);
    }

    fn load_if_test(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        //let header_block = self.builder.create_block();
        let body_block = context.if_body_blocks.pop().unwrap();
        let exit_block = context.if_exit_blocks.pop().unwrap();

        // let condition = context.exp.pop().unwrap();
        let is_true = self.load_is_true(context);
        let condition = is_true;
        let not_condition = self.builder.ins().bnot(condition);
        self.builder
            .ins()
            .brif(condition, body_block, &[], exit_block.0, &[not_condition]);

        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);
        context.if_exit_blocks.push((exit_block.0, false));
        //context.ins_check_gc = true;
    }

    fn load_else_if(&mut self, context: &mut OperatorContext) {
        //context.ins_check_gc = true;
        //let test_header_block = context.if_blocks.pop().unwrap();
        //let exit_block = context.if_exit_blocks.pop().unwrap();
        //self.builder.switch_to_block(exit_block.0);
        //self.builder.seal_block(exit_block.0);

        // get last current block param
        if context.if_exit_blocks.last().unwrap().1 || context.is_body_jump {
        } else {
            let false_value = self.builder.ins().iconst(types::I8, 0);
            self.builder
                .ins()
                .jump(context.if_exit_blocks.last().unwrap().0, &[false_value]);
        }

        context.is_body_jump = false;

        let v = context.if_header_blocks.pop().unwrap();
        self.builder.seal_block(v);

        let will_test_block = context.if_exit_blocks.pop().unwrap();
        //self.builder.seal_block(will_test_block.0);
        self.builder.switch_to_block(will_test_block.0);
        let call_be_test = self.builder.block_params(will_test_block.0)[0];
        let header_test_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        self.builder.seal_block(will_test_block.0);
        context.if_header_blocks.push(header_test_block);
        let body_block = self.builder.create_block();
        let end_block = self.builder.create_block();
        self.builder.append_block_param(end_block, types::I8);
        let false_value = self.builder.ins().iconst(types::I8, 0);

        self.builder.ins().brif(
            call_be_test,
            header_test_block,
            &[],
            end_block,
            &[false_value],
        );

        //self.builder.seal_block(header_test_block);
        self.builder.switch_to_block(header_test_block);
        context.if_body_blocks.push(body_block);
        context.if_exit_blocks.push((end_block, false));
    }

    fn load_else_if_test(&mut self, context: &mut OperatorContext) {
        //let header_block = self.builder.create_block();
        // let body_block = self.builder.create_block();
        // let exit_block = self.builder.create_block();
        let body_block = context.if_body_blocks.pop().unwrap();
        let exit_block = context.if_exit_blocks.pop().unwrap();
        // let condition = context.exp.pop().unwrap();
        let is_true = self.load_is_true(context);
        let condition = is_true;
        let not_condition = self.builder.ins().bnot(condition);
        self.builder
            .ins()
            .brif(condition, body_block, &[], exit_block.0, &[not_condition]);

        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);
        context.if_exit_blocks.push((exit_block.0, false));
        //context.ins_check_gc = true;
    }

    fn load_else(&mut self, context: &mut OperatorContext) {
        //context.ins_check_gc = true;
        //let test_header_block = context.if_blocks.pop().unwrap();
        //let exit_block = context.if_exit_blocks.pop().unwrap();
        //self.builder.switch_to_block(exit_block.0);
        //self.builder.seal_block(exit_block.0);

        // get last current block param
        if context.if_exit_blocks.last().unwrap().1 || context.is_body_jump {
        } else {
            let false_value = self.builder.ins().iconst(types::I8, 0);
            self.builder
                .ins()
                .jump(context.if_exit_blocks.last().unwrap().0, &[false_value]);
        }

        context.is_body_jump = false;

        let v = context.if_header_blocks.pop().unwrap();
        self.builder.seal_block(v);

        let will_test_block = context.if_exit_blocks.pop().unwrap();
        //self.builder.seal_block(will_test_block.0);
        self.builder.switch_to_block(will_test_block.0);
        let call_be_test = self.builder.block_params(will_test_block.0)[0];
        let header_test_block = self.builder.create_block();
        //let body_block = self.builder.create_block();
        self.builder.seal_block(will_test_block.0);
        context.if_header_blocks.push(header_test_block);
        //let body_block = self.builder.create_block();
        let end_block = self.builder.create_block();
        self.builder.append_block_param(end_block, types::I8);
        let false_value = self.builder.ins().iconst(types::I8, 0);

        self.builder.ins().brif(
            call_be_test,
            header_test_block,
            &[],
            end_block,
            &[false_value],
        );

        //self.builder.seal_block(header_test_block);
        self.builder.switch_to_block(header_test_block);
        //context.if_body_blocks.push(body_block);
        context.if_exit_blocks.push((end_block, false));
        //unimplemented!("LoadElse is not implemented yet. This function should handle the else block logic.");
    }

    fn load_if_end(&mut self, context: &mut OperatorContext) {
        if context.if_exit_blocks.last().unwrap().1 || context.is_body_jump {
        } else {
            let false_value = self.builder.ins().iconst(types::I8, 0);
            self.builder
                .ins()
                .jump(context.if_exit_blocks.last().unwrap().0, &[false_value]);
        }

        context.is_body_jump = false;

        //self.builder.ins().nop();

        //context.is_while = false;
        let v = context.if_header_blocks.pop().unwrap();
        context.if_body_blocks.pop();
        let exit_block = context.if_exit_blocks.pop().unwrap();
        self.builder.seal_block(v);
        self.builder.switch_to_block(exit_block.0);
        self.builder.seal_block(exit_block.0);
        //context.ins_check_gc = true;
        //self.builder.ins().iconst(self.int, 0);
    }

    fn load_if_body_end(&mut self, context: &mut OperatorContext) {
        // if context.if_exit_blocks.last().unwrap().1 {
        // } else {
        //     let params = self.builder.block_params(context.if_exit_blocks.last().unwrap().0)[0];
        //     self.builder
        //         .ins()
        //         .jump(context.if_exit_blocks.last().unwrap().clone().0, &[params]);
        // }

        // //self.builder.ins().nop();

        // //context.is_while = false;
        // let v = context.if_blocks.pop().unwrap();
        // let exit_block = *context.if_exit_blocks.last().unwrap();
        // self.builder.seal_block(v);
        // self.builder.switch_to_block(exit_block.0);
        // self.builder.seal_block(exit_block.0);
        // context.ins_check_gc = true;
        //self.builder.ins().iconst(self.int, 0);
    }

    fn load_make_arg_list(&mut self, context: &mut OperatorContext, len: usize) -> Value {
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

    fn load_make_method_arg_list(&mut self, context: &mut OperatorContext, len: usize) -> Value {
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

        rev_args.push(context.exp.pop().unwrap()); // Add the method object as the last argument

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

    fn get_cl_type(ptr: ir::Type, ty: &FSRSType) -> types::Type {
        match ty {
            FSRSType::Bool => types::I8,
            FSRSType::UInt8 | FSRSType::IInt8 => types::I8,
            FSRSType::UInt16 | FSRSType::IInt16 => types::I16,
            FSRSType::UInt32 | FSRSType::IInt32 => types::I32,
            FSRSType::UInt64 | FSRSType::IInt64 => types::I64,
            FSRSType::Float32 => types::F32,
            FSRSType::Float64 => types::F64,
            FSRSType::String => todo!(),
            FSRSType::Struct(fsrstruct) => ptr,
            FSRSType::Ptr(fsrstype) => ptr,
            FSRSType::Fn(fn_call_sig) => ptr,
        }
    }

    fn make_inner_call_fn(&self, call_sig: &FnCallSig) -> Signature {
        let mut inner_call_fn_sig = self.module.make_signature();
        let ptr = self.module.target_config().pointer_type();
        inner_call_fn_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
        for params in call_sig.params.iter() {
            inner_call_fn_sig
                .params
                .push(AbiParam::new(Self::get_cl_type(ptr ,params))); // args
        }

        if let Some(ret_type) = &call_sig.return_type {
            inner_call_fn_sig
                .returns
                .push(AbiParam::new(Self::get_cl_type(ptr, ret_type)));
        } else {
            inner_call_fn_sig.returns.push(AbiParam::new(types::I64)); // return type (ObjId)
        }

        inner_call_fn_sig
    }

    fn load_call(&mut self, arg: &BytecodeArg, context: &mut OperatorContext) {
        if let ArgType::CallArgsNumber((len, call_sig)) = arg.get_arg() {
            let call_fn_sig = self.make_inner_call_fn(call_sig.as_ref().unwrap());

            // generate SigRef from Signature
            let call_fn_sig_ref = self.builder.import_signature(call_fn_sig.clone());
            let mut rev_args = vec![];

            for i in 0..*len {
                // Assuming we have a way to get the next argument value
                let arg_value = context.exp.pop().unwrap(); // This should be replaced with actual argument retrieval logic
                context.middle_value.push(arg_value);
                rev_args.push(arg_value);
            }
            let fn_ptr = context.exp.pop().unwrap();
            rev_args.reverse();
            rev_args.insert(0, self.builder.block_params(context.entry_block)[0]); // insert thread runtime at the beginning
            let call_inst = self
                .builder
                .ins()
                .call_indirect(call_fn_sig_ref, fn_ptr, &rev_args);
            let ret = self.builder.inst_results(call_inst)[0];

            // Free the argument list after the call
            //self.load_free_arg_list(list_ptr, context, *v as i64);

            context.exp.push(ret);
            context.middle_value.push(ret);
        } else {
            unimplemented!()
        }
    }

    fn get_obj_method(&mut self, father: Value, name: &str) -> Value {
        // pub extern "C" fn get_obj_method(father: ObjId, name: *const u8, len: usize) -> ObjId {
        let mut get_obj_method_sig = self.module.make_signature();
        get_obj_method_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // father object
        get_obj_method_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // name pointer
        get_obj_method_sig.params.push(AbiParam::new(types::I64)); // name length
        get_obj_method_sig
            .returns
            .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
        let fn_id = self
            .module
            .declare_function(
                "get_obj_method",
                cranelift_module::Linkage::Import,
                &get_obj_method_sig,
            )
            .unwrap();
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let name_ptr = self.builder.ins().iconst(
            self.module.target_config().pointer_type(),
            name.as_ptr() as i64,
        );
        let name_len = self.builder.ins().iconst(types::I64, name.len() as i64);
        let call = self
            .builder
            .ins()
            .call(func_ref, &[father, name_ptr, name_len]);
        let ret = self.builder.inst_results(call)[0];
        ret
    }

    fn load_call_method(&mut self, arg: &BytecodeArg, context: &mut OperatorContext) {
        if let ArgType::CallArgsNumberWithAttr(v) = arg.get_arg() {
            let father_obj_id = *context.exp.last().unwrap();
            let fn_obj_id = self.get_obj_method(father_obj_id, v.2.as_str());

            let call_fn_sig = self.make_call_fn();
            let fn_id = self
                .module
                .declare_function("call_fn", cranelift_module::Linkage::Import, &call_fn_sig)
                .unwrap();
            let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
            let list_ptr = self.load_make_method_arg_list(context, v.0);
            let len = self.builder.ins().iconst(types::I64, v.0 as i64 + 1);
            let thread_runtime = self.builder.block_params(context.entry_block)[0];
            let code_object = self.builder.block_params(context.entry_block)[1];

            //self.save_middle_value(context);
            // self.save_object_to_exp(context);
            let call = self.builder.ins().call(
                func_ref,
                &[list_ptr, len, fn_obj_id, thread_runtime, code_object],
            );

            let ret = self.builder.inst_results(call)[0];

            context.exp.push(ret);
            context.middle_value.push(ret);
        } else {
            unimplemented!()
        }
    }

    fn load_none(&mut self, context: &mut OperatorContext) {
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
            let ret = match op {
                BinaryOffset::Add => {
                    // For addition, we can use integer addition directly
                    self.builder.ins().iadd(left, right)
                }
                BinaryOffset::Sub => {
                    // For subtraction, we can use integer subtraction directly
                    self.builder.ins().isub(left, right)
                }
                BinaryOffset::Mul => {
                    // For multiplication, we can use integer multiplication directly
                    self.builder.ins().imul(left, right)
                }
                BinaryOffset::Div => {
                    // For division, we can use integer division directly
                    self.builder.ins().sdiv(left, right)
                }
                _ => {
                    unimplemented!("Binary operation {:?} is not implemented yet", op);
                }
            };
            context.exp.push(ret);
            // context.middle_value.push(ret);
            // context.middle_value.push(right);
            // context.middle_value.push(left);
            // self.clear_middle_value(context);
        } else {
            unimplemented!("BinaryAdd requires both left and right operands");
        }
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

    fn load_data(&mut self, var_type: &Arc<FSRSType>, value: Value) -> Value {
        // input a data ptr to get value
        match var_type.as_ref() {
            FSRSType::Bool => {
                // like &i8 to i8
                self.builder.ins().load(
                    types::I8,
                    cranelift::codegen::ir::MemFlags::new(),
                    value,
                    0,
                )
            }
            FSRSType::UInt8 => self.builder.ins().load(
                types::I8,
                cranelift::codegen::ir::MemFlags::new(),
                value,
                0,
            ),
            FSRSType::UInt16 => self.builder.ins().load(
                types::I16,
                cranelift::codegen::ir::MemFlags::new(),
                value,
                0,
            ),
            FSRSType::UInt32 => self.builder.ins().load(
                types::I32,
                cranelift::codegen::ir::MemFlags::new(),
                value,
                0,
            ),
            FSRSType::UInt64 => self.builder.ins().load(
                types::I64,
                cranelift::codegen::ir::MemFlags::new(),
                value,
                0,
            ),
            FSRSType::IInt8 => self.builder.ins().load(
                types::I8,
                cranelift::codegen::ir::MemFlags::new(),
                value,
                0,
            ),
            FSRSType::IInt16 => self.builder.ins().load(
                types::I16,
                cranelift::codegen::ir::MemFlags::new(),
                value,
                0,
            ),
            FSRSType::IInt32 => self.builder.ins().load(
                types::I32,
                cranelift::codegen::ir::MemFlags::new(),
                value,
                0,
            ),
            FSRSType::IInt64 => self.builder.ins().load(
                types::I64,
                cranelift::codegen::ir::MemFlags::new(),
                value,
                0,
            ),
            FSRSType::Float32 => self.builder.ins().load(
                types::F32,
                cranelift::codegen::ir::MemFlags::new(),
                value,
                0,
            ),
            FSRSType::Float64 => self.builder.ins().load(
                types::F64,
                cranelift::codegen::ir::MemFlags::new(),
                value,
                0,
            ),
            FSRSType::String => todo!(),
            FSRSType::Struct(fsrstruct) => value,
            FSRSType::Ptr(fsrstype) => value,
            FSRSType::Fn(fn_call_sig) => value,
        }
    }

    fn load_static_args(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        let base = 1; // first args is taken by FSRThreadRuntime
        let index = context.args_index;
        if let ArgType::Local(v) = arg.get_arg() {
            if let Some(var_type) = &v.var_type {
                let new_type = self.get_var_type(&v.var_type.as_ref().unwrap());
                let var_type = new_type.unwrap();
                let mut var_id = self.var_index;
                let new_var = declare_variable(
                    var_type,
                    &mut self.builder,
                    &mut self.variables,
                    &mut var_id,
                    &v.name,
                );
                self.var_index = var_id;
            }

            let data = self.builder.block_params(context.entry_block)[base + index];
            let variable = self.variables.get(v.name.as_str()).unwrap();

            self.builder.def_var(*variable, data);
            self.defined_variables.insert(v.name.to_string(), *variable);
        } else {
            panic!("GetArgs requires a Local argument");
        }

        context.args_index += 1;
    }

    fn load_entry_args(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
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
            if let Some(var_type) = &v.var_type {
                let new_type = self.get_var_type(&v.var_type.as_ref().unwrap());
                let var_type = new_type.unwrap();
                let mut var_id = self.var_index;
                let new_var = declare_variable(
                    var_type,
                    &mut self.builder,
                    &mut self.variables,
                    &mut var_id,
                    &v.name,
                );
                self.var_index = var_id;
            }

            let trans_data = Self::load_data(self, v.var_type.as_ref().unwrap(), ret);
            Self::println(self, context, trans_data);
            let variable = self.variables.get(v.name.as_str()).unwrap();

            self.builder.def_var(*variable, trans_data);
            self.defined_variables.insert(v.name.to_string(), *variable);
        } else {
            panic!("GetArgs requires a Local argument");
        }

        context.args_index += 1;
    }

    fn load_or_jump(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        // process or logic like a or b
        //let last_ssa_value = *context.exp.last().unwrap();
        let last_ssa_value = self.load_is_true(context);
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

    fn load_and_jump(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) -> Result<()> {
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
            return Err(anyhow::anyhow!("AndJump requires an AddOffset argument"));
        }

        Ok(())
    }

    fn load_init_integer(
        &mut self,
        arg: &BytecodeArg,
        context: &mut OperatorContext,
    ) -> Result<()> {
        if let ArgType::ConstInteger(id, s, op) = arg.get_arg() {
            // let mut load_integer_sig = self.module.make_signature();
            // load_integer_sig.params.push(AbiParam::new(types::I64)); // value
            // load_integer_sig
            //     .params
            //     .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
            // load_integer_sig
            //     .returns
            //     .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
            let v = match op {
                Some(SingleOp::Minus) => -*s,
                None => *s,
                _ => panic!("Unsupported single operation for constant integer"),
            };

            let ret = self.builder.ins().iconst(types::I64, v);

            let name = format!("{}_constant", id);

            let variable = self.variables.get(&name).unwrap();
            self.builder.def_var(*variable, ret);
            self.defined_variables.insert(name.to_string(), *variable);
        } else {
            return Err(anyhow::anyhow!("Expected ConstInteger argument"));
        }

        Ok(())
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

    fn load_list(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) -> Result<()> {
        if let ArgType::LoadListNumber(len) = arg.get_arg() {
            let arg_ptr = self
                .builder
                .use_var(*self.variables.get("#args_ptr").unwrap());

            let len_value = self
                .builder
                .ins()
                .iconst(self.module.target_config().pointer_type(), *len as i64);
            let mut list_args = vec![];
            for i in 0..*len {
                // store to arg_ptr
                list_args.push(context.exp.pop().unwrap());
            }

            for i in list_args.iter().enumerate() {
                let offset = self
                    .builder
                    .ins()
                    .iconst(types::I64, i.0 as i64 * std::mem::size_of::<ObjId>() as i64); // Replace with actual offset calculation
                let ptr = self.builder.ins().iadd(arg_ptr, offset);
                self.builder
                    .ins()
                    .store(cranelift::codegen::ir::MemFlags::new(), *i.1, ptr, 0);
            }

            // pub extern "C" fn load_list(args: *const ObjId, len: usize, thread: &mut FSRThreadRuntime) -> ObjId
            let mut load_list_sig = self.module.make_signature();
            load_list_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // args pointer
            load_list_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // length of the list
            load_list_sig
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
            load_list_sig
                .returns
                .push(AbiParam::new(self.module.target_config().pointer_type())); // return type (ObjId)
            let fn_id = self
                .module
                .declare_function(
                    "load_list",
                    cranelift_module::Linkage::Import,
                    &load_list_sig,
                )
                .unwrap();
            let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
            let thread_runtime = self.builder.block_params(context.entry_block)[0];
            let call = self
                .builder
                .ins()
                .call(func_ref, &[arg_ptr, len_value, thread_runtime]);
            let ret = self.builder.inst_results(call)[0];
            context.exp.push(ret);
            return Ok(());
        }
        unimplemented!()
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

    fn load_ptr_data(&mut self, var_type: &Arc<FSRSType>, value: Value) -> Value {
        // input a data ptr to get value
        match var_type.as_ref() {
            &FSRSType::UInt64 => {
                // like &i64 to i64
                self.builder.ins().load(
                    types::I64,
                    cranelift::codegen::ir::MemFlags::new(),
                    value,
                    0,
                )
            },
            
            FSRSType::Ptr(fsrstype) => {
                value
            },
            _ => panic!("load_ptr_data expects a Ptr type"),
        }
    }

    fn binary_dot_process(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        if let ArgType::Attr(attr_var) = arg.get_arg() {
            let attr_type = attr_var.attr_type.as_ref().unwrap().clone();
            let offset = attr_var.offset.unwrap();
            let father_value = *context.exp.last().unwrap();

            let addr = self.builder.ins().iadd_imm(father_value, offset as i64);
            let data = Self::load_ptr_data(self, &attr_type, addr);
            context.exp.push(data);
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

    fn println(&mut self, context: &mut OperatorContext, value: Value) {
        // pub extern "C" fn c_println(obj: i64) {
        let mut println_sig = self.module.make_signature();
        println_sig
            .params
            .push(AbiParam::new(self.module.target_config().pointer_type())); // object to print
        println_sig.returns.push(AbiParam::new(types::I32)); // return type
        let fn_id = self
            .module
            .declare_function("c_println", cranelift_module::Linkage::Import, &println_sig)
            .unwrap();
        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
        let call = self.builder.ins().call(func_ref, &[value]);
        let _ret = self.builder.inst_results(call)[0];
    }

    fn malloc_call_args(&mut self, context: &mut OperatorContext) {
        let var = self.variables.get("#call_args_ptr").unwrap();
        // self.builder.def_var(*var, malloc_ret);
        let slot = self.builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            CALL_ARGS_LEN as u32,
            0,
        ));
        let stack_slot_addr =
            self.builder
                .ins()
                .stack_addr(self.module.target_config().pointer_type(), slot, 0);
        self.builder.def_var(*var, stack_slot_addr);
    }

    fn struct_alloc(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        let size = if let ArgType::Alloc((_, size)) = arg.get_arg() {
            *size as i64
        } else {
            panic!("StructAlloc requires a StructSize argument");
        };
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

        let size_value = self
            .builder
            .ins()
            .iconst(self.module.target_config().pointer_type(), size);
        let malloc_call = self.builder.ins().call(malloc_func_ref, &[size_value]);
        let malloc_ret = self.builder.inst_results(malloc_call)[0];

        context.exp.push(malloc_ret);
    }

    fn store_attr(&mut self, context: &mut OperatorContext, arg: &BytecodeArg) {
        if let ArgType::Attr(attr_var) = arg.get_arg() {
            let attr_type = attr_var.attr_type.as_ref().unwrap().clone();
            let offset = attr_var.offset.unwrap();
            let father_value = context.exp.pop().unwrap();
            let value_to_store = context.exp.pop().unwrap();
            let op_assign = attr_var.op_assign;

            let value_to_store = if let Some(op_assign) = op_assign {
                // load current value
                let addr = self.builder.ins().iadd_imm(father_value, offset as i64);
                let current_value = Self::load_data(self, &attr_type, addr);
                // perform operation
                let new_value = match op_assign {
                    OpAssign::Add => self.builder.ins().iadd(current_value, value_to_store),
                    OpAssign::Sub => self.builder.ins().isub(current_value, value_to_store),
                    OpAssign::Mul => self.builder.ins().imul(current_value, value_to_store),
                    OpAssign::Div => self.builder.ins().sdiv(current_value, value_to_store),
                    OpAssign::Reminder => self.builder.ins().srem(current_value, value_to_store),
                };
                new_value
            } else {
                value_to_store
            };

            let addr = self.builder.ins().iadd_imm(father_value, offset as i64);
            self.builder.ins().store(
                cranelift::codegen::ir::MemFlags::new(),
                value_to_store,
                addr,
                0,
            );
        } else {
            panic!("StoreAttr requires an Attr argument");
        }
    }

    fn get_var_type(&self, s_type: &FSRSType) -> Option<types::Type> {
        match s_type {
            FSRSType::UInt8 => Some(types::I8),
            FSRSType::UInt16 => Some(types::I16),
            FSRSType::UInt32 => Some(types::I32),
            FSRSType::UInt64 => Some(types::I64),
            FSRSType::IInt8 => Some(types::I8),
            FSRSType::IInt16 => Some(types::I16),
            FSRSType::IInt32 => Some(types::I32),
            FSRSType::IInt64 => Some(types::I64),
            FSRSType::Float32 => Some(types::F32),
            FSRSType::Float64 => Some(types::F64),
            FSRSType::String => Some(self.module.target_config().pointer_type()),
            FSRSType::Struct(_) => None,
            FSRSType::Bool => Some(types::I8),
            FSRSType::Ptr(fsrstype) => Some(self.module.target_config().pointer_type()),
            FSRSType::Fn(fn_call_sig) => Some(self.module.target_config().pointer_type()),
        }
    }

    fn compile_expr(
        &mut self,
        expr: &[BytecodeArg],
        context: &mut OperatorContext,
        code: ObjId,
        is_entry: bool,
    ) {
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

        if expr.last().unwrap().get_operator() == &BytecodeOperator::IfTest
        //|| expr.last().unwrap().get_operator() == &BytecodeOperator::ElseIfTest
        {
            //context.is_if += 1;
            let header_block = self.builder.create_block();
            let body_block = self.builder.create_block();
            let end_block = self.builder.create_block();
            self.builder.append_block_param(end_block, types::I8);
            self.builder.ins().jump(header_block, &[]);
            self.builder.switch_to_block(header_block);
            context.if_header_blocks.push(header_block);
            context.if_body_blocks.push(body_block);
            context.if_exit_blocks.push((end_block, false));
        }

        if expr.last().unwrap().get_operator() == &BytecodeOperator::LoadForIter {
            //context.is_for += 1;
        }

        for arg in expr {
            match arg.get_operator() {
                BytecodeOperator::Load => {
                    if let ArgType::Local(v) = arg.get_arg() {
                        let variable = self.variables.get(v.name.as_str()).unwrap();
                        // context.left = Some(self.builder.use_var(*variable));
                        let value = self.builder.use_var(*variable);
                        context.exp.push(value);
                    } else if let ArgType::JitFunction(f_name) = arg.get_arg() {
                        let module = FSRObject::id_to_obj(code).as_code().module;
                        let module_obj = FSRObject::id_to_obj(module).as_module();
                        let target_fn = module_obj
                            .jit_code_map
                            .get(f_name)
                            .and_then(|x| x.clone())
                            .expect("Not found jit");
                        let fn_value = self
                            .builder
                            .ins()
                            .iconst(self.module.target_config().pointer_type(), target_fn as i64);
                        context.exp.push(fn_value);
                        //self.load_jit_function(f_name, context);
                    } else if let ArgType::Const(c) = arg.get_arg() {
                        self.load_constant(*c, context);
                    } else if let ArgType::Global(name) = arg.get_arg() {
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
                        //let true_id = FSRObject::true_id();
                        let true_value = self.builder.ins().iconst(types::I8, 1 as i64);
                        context.exp.push(true_value);
                    } else if let ArgType::LoadFalse = arg.get_arg() {
                        //let false_id = FSRObject::false_id();
                        let false_value = self.builder.ins().iconst(types::I8, 0 as i64);
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
                        let name_len = self
                            .builder
                            .ins()
                            .iconst(self.module.target_config().pointer_type(), v.1.len() as i64);
                        self.load_global_name(name_ptr, name_len, context);
                    } else {
                        panic!("Load requires a variable or constant argument");
                    }

                    //unimplemented!()
                }
                BytecodeOperator::Assign => {
                    if let ArgType::Local(v) = arg.get_arg() {
                        if let Some(var_type) = &v.var_type {
                            let new_type = self.get_var_type(&v.var_type.as_ref().unwrap());
                            let var_type = new_type.unwrap();
                            let mut var_id = self.var_index;
                            let new_var = declare_variable(
                                var_type,
                                &mut self.builder,
                                &mut self.variables,
                                &mut var_id,
                                &v.name,
                            );
                            self.var_index = var_id;
                        }

                        let op_assign = v.op_assign;
                        let var = if let Some(op) = op_assign {
                            // load the current value
                            let variable = self.variables.get(v.name.as_str()).unwrap();
                            let current_value = self.builder.use_var(*variable);
                            let assign_value = context.exp.pop().unwrap();
                            // the value to assign is already on the stack
                            let result = match op {
                                crate::backend::compiler::bytecode::OpAssign::Add => {
                                    let result = self.builder.ins().iadd(current_value, assign_value);
                                    result
                                },
                                crate::backend::compiler::bytecode::OpAssign::Sub => {
                                    let result = self.builder.ins().isub(current_value, assign_value);
                                    result
                                },
                                crate::backend::compiler::bytecode::OpAssign::Mul => {
                                    let result = self.builder.ins().imul(current_value, assign_value);
                                    result
                                },
                                crate::backend::compiler::bytecode::OpAssign::Div => {
                                    let result = self.builder.ins().sdiv(current_value, assign_value);
                                    result
                                },
                                crate::backend::compiler::bytecode::OpAssign::Reminder => {
                                    let result = self.builder.ins().srem(current_value, assign_value);
                                    result
                                },
                            };
                            result
                        } else {
                            let var = context.exp.pop().unwrap();
                            var
                        };

                        let variable = self.variables.get(v.name.as_str()).unwrap();
                        
                        context.middle_value.push(var);
                        self.builder.def_var(*variable, var);
                        self.defined_variables.insert(v.name.to_string(), *variable);
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
                    if is_entry {
                        self.load_entry_args(context, arg);
                    } else {
                        self.load_static_args(context, arg);
                    }

                    //context.ins_check_gc = true;
                }

                BytecodeOperator::EndFn => {
                    // do nothing, function will auto add return operator
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
                    // let args_ptr = self
                    //     .builder
                    //     .use_var(*self.variables.get("#args_ptr").unwrap());
                    // let call_args_ptr = self
                    //     .builder
                    //     .use_var(*self.variables.get("#call_args_ptr").unwrap());
                    //self.load_free_arg_list(args_ptr, context, ARGS_LEN);
                    //self.load_free_arg_list(call_args_ptr, context, CALL_ARGS_LEN);
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
                    self.load_if_test(context, arg);
                }
                BytecodeOperator::ElseIf => {
                    self.load_else_if(context);
                }
                BytecodeOperator::ElseIfTest => {
                    self.load_else_if_test(context);
                }
                BytecodeOperator::Else => {
                    self.load_else(context);
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
                BytecodeOperator::CallMethod => {
                    self.load_call_method(arg, context);
                }
                BytecodeOperator::LoadList => {
                    self.load_list(context, arg);
                }
                BytecodeOperator::Continue => {
                    self.load_continue(context);
                }
                BytecodeOperator::Break => {
                    self.load_break(context);
                }
                BytecodeOperator::SAlloc => {
                    self.struct_alloc(context, arg);
                }
                BytecodeOperator::AssignAttr => {
                    self.store_attr(context, arg);
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
    var_type: types::Type,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
    index: &mut usize,
    name: &str,
) -> Variable {
    let var = Variable::new(*index);
    if !variables.contains_key(name) {
        variables.insert(name.into(), var);
        builder.declare_var(var, var_type);
        *index += 1;
    }
    var
}

fn declare_variables(
    module: &JITModule,
    var_type: types::Type,
    builder: &mut FunctionBuilder,
    params: &[String],
    //the_return: &str,
    //stmts: &[Expr],
    entry_block: Block,
) -> (HashMap<String, Variable>, usize) {
    let mut variables = HashMap::new();
    let mut index = 0;

    for (i, name) in params.iter().enumerate() {
        // TODO: cranelift_frontend should really have an API to make it easy to set
        // up param variables.
        // let val = builder.block_params(entry_block)[i];
        let val = builder
            .ins()
            .iconst(module.target_config().pointer_type(), 0);
        let var = declare_variable(var_type, builder, &mut variables, &mut index, name);
        builder.def_var(var, val);
    }

    let zero = builder.ins().iconst(var_type, 0);
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

    (variables, index)
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
        builder.symbol("get_obj_method", get_obj_method as *const u8);
        builder.symbol("load_list", load_list as *const u8);
        builder.symbol("c_println", c_println as *const u8);
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

    pub fn compile(
        &mut self,
        bs_code: &Bytecode,
        code: ObjId,
        is_entry: bool,
        call_sig: Option<Arc<FnCallSig>>,
    ) -> Result<*const u8> {
        let ptr = self.module.target_config().pointer_type();

        if is_entry {
            self.ctx.func.signature.params.push(AbiParam::new(ptr)); // Add a parameter for the thread runtime.
            self.ctx.func.signature.params.push(AbiParam::new(ptr)); // Add a parameter for the code object.
                                                                     // self.ctx.func.signature.params.push(AbiParam::new(ptr)); // Add a parameter for list of arguments.
                                                                     // self.ctx
                                                                     //     .func
                                                                     //     .signature
                                                                     //     .params
                                                                     //     .push(AbiParam::new(types::I32)); // Add a parameter for the number of arguments.
            self.ctx.func.signature.returns.push(AbiParam::new(ptr)); // Add a return type for the function.
        } else {
            self.ctx.func.signature
                .params
                .push(AbiParam::new(self.module.target_config().pointer_type())); // thread runtime
            for params in call_sig.as_ref().unwrap().params.iter() {
                self.ctx.func.signature
                    .params
                    .push(AbiParam::new(JitBuilder::get_cl_type(ptr, params))); // args
            }

            if let Some(ret_type) = &call_sig.as_ref().unwrap().return_type {
                self.ctx.func.signature
                    .returns
                    .push(AbiParam::new(JitBuilder::get_cl_type(ptr, ret_type.as_ref())));
            } else {
                self.ctx.func.signature.returns.push(AbiParam::new(types::I64)); // return type (ObjId)
            }

        }

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        //let mut variables = code.var_map.var_map.keys().cloned().collect::<Vec<_>>();
        let mut variables = vec![];

        let constans = bs_code
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
            entry_block,
            if_header_blocks: vec![],
            if_exit_blocks: vec![],
            args_index: 0,
            //ins_check_gc: false,
            for_obj: vec![],
            for_iter_obj: vec![],
            logic_end_block: None,
            logic_rest_bytecode_count: None,
            middle_value: vec![],
            //if_body_line: None,
            if_body_blocks: vec![],
            is_body_jump: false,
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
            variables: variables.0,
            module: &mut self.module,
            defined_variables: HashMap::new(),
            constans: HashMap::new(),
            var_index: variables.1,
            self_call_sig: call_sig.unwrap(),
        };

        //trans.malloc_args(&mut context);
        trans.malloc_call_args(&mut context);
        for (i, expr) in bs_code.bytecode.iter().enumerate() {
            // if i % 20 == 0 || context.ins_check_gc {
            //     trans.load_check_gc(&mut context);
            //     context.ins_check_gc = false;
            // }

            trans.compile_expr(expr, &mut context, code, is_entry);
            context.exp.clear();
            context.middle_value.clear();
        }

        trans.builder.finalize();

        let fn_name = bs_code.name.as_str();

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
