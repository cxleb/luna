use std::collections::HashMap;

use cranelift_codegen::{ir::types::{F64, I8, I64}, isa::{OwnedTargetIsa, TargetIsa}, settings::Configurable};
//use cranelift_jit::{JITBuilder, JITModule};
//use cranelift_module::{FuncId, Module};
use target_lexicon::Triple;

use crate::{builtins::Builtins, ir::Signature, runtime::{gc::GarbageCollector, translate::TranslateSignature}};

mod translate;
mod gc;
pub mod string;
mod cranelift;
mod stack_roots;

use cranelift::backend::{JITBuilder, JITModule};
use cranelift::module::{FuncId, Module};

pub struct CompiledFunc {
    id: FuncId,
    name: String,
    code: *const u8,
}

#[derive(Debug, Clone)]
pub struct StackMap {
    pub map: Vec<u32>,
    pub frame_to_fp_offset: usize,
}

#[derive(Debug, Clone)]
pub struct StackMaps {
    pub lr_map: HashMap<usize, StackMap>
}

pub struct RuntimeContext {
    pub gc: GarbageCollector,
    pub stack_maps: StackMaps,
}

impl RuntimeContext {
    pub fn new() -> Self {
        Self {
            gc: GarbageCollector::new(),
            stack_maps: StackMaps {
                lr_map: HashMap::new(),
            },
        }
    }
}

struct Fiber {
    ctx: *mut RuntimeContext,
    entry_point: *const u8,
}

pub extern "C" fn panic(_: *mut RuntimeContext) {
    panic!("Temporary Panic Handler!");
}

pub extern "C" fn create_array(ctx: *mut RuntimeContext, size: i64) -> *const i64 {
    let gc = unsafe { &mut (*ctx).gc };
    let array = gc.create_array(size as usize);
    array
}

pub extern "C" fn create_object(ctx: *mut RuntimeContext, size: i64) -> *const i64 {
    let gc = unsafe { &mut (*ctx).gc };
    let array = gc.create_object(size as usize);
    array
}

#[cfg(target_arch = "aarch64")]
pub extern "C" fn check_yield(ctx: *mut RuntimeContext) {
    let mut fp: usize;
    unsafe {
        core::arch::asm!("mov {}, fp", out(reg) fp);
    }

    check_yield_common(ctx, fp);
}

#[cfg(target_arch = "x86_64")]
pub extern "C" fn check_yield(ctx: *mut RuntimeContext) {
    let mut fp: usize;
    unsafe {
        core::arch::asm!("mov {}, rbp", out(reg) fp);
    }
    
    check_yield_common(ctx, fp);
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
pub extern "C" fn check_yield(ctx: *mut RuntimeContext) {
}

fn check_yield_common(ctx: *mut RuntimeContext, fp: usize) {
    let gc = unsafe { &mut (*ctx).gc };
    
    if gc.should_collect() {
        let stack_maps = unsafe { &mut (*ctx).stack_maps };
        let roots = stack_roots::collect_roots(stack_maps, fp);
        gc.collect(&roots);
    }

    // perform fiber switch if we need ?
}
 
// Entry point into a task.
pub extern "C" fn fiber_entry(t: context::Transfer) -> ! {    
    let fiber = unsafe { &mut *(t.data as *mut Fiber) };

    unsafe {
        let code_fn = core::mem::transmute::<_, fn(*mut RuntimeContext)>(fiber.entry_point);
        code_fn(fiber.ctx);
    }

    unsafe {
        t.context.resume(0);
    }

    loop {
        check_yield(fiber.ctx);
    }
}

pub struct JitContext {
    isa: OwnedTargetIsa,
    module: JITModule,
    builtins: Builtins,
    compiled_funcs: Vec<CompiledFunc>,
    runtime_ctx: *mut RuntimeContext,
}

impl JitContext {
    pub fn new(builtins: Builtins) -> Self {
        let mut shared_builder = cranelift_codegen::settings::builder();
        shared_builder
            .set("preserve_frame_pointers", "true")
            .expect("failed to enable frame pointers for stack-root walking");
        let shared_flags = cranelift_codegen::settings::Flags::new(shared_builder);
        let triple = Triple::host();
        let isa = cranelift_codegen::isa::lookup(triple)
            .unwrap()
            .finish(shared_flags)
            .unwrap();

        let mut builder = 
            JITBuilder::new(cranelift::default_libcall_names())
            .unwrap();

        // add the builtins as look up symbols
        for func in &builtins.functions {
            builder.symbol(format!("_Lbuiltins_{}", func.id), func.implementation);
        }

        builder.symbol("__panic", panic as *const u8);
        builder.symbol("__create_array", create_array as *const u8);
        builder.symbol("__create_object", create_object as *const u8);
        builder.symbol("__check_yield", check_yield as *const u8);

        let runtime_ctx = Box::new(RuntimeContext::new());
        let runtime_ctx = Box::into_raw(runtime_ctx);
        
        Self {
            isa,
            module: JITModule::new(builder),
            compiled_funcs: Vec::new(),
            builtins,
            runtime_ctx,
        }
    }

    fn isa(&self) -> &dyn TargetIsa {
        &*self.isa
    }

    fn translate_type(&self, ty: &crate::types::Type) -> cranelift_codegen::ir::Type {
        match ty.kind() {
            crate::types::TypeKind::Integer => I64,
            crate::types::TypeKind::Bool => I8,
            crate::types::TypeKind::Number => F64,

            _ => cranelift_codegen::ir::Type::triple_pointer_type(self.isa().triple()), // pointer?
        }
    }

    fn context_type(&self) -> cranelift_codegen::ir::Type {
        cranelift_codegen::ir::Type::triple_pointer_type(self.isa().triple())
    }

    pub fn compile_ir_module(&mut self, module: &crate::ir::Module) {
        let mut context = self.module.make_context();
        //context.set_disasm(true);
        let mut signatures = Vec::new();
        for func in module.funcs.iter() {
            signatures.push(TranslateSignature {
                id: func.id.clone(),
                signature: func.signature.clone()
            });
        }
        for func in self.builtins.functions.iter() {
            signatures.push(TranslateSignature {
                id: format!("_Lbuiltins_{}", func.id),
                signature: Signature {
                    parameters: func.parameters.clone(),
                    ret_types: match &func.returns {
                        Some(ret) => vec![ret.clone()],
                        None => vec![]
                    }
                }
            });
        }


        signatures.push(TranslateSignature {
            id: "__panic".into(),
            signature: Signature {
                parameters: vec![],
                ret_types: vec![],
            }
        });
        signatures.push(TranslateSignature {
            id: "__create_array".into(),
            signature: Signature {
                parameters: vec![crate::types::integer()],
                ret_types: vec![crate::types::unknown_reference()],
            }
        });
        signatures.push(TranslateSignature {
            id: "__create_object".into(),
            signature: Signature {
                parameters: vec![crate::types::integer()],
                ret_types: vec![crate::types::unknown_reference()],
            }
        });
        signatures.push(TranslateSignature {
            id: "__check_yield".into(),
            signature: Signature {
                parameters: vec![],
                ret_types: vec![],
            }
        });

        let translated = module.funcs.iter().map(|func| {
            translate::translate_function(self, &func, &mut context, &signatures, &module.string_map);
            let id = self.module.declare_function(&func.id, cranelift::module::Linkage::Local, &context.func.signature).unwrap();
            self.module.define_function(id, &mut context).unwrap();
            self.module.clear_context(&mut context);
            (id, func.id.clone())
        }).collect::<Vec<_>>();

        self.module.finalize_definitions().unwrap();

        for (id, name) in translated {
            let blob = self.module.get_finalized_function(id);
            self.compiled_funcs.push(CompiledFunc { 
                id, 
                name, 
                code: blob.ptr
            });

            for stack_map in blob.stack_maps.iter() {
                unsafe { 
                    (*self.runtime_ctx) 
                        .stack_maps
                        .lr_map
                        .insert(
                            blob.ptr as usize + stack_map.offset as usize, 
                            StackMap { map: stack_map.map.clone(), frame_to_fp_offset: blob.frame_to_fp_offset as usize }
                        ); 
                }
            }

        }
    }

    pub fn call_function_no_params_no_return(&self, name: &str) {
        let compiled_func = self.compiled_funcs.iter().find(|c| c.name == name).unwrap();
        let fiber = Box::new(Fiber {
            ctx: self.runtime_ctx,
            entry_point: compiled_func.code,
        });
        let fiber = Box::into_raw(fiber);

        unsafe {
            let s = context::stack::ProtectedFixedSizeStack::new(1024 * 1024).unwrap();
            let t = context::Transfer::new(context::Context::new(&s, fiber_entry), 0);
            t.context.resume(fiber as usize);
        }

        //unsafe {
        //    let code_fn = core::mem::transmute::<_, fn(*mut RuntimeContext)>(compiled_func.code);
        //    code_fn(self.runtime_ctx);
        //}    
    }

    pub fn call_function_no_params<Returns>(&self, name: &str) -> Returns {
        let compiled_func = self.compiled_funcs.iter().find(|c| c.name == name).unwrap();
        unsafe {
            let code_fn = core::mem::transmute::<_, fn(*mut RuntimeContext) -> Returns>(compiled_func.code);
            code_fn(self.runtime_ctx)
        }    
    }
}
