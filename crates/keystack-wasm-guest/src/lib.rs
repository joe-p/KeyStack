use std::path::PathBuf;

pub struct PreProcessorGuestContext {
    pub user: String,
    pub key_path: PathBuf,
    pub action_id: String,
    pub payload: Vec<u8>,
}

/// Allocate memory in the guest WASM module.
/// The host calls this to allocate space for context data.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, 1).unwrap();
    unsafe { std::alloc::alloc(layout) }
}

/// Deallocate memory in the guest WASM module.
/// The host calls this to free memory allocated by alloc.
///
/// # Safety
/// The caller must ensure that the pointer was allocated by `alloc` and that the size matches
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    if !ptr.is_null() {
        let layout = std::alloc::Layout::from_size_align(size, 1).unwrap();
        unsafe { std::alloc::dealloc(ptr, layout) }
    }
}
