use std::slice;

use keystack_wasm_guest::ContextProviderGuestContext;
pub use keystack_wasm_guest::alloc;

/// # Safety
/// This function assumes the host has provided valid pointers and lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pre_action_hook(
    user_ptr: *const u8,
    user_len: usize,
    key_path_ptr: *const u8,
    key_path_len: usize,
    action_id_ptr: *const u8,
    action_id_len: usize,
    payload_ptr: *const u8,
    payload_len: usize,
) -> *const u8 {
    let user =
        unsafe { String::from_utf8_lossy(slice::from_raw_parts(user_ptr, user_len)).into_owned() };
    let key_path = unsafe {
        String::from_utf8_lossy(slice::from_raw_parts(key_path_ptr, key_path_len)).into_owned()
    };
    let action_id = unsafe {
        String::from_utf8_lossy(slice::from_raw_parts(action_id_ptr, action_id_len)).into_owned()
    };
    let payload = unsafe { slice::from_raw_parts(payload_ptr, payload_len).to_vec() };

    // For now, just return dummy data [0x01, 0x02, 0x03, 0x04]
    let _context = ContextProviderGuestContext {
        user,
        key_path: key_path.into(),
        action_id,
        payload,
    };

    let result_data = vec![0x01u8, 0x02, 0x03, 0x04];
    let result_len = result_data.len() as i32;

    let result_vec = [result_len.to_le_bytes().as_ref(), result_data.as_slice()].concat();

    result_vec.as_ptr()
}
