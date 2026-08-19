//! Cranelift-based JIT compiler for Franken Shell
//!
//! Compiles hot loops to native code using Cranelift as the backend.

use super::{LoopId, MAX_COMPILED_LOOPS};
use rustc_hash::FxHashMap;

use cranelift_codegen::Context;
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature, UserFuncName, types};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Linkage, Module};

/// Type alias for compiled loop functions
///
/// The function signature is: fn(vars: *mut i64, nvars: usize) -> i64
/// - vars: pointer to array of variable values
/// - nvars: number of variables
/// - returns: exit status code
pub type CompiledLoopFn = unsafe extern "C" fn(*mut i64, usize) -> i64;

/// JIT compiler using Cranelift
pub struct JitCompiler {
    /// The JIT module for code generation
    module: JITModule,
    /// Function builder context (reused for efficiency)
    builder_ctx: FunctionBuilderContext,
    /// Codegen context
    ctx: Context,
    /// Compiled functions cache
    compiled: FxHashMap<LoopId, CompiledLoopFn>,
    /// Data description for constants
    #[allow(dead_code)]
    data_desc: DataDescription,
}

impl JitCompiler {
    /// Create a new JIT compiler
    pub fn new() -> Result<Self, String> {
        // Create ISA (Instruction Set Architecture) for the host
        let mut flag_builder = settings::builder();
        flag_builder
            .set("opt_level", "speed")
            .map_err(|e| e.to_string())?;
        flag_builder
            .set("is_pic", "true")
            .map_err(|e| e.to_string())?;

        let isa_builder =
            cranelift_native::builder().map_err(|e| format!("Failed to get native ISA: {}", e))?;

        let flags = settings::Flags::new(flag_builder);
        let isa = isa_builder.finish(flags).map_err(|e| e.to_string())?;

        // Create JIT module
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        Ok(Self {
            module,
            builder_ctx: FunctionBuilderContext::new(),
            ctx: Context::new(),
            compiled: FxHashMap::default(),
            data_desc: DataDescription::new(),
        })
    }

    /// Check if a loop is already compiled
    pub fn is_compiled(&self, loop_id: LoopId) -> bool {
        self.compiled.contains_key(&loop_id)
    }

    /// Get a compiled loop function
    pub fn get_compiled(&self, loop_id: LoopId) -> Option<CompiledLoopFn> {
        self.compiled.get(&loop_id).copied()
    }

    /// Compile a simple counting loop (demonstration)
    ///
    /// This compiles a loop equivalent to:
    /// ```sh
    /// sum=0
    /// for i in $(seq 1 $count); do
    ///     sum=$((sum + i))
    /// done
    /// echo $sum
    /// ```
    pub fn compile_counting_loop(
        &mut self,
        loop_id: LoopId,
        count: i64,
    ) -> Result<CompiledLoopFn, String> {
        // Check cache
        if let Some(func) = self.compiled.get(&loop_id) {
            return Ok(*func);
        }

        // Check cache size limit
        if self.compiled.len() >= MAX_COMPILED_LOOPS {
            return Err("Compiled loop cache full".to_string());
        }

        // Create function signature: fn() -> i64
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::I64));

        // Create function
        let func_name = format!("loop_{}", loop_id.0);
        let func_id = self
            .module
            .declare_function(&func_name, Linkage::Local, &sig)
            .map_err(|e| e.to_string())?;

        self.ctx.func.signature = sig;
        self.ctx.func.name = UserFuncName::user(0, func_id.as_u32());

        // Build function body
        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);

            // Create entry block
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            // Create variables
            let var_sum = Variable::new(0);
            let var_i = Variable::new(1);
            builder.declare_var(var_sum, types::I64);
            builder.declare_var(var_i, types::I64);

            // Initialize: sum = 0, i = 1
            let zero = builder.ins().iconst(types::I64, 0);
            let one = builder.ins().iconst(types::I64, 1);
            let limit = builder.ins().iconst(types::I64, count);
            builder.def_var(var_sum, zero);
            builder.def_var(var_i, one);

            // Create loop blocks
            let loop_header = builder.create_block();
            let loop_body = builder.create_block();
            let loop_exit = builder.create_block();

            // Jump to loop header
            builder.ins().jump(loop_header, &[]);

            // Loop header: check condition (i <= count)
            builder.switch_to_block(loop_header);
            let i_val = builder.use_var(var_i);
            let cond = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                i_val,
                limit,
            );
            builder.ins().brif(cond, loop_body, &[], loop_exit, &[]);
            builder.seal_block(loop_header);

            // Loop body: sum += i; i++
            builder.switch_to_block(loop_body);
            let sum_val = builder.use_var(var_sum);
            let i_val = builder.use_var(var_i);
            let new_sum = builder.ins().iadd(sum_val, i_val);
            let new_i = builder.ins().iadd(i_val, one);
            builder.def_var(var_sum, new_sum);
            builder.def_var(var_i, new_i);
            builder.ins().jump(loop_header, &[]);
            builder.seal_block(loop_body);

            // Loop exit: return sum
            builder.switch_to_block(loop_exit);
            let result = builder.use_var(var_sum);
            builder.ins().return_(&[result]);
            builder.seal_block(loop_exit);

            builder.finalize();
        }

        // Compile the function
        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        self.module.clear_context(&mut self.ctx);
        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;

        // Get function pointer
        let code_ptr = self.module.get_finalized_function(func_id);
        let func: CompiledLoopFn = unsafe { std::mem::transmute(code_ptr) };

        // Cache and return
        self.compiled.insert(loop_id, func);
        Ok(func)
    }

    /// Compile a simple arithmetic expression loop
    ///
    /// This is a building block for more complex loop compilation.
    pub fn compile_arithmetic_loop(
        &mut self,
        loop_id: LoopId,
        iterations: i64,
        initial: i64,
        increment: i64,
    ) -> Result<CompiledLoopFn, String> {
        // Check cache
        if let Some(func) = self.compiled.get(&loop_id) {
            return Ok(*func);
        }

        // Check cache size limit
        if self.compiled.len() >= MAX_COMPILED_LOOPS {
            return Err("Compiled loop cache full".to_string());
        }

        // Create function signature
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::I64));

        let func_name = format!("arith_loop_{}", loop_id.0);
        let func_id = self
            .module
            .declare_function(&func_name, Linkage::Local, &sig)
            .map_err(|e| e.to_string())?;

        self.ctx.func.signature = sig;
        self.ctx.func.name = UserFuncName::user(0, func_id.as_u32());

        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            let var_acc = Variable::new(0);
            let var_i = Variable::new(1);
            builder.declare_var(var_acc, types::I64);
            builder.declare_var(var_i, types::I64);

            let init_val = builder.ins().iconst(types::I64, initial);
            let zero = builder.ins().iconst(types::I64, 0);
            let inc_val = builder.ins().iconst(types::I64, increment);
            let limit = builder.ins().iconst(types::I64, iterations);

            builder.def_var(var_acc, init_val);
            builder.def_var(var_i, zero);

            let loop_header = builder.create_block();
            let loop_body = builder.create_block();
            let loop_exit = builder.create_block();

            builder.ins().jump(loop_header, &[]);

            builder.switch_to_block(loop_header);
            let i_val = builder.use_var(var_i);
            let cond = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
                i_val,
                limit,
            );
            builder.ins().brif(cond, loop_body, &[], loop_exit, &[]);
            builder.seal_block(loop_header);

            builder.switch_to_block(loop_body);
            let acc_val = builder.use_var(var_acc);
            let i_val = builder.use_var(var_i);
            let new_acc = builder.ins().iadd(acc_val, inc_val);
            let one = builder.ins().iconst(types::I64, 1);
            let new_i = builder.ins().iadd(i_val, one);
            builder.def_var(var_acc, new_acc);
            builder.def_var(var_i, new_i);
            builder.ins().jump(loop_header, &[]);
            builder.seal_block(loop_body);

            builder.switch_to_block(loop_exit);
            let result = builder.use_var(var_acc);
            builder.ins().return_(&[result]);
            builder.seal_block(loop_exit);

            builder.finalize();
        }

        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        self.module.clear_context(&mut self.ctx);
        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;

        let code_ptr = self.module.get_finalized_function(func_id);
        let func: CompiledLoopFn = unsafe { std::mem::transmute(code_ptr) };

        self.compiled.insert(loop_id, func);
        Ok(func)
    }

    /// Get the number of compiled loops
    pub fn compiled_count(&self) -> usize {
        self.compiled.len()
    }

    /// Clear the compiled loop cache
    pub fn clear_cache(&mut self) {
        self.compiled.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_compiler_creation() {
        let compiler = JitCompiler::new();
        assert!(compiler.is_ok());
    }

    #[test]
    #[ignore = "Cranelift compilation is slow - run with --ignored"]
    fn test_counting_loop_compilation() {
        let mut compiler = JitCompiler::new().unwrap();
        let loop_id = LoopId::new();

        // Compile loop that sums 1..=10
        let func = compiler.compile_counting_loop(loop_id, 10).unwrap();

        // Execute compiled function
        let result = unsafe { func(std::ptr::null_mut(), 0) };

        // Sum of 1..=10 = 55
        assert_eq!(result, 55);
    }

    #[test]
    #[ignore = "Cranelift compilation is slow - run with --ignored"]
    fn test_counting_loop_large() {
        let mut compiler = JitCompiler::new().unwrap();
        let loop_id = LoopId::new();

        // Compile loop that sums 1..=1000
        let func = compiler.compile_counting_loop(loop_id, 1000).unwrap();
        let result = unsafe { func(std::ptr::null_mut(), 0) };

        // Sum of 1..=1000 = 500500
        assert_eq!(result, 500500);
    }

    #[test]
    #[ignore = "Cranelift compilation is slow - run with --ignored"]
    fn test_arithmetic_loop_compilation() {
        let mut compiler = JitCompiler::new().unwrap();
        let loop_id = LoopId::new();

        // initial=0, increment=5, iterations=10
        // Result should be 0 + 5*10 = 50
        let func = compiler.compile_arithmetic_loop(loop_id, 10, 0, 5).unwrap();
        let result = unsafe { func(std::ptr::null_mut(), 0) };

        assert_eq!(result, 50);
    }

    #[test]
    #[ignore = "Cranelift compilation is slow - run with --ignored"]
    fn test_compiled_function_caching() {
        let mut compiler = JitCompiler::new().unwrap();
        let loop_id = LoopId::new();

        // First compilation
        let func1 = compiler.compile_counting_loop(loop_id, 10).unwrap();

        // Second call should return cached function
        let func2 = compiler.compile_counting_loop(loop_id, 10).unwrap();

        // Same function pointer
        assert_eq!(func1 as usize, func2 as usize);
        assert_eq!(compiler.compiled_count(), 1);
    }
}
