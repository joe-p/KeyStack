use std::path::PathBuf;

pub struct ContextProviderGuestContext {
    pub user: String,
    pub key_path: PathBuf,
    pub action_id: String,
    pub payload: Vec<u8>,
}

impl ContextProviderGuestContext {
    /// # Safety
    /// This function assumes the host has provided valid pointers and lengths for UTF-8 strings and
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn from_parts(
        user_ptr: *const u8,
        user_len: usize,
        key_path_ptr: *const u8,
        key_path_len: usize,
        action_id_ptr: *const u8,
        action_id_len: usize,
        payload_ptr: *const u8,
        payload_len: usize,
    ) -> Self {
        let user = unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(user_ptr, user_len)).into_owned()
        };
        let key_path = unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(key_path_ptr, key_path_len))
                .into_owned()
        };
        let action_id = unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(action_id_ptr, action_id_len))
                .into_owned()
        };
        let payload = unsafe { std::slice::from_raw_parts(payload_ptr, payload_len).to_vec() };

        Self {
            user,
            key_path: key_path.into(),
            action_id,
            payload,
        }
    }
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
