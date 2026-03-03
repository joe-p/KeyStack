use std::path::PathBuf;

pub struct ContextProviderGuestContext {
    pub user: String,
    pub key_path: PathBuf,
    pub action_id: String,
    pub payload: Vec<u8>,
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: u32) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }

    let layout = match std::alloc::Layout::array::<u8>(len as usize) {
        Ok(layout) => layout,
        Err(_) => return std::ptr::null_mut(),
    };

    // SAFETY: layout is non-zero sized (len > 0 checked above)
    unsafe { std::alloc::alloc(layout) }
}

/// # Safety
/// `ptr` must have been allocated by `alloc` with the same `len`, and not already deallocated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: u32) {
    if ptr.is_null() || len == 0 {
        return;
    }

    let layout = match std::alloc::Layout::array::<u8>(len as usize) {
        Ok(layout) => layout,
        Err(_) => return,
    };

    // SAFETY: ptr was allocated with the same layout via `alloc`
    unsafe { std::alloc::dealloc(ptr, layout) }
}
