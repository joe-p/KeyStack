use keystack_wasm_guest::ContextProviderGuestContext;
pub use keystack_wasm_guest::alloc;

/// # Safety
/// This function assumes the host has provided valid pointers and lengths.
#[unsafe(no_mangle)]
extern "C" fn pre_action_hook(
    user_ptr: *const u8,
    user_len: usize,
    key_path_ptr: *const u8,
    key_path_len: usize,
    action_id_ptr: *const u8,
    action_id_len: usize,
    payload_ptr: *const u8,
    payload_len: usize,
) -> *const u8 {
    let _context = unsafe {
        ContextProviderGuestContext::from_parts(
            user_ptr,
            user_len,
            key_path_ptr,
            key_path_len,
            action_id_ptr,
            action_id_len,
            payload_ptr,
            payload_len,
        )
    };

    let result_data = vec![0x01u8, 0x02, 0x03, 0x04];
    let result_len = result_data.len() as i32;

    let result_vec = [result_len.to_le_bytes().as_ref(), result_data.as_slice()].concat();

    result_vec.as_ptr()
}
