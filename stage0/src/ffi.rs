use std::ffi::c_char;

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
    pub fn llvm_block_new(cx: *mut LlvmContext) -> *mut LlvmBlock;
    pub fn llvm_block_dispose(bb: *mut LlvmBlock);
    pub fn llvm_builder_new(cx: *mut LlvmContext) -> *mut LlvmBuilder;
    pub fn llvm_builder_dispose(ib: *mut LlvmBuilder);
    pub fn llvm_builder_append_block(ib: *mut LlvmBuilder, bb: *mut LlvmBlock);
    pub fn llvm_builder_ret_void(ib: *mut LlvmBuilder) -> *mut LlvmReturn;
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
pub struct LlvmFunction(());
pub struct LlvmBlock(());
pub struct LlvmBuilder(());
pub struct LlvmReturn(());
