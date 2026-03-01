use std::path::PathBuf;

pub struct ContextProviderGuestContext {
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
