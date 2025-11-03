use cranelift_codegen::{
    control::ControlPlane,
    ir::{types::{F64, I64}, AbiParam, InstBuilder, Signature},
    isa::{CallConv, OwnedTargetIsa, TargetIsa},
    verify_function,
};
use target_lexicon::triple;

mod translate;

extern crate libc;

use std::{io::Write, ops::{Index, IndexMut}};
use std::{ffi::c_void, mem};

unsafe extern "C" {
    fn memset(s: *mut libc::c_void, c: u32, n: libc::size_t) -> *mut libc::c_void;
}

const PAGE_SIZE: usize = 4096;

struct JitMemory {
    contents: *mut u8,
    size: usize,
    align: usize,
}

impl JitMemory {
    fn new(align: usize, size: usize) -> JitMemory {
        let contents: *mut u8;
        unsafe {
            //let size = num_pages * PAGE_SIZE;
            let mut _contents: *mut libc::c_void = 0 as *mut libc::c_void;

            // libc::mmap(
            //     &mut _contents,
            //     size,
            //     libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
            //     flags, fd, offset)
            let errno = libc::posix_memalign(&mut _contents, align, size);
            if errno != 0 {
                match errno {
                    libc::EINVAL => panic!(
                        "posix_memalign failed: alignment is not a power of two or is not a multiple of sizeof(void *)"
                    ),
                    libc::ENOMEM => panic!("posix_memalign failed: insufficient memory"),
                    _ => panic!("posix_memalign failed with error code {}", errno),
                }
            }
            if _contents.is_null() {
                panic!("posix_memalign returned null pointer");
            }
            // println!("ptr value {:x}", _contents as usize);
            // let err = libc::mprotect(_contents, size, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);
            // if err != 0 {
            //     let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(-1) ;
            //     match err {
            //         libc::EACCES => panic!("mprotect failed: the requested access cannot be granted"),
            //         libc::EINVAL => panic!("mprotect failed: the value of prot is invalid or length is zero"),
            //         libc::ENOMEM => panic!("mprotect failed: the memory range specified is not currently mapped"),
            //         _ => panic!("mprotect failed with error code {}", err),
            //     }
            // }

            //memset(_contents, 0xc3, size);  // for now, prepopulate with 'RET'

            contents = mem::transmute(_contents);
        }

        JitMemory {
            contents: contents,
            size,
            align,
        }
    }

    fn enable_exec(&self) {
        unsafe {
            let err = libc::mprotect(
                self.contents as *mut libc::c_void,
                self.size,
                libc::PROT_EXEC,
            );
            if err != 0 {
                let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(-1);
                match err {
                    libc::EACCES => {
                        panic!("mprotect failed: the requested access cannot be granted")
                    }
                    libc::EINVAL => {
                        panic!("mprotect failed: the value of prot is invalid or length is zero")
                    }
                    libc::ENOMEM => panic!(
                        "mprotect failed: the memory range specified is not currently mapped"
                    ),
                    _ => panic!("mprotect failed with error code {}", err),
                }
            }
        }
    }

    fn copy(&mut self, code: &[u8]) {
        unsafe {
            for i in 0..code.len() {
                *self.contents.offset(i as isize) = code[i];
            }
        }
    }

    fn drop(self) {
        unsafe {
            libc::free(self.contents as *mut libc::c_void);
        }
    }
}

impl Index<usize> for JitMemory {
    type Output = u8;

    fn index(&self, _index: usize) -> &u8 {
        unsafe { &*self.contents.offset(_index as isize) }
    }
}

impl IndexMut<usize> for JitMemory {
    fn index_mut(&mut self, _index: usize) -> &mut u8 {
        unsafe { &mut *self.contents.offset(_index as isize) }
    }
}

pub struct JitContext {
    isa: OwnedTargetIsa,
}

impl JitContext {
    fn new() -> Self {
        let shared_builder = cranelift_codegen::settings::builder();
        let shared_flags = cranelift_codegen::settings::Flags::new(shared_builder);
        let triple = triple!("arm64-apple-macosx");
        let isa = cranelift_codegen::isa::lookup(triple)
            .unwrap()
            .finish(shared_flags)
            .unwrap();
        Self { isa}
    }

    fn isa(&self) -> &dyn TargetIsa {
        &*self.isa
    }

    fn translate_type(&self, ty: &crate::types::Type) -> cranelift_codegen::ir::Type {
        match ty {
            crate::types::Type::Integer => I64,
            crate::types::Type::Number => F64,
            _ => cranelift_codegen::ir::Type::triple_pointer_type(self.isa().triple()), // pointer?
        }
    }
}


extern "C" fn callee() {
    println!("In callee!");
}

pub(crate) fn start(module: &crate::ir::Module) {
    let ctx = JitContext::new();
    
    print!("translating... "); std::io::stdout().flush().unwrap();
    let translated = module.funcs.iter().map(|func|
        translate::translate_function(&ctx, &func)).collect::<Vec<_>>();
    println!("done.");

    print!("compiling... "); std::io::stdout().flush().unwrap();
    let mut context = cranelift_codegen::Context::for_function(translated[0].clone());

    let res = context
        .compile(ctx.isa(), &mut ControlPlane::default())
        .unwrap();
    let alignment = res.buffer.alignment as u64;
    let compiled_code = context.compiled_code().unwrap();

    let size = compiled_code.code_info().total_size as usize;
    let align = alignment
        .max(ctx.isa().function_alignment().minimum as u64)
        .max(ctx.isa().symbol_alignment())
        .max(size_of::<*mut c_void>() as u64);

    let code = compiled_code.buffer.data();
    println!("done. code size: {}, alignment: {}", size, align);

    print!("copying to exec memory... "); std::io::stdout().flush().unwrap();
    let mut jit_memory = JitMemory::new(align as usize, size);
    jit_memory.copy(code);
    jit_memory.enable_exec();
    println!("done.");

    println!("attempting to run...\n");
    unsafe {
        let compiled_func = core::mem::transmute::<_, fn() -> i64>(jit_memory.contents);
        //println!("ptr value {}", compiled_func as usize);
        let ret = compiled_func();
        println!("\nResult: {}", ret);
    }
    println!("finished.");
    //jit_memory.drop();

}

pub(crate) fn run() {
    let shared_builder = cranelift_codegen::settings::builder();
    let shared_flags = cranelift_codegen::settings::Flags::new(shared_builder);
    let triple = triple!("arm64-apple-macosx");
    let isa = cranelift_codegen::isa::lookup(triple)
        .unwrap()
        .finish(shared_flags)
        .unwrap();
    println!("Using triple: {}", isa.triple().to_string());

    let mut sig = Signature::new(CallConv::SystemV);
    sig.returns.push(AbiParam::new(I64));

    let mut func = cranelift_codegen::ir::Function::with_name_signature(
        cranelift_codegen::ir::UserFuncName::user(0, 0),
        sig,
    );

    let mut func_bld_ctx = cranelift_frontend::FunctionBuilderContext::new();
    let mut builder = cranelift_frontend::FunctionBuilder::new(&mut func, &mut func_bld_ctx);

    let sig = builder.import_signature(Signature {
        params: Vec::new(),
        returns: Vec::new(),
        call_conv: CallConv::AppleAarch64,
    });

    let block = builder.create_block();
    builder.switch_to_block(block);
    // translate calle into a function pointer
    //    let ptr = &callee;

    let func_address_as_u64: i64 = callee as *const () as i64;
    let ptr = builder.ins().iconst(I64, func_address_as_u64);
    //let callee_value = builder.ins().iconst(I64, (&callee as *fn() -> ()) as u64);
    let _ = builder.ins().call_indirect(sig, ptr, &[]);
    let tmp = builder.ins().iconst(I64, 420);
    //builder.use_var(var)
    builder.ins().return_(&[tmp]);

    builder.seal_all_blocks();
    builder.finalize();

    let res = verify_function(&func, &*isa);

    println!("{}", func.display());
    if let Err(errors) = res {
        panic!("{}", errors);
    }

    let mut context = cranelift_codegen::Context::for_function(func);

    let res = context
        .compile(&*isa, &mut ControlPlane::default())
        .unwrap();
    let alignment = res.buffer.alignment as u64;
    let compiled_code = context.compiled_code().unwrap();

    let size = compiled_code.code_info().total_size as usize;
    let align = alignment
        .max(isa.function_alignment().minimum as u64)
        .max(isa.symbol_alignment())
        .max(size_of::<*mut c_void>() as u64);

    let code = compiled_code.buffer.data();
    println!("{:?}", compiled_code.buffer.relocs());

    println!("Code size: {}, Alignment: {}", size, align);

    let mut jit_memory = JitMemory::new(align as usize, size);
    jit_memory.copy(code);
    jit_memory.enable_exec();
    unsafe {
        let compiled_func = core::mem::transmute::<_, fn() -> i64>(jit_memory.contents);
        println!("ptr value {}", compiled_func as usize);
        let value = compiled_func();
        println!("Result: {}", value);
    }
}
