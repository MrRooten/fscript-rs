use std::collections::HashMap;

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
    compiler::bytecode::{ArgType, Bytecode, BytecodeArg, BytecodeOperator},
    types::base::{FSRObject, ObjId},
    vm::thread::FSRThreadRuntime,
};

use super::jit_wrapper::get_constant;

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
        let code = self
            .builder
            .block_params(self.builder.current_block().unwrap())[1];
        let index = self.builder.ins().iconst(types::I64, c as i64);
        let call = self.builder.ins().call(func_ref, &[code, index]);
        let ret = self.builder.inst_results(call)[0];
        if context.left.is_none() {
            context.left = Some(ret);
        } else if context.right.is_none() {
            context.right = Some(ret);
        } else {
            panic!("Both left and right operands are already set");
        }
    }
    fn compile_expr(&mut self, expr: &[BytecodeArg], context: &mut OperatorContext) {
        for arg in expr {
            match arg.get_operator() {
                BytecodeOperator::Load => {
                    if let ArgType::Variable(v) = arg.get_arg() {
                        let variable = self.variables.get(v.1.as_str()).unwrap();
                        // context.left = Some(self.builder.use_var(*variable));
                        if context.left.is_none() {
                            context.left = Some(self.builder.use_var(*variable));
                        } else if context.right.is_none() {
                            context.right = Some(self.builder.use_var(*variable));
                        } else {
                            panic!("Both left and right operands are already set");
                        }
                    } else if let ArgType::Const(c) = arg.get_arg() {
                        self.load_constant(*c, context);
                    } else {
                        panic!("Load requires a variable or constant argument");
                    }

                    //unimplemented!()
                }
                BytecodeOperator::Assign => {
                    if let ArgType::Variable(v) = arg.get_arg() {
                        let variable = self.variables.get(v.1.as_str()).unwrap();
                        self.builder
                            .def_var(*variable, context.left.take().unwrap());
                    } else {
                        panic!("not supported assign type: {:?}", arg.get_arg());
                    }
                }
                BytecodeOperator::BinaryAdd => {
                    if let (Some(left), Some(right)) = (context.left.take(), context.right.take()) {
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
                        let func_ref = self.module.declare_func_in_func(fn_id, self.builder.func);
                        let add_t = self.builder.ins().iconst(types::I32, 0);
                        let call = self.builder.ins().call(func_ref, &[left, right, add_t]);
                        let ret = self.builder.inst_results(call)[0];
                        context.left = Some(ret);
                    } else {
                        unimplemented!("BinaryAdd requires both left and right operands");
                    }
                }

                BytecodeOperator::AssignArgs => {}

                BytecodeOperator::EndFn => {}
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

struct OperatorContext {
    left: Option<Value>,
    right: Option<Value>,
    operator: &'static str,
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
        builder.symbol("binary_op", FSRThreadRuntime::binary_op as *const u8);
        builder.symbol("get_constant", get_constant as *const u8);
        let module = JITModule::new(builder);

        CraneLiftJitBackend {
            ctx: codegen::Context::new(),
            builder_context: FunctionBuilderContext::new(),
            //variable: HashMap::new(),
            module,
        }
    }

    pub fn compile(&mut self, code: &Bytecode) -> Result<(), String> {
        let mut context = OperatorContext {
            left: None,
            right: None,
            operator: "",
        };

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
        }

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

        self.module.define_function(id, &mut self.ctx).unwrap();

        println!("Cranelift JIT compiled function: {}", fn_name);
        println!("{}", self.ctx.func.display());

        self.module.clear_context(&mut self.ctx);
        // Tell the builder we're done with this function.
        Ok(())
    }
}

mod test {
    use crate::backend::{
        types::{base::FSRObject, code::FSRCode, module::FSRModule},
        vm::virtual_machine::FSRVM,
    };

    #[test]
    fn test_module() {
        let _ = FSRVM::single();
        let module1 = r#"
        fn abc() {
            a = 1 + 2
            b = 1 + 3
            c = a + b
        }

        abc()
        "#;
        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", module1, obj_id).unwrap();
        let obj = v.get("abc").unwrap().as_code();
        let bytecode = obj.get_bytecode();
        let mut jit = super::CraneLiftJitBackend::new();
        jit.compile(&bytecode).unwrap();
        //println!("Code object: {:#?}", obj);
    }
}
