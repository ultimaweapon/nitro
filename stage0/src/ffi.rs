use std::ffi::c_char;

#[allow(improper_ctypes)]
extern "C" {
    pub fn llvm_init();
    pub fn llvm_target_lookup(triple: *const c_char, err: &mut String) -> *const LlvmTarget;
    pub fn llvm_target_create_machine(
        target: *const LlvmTarget,
        triple: *const c_char,
        cpu: *const c_char,
        features: *const c_char,
    ) -> *mut LlvmMachine;
    pub fn llvm_target_dispose_machine(mc: *mut LlvmMachine);
    pub fn llvm_target_emit_object(
        mc: *mut LlvmMachine,
        md: *mut LlvmModule,
        file: *const c_char,
        err: &mut String,
    ) -> bool;
    pub fn llvm_layout_new(mc: *const LlvmMachine) -> *mut LlvmLayout;
    pub fn llvm_layout_dispose(dl: *mut LlvmLayout);
    pub fn llvm_layout_pointer_size(dl: *const LlvmLayout) -> u32;
    pub fn llvm_context_new() -> *mut LlvmContext;
    pub fn llvm_context_dispose(cx: *mut LlvmContext);
    pub fn llvm_module_new(cx: *mut LlvmContext, id: *const c_char) -> *mut LlvmModule;
    pub fn llvm_module_dispose(md: *mut LlvmModule);
    pub fn llvm_module_set_layout(md: *mut LlvmModule, dl: *const LlvmLayout);
    pub fn llvm_module_get_function(
        md: *const LlvmModule,
        name: *const c_char,
    ) -> *mut LlvmFunction;
    pub fn llvm_type_void(cx: *mut LlvmContext) -> *mut LlvmType;
    pub fn llvm_type_int8(cx: *mut LlvmContext) -> *mut LlvmInteger;
    pub fn llvm_type_int32(cx: *mut LlvmContext) -> *mut LlvmInteger;
    pub fn llvm_type_int64(cx: *mut LlvmContext) -> *mut LlvmInteger;
    pub fn llvm_type_ptr(cx: *mut LlvmContext) -> *mut LlvmPointer;
    pub fn llvm_type_func(
        ret: *mut LlvmType,
        params: *const *mut LlvmType,
        count: usize,
        va: bool,
    ) -> *mut LlvmPrototype;
    pub fn llvm_function_new(
        md: *mut LlvmModule,
        ty: *mut LlvmPrototype,
        name: *const c_char,
    ) -> *mut LlvmFunction;
    pub fn llvm_function_append(f: *mut LlvmFunction, bb: *mut LlvmBlock);
    pub fn llvm_function_set_stdcall(f: *mut LlvmFunction);
    pub fn llvm_function_set_noreturn(f: *mut LlvmFunction);
    pub fn llvm_integer_const(ty: *mut LlvmInteger, val: u64, sign: bool) -> *mut LlvmConstInt;
    pub fn llvm_block_new(cx: *mut LlvmContext) -> *mut LlvmBlock;
    pub fn llvm_block_dispose(bb: *mut LlvmBlock);
    pub fn llvm_builder_new(cx: *mut LlvmContext) -> *mut LlvmBuilder;
    pub fn llvm_builder_dispose(ib: *mut LlvmBuilder);
    pub fn llvm_builder_append_block(ib: *mut LlvmBuilder, bb: *mut LlvmBlock);
    pub fn llvm_builder_call(
        ib: *mut LlvmBuilder,
        func: *mut LlvmFunction,
        args: *const *mut LlvmValue,
        nargs: usize,
    ) -> *mut LlvmCall;
    pub fn llvm_builder_ret_void(ib: *mut LlvmBuilder) -> *mut LlvmReturn;
    pub fn llvm_builder_ret(ib: *mut LlvmBuilder, v: *mut LlvmValue) -> *mut LlvmReturn;
    pub fn ZSTD_createCStream() -> *mut ZstdContex;
    pub fn ZSTD_freeCStream(zcs: *mut ZstdContex) -> usize;
    pub fn ZSTD_compressStream2(
        cctx: *mut ZstdContex,
        output: *mut ZSTD_outBuffer,
        input: *mut ZSTD_inBuffer,
        endOp: ZSTD_EndDirective,
    ) -> usize;
    pub fn ZSTD_CStreamInSize() -> usize;
    pub fn ZSTD_CStreamOutSize() -> usize;
    pub fn ZSTD_isError(code: usize) -> u32;
    pub fn ZSTD_getErrorName(code: usize) -> *const c_char;
}

pub struct LlvmTarget(());
pub struct LlvmMachine(());
pub struct LlvmLayout(());
pub struct LlvmContext(());
pub struct LlvmModule(());
pub struct LlvmType(());
pub struct LlvmInteger(());
pub struct LlvmPointer(());
pub struct LlvmPrototype(());
pub struct LlvmValue(());
pub struct LlvmFunction(());
pub struct LlvmConstInt(());
pub struct LlvmBlock(());
pub struct LlvmBuilder(());
pub struct LlvmCall(());
pub struct LlvmReturn(());
pub struct ZstdContex(());

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct ZSTD_inBuffer {
    pub src: *const u8,
    pub size: usize,
    pub pos: usize,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct ZSTD_outBuffer {
    pub dst: *mut u8,
    pub size: usize,
    pub pos: usize,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum ZSTD_EndDirective {
    ZSTD_e_continue = 0,
    ZSTD_e_end = 2,
}
