use keystack_wasm_guest::alloc as guest_alloc;
use std::slice;

/// Allocate memory in the guest WASM module.
/// The host calls this to allocate space for context data.
///
/// # Safety
/// This function leaks memory (by design for WASM guests). The host
/// is responsible for managing the guest's memory lifecycle.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: u32) -> *mut u8 {
    guest_alloc(len)
}

/// Context structure passed from host to guest.
///
/// This struct is used by the host to serialize context data
/// before passing it to the guest.
pub struct ContextProviderGuestContext {
    pub user: String,
    pub key_path: String,
    pub action_id: String,
    pub payload: Vec<u8>,
}

#[unsafe(no_mangle)]
pub extern "C" fn pre_action_hook(
    user_ptr: i32,
    user_len: i32,
    key_path_ptr: i32,
    key_path_len: i32,
    action_id_ptr: i32,
    action_id_len: i32,
    payload_ptr: i32,
    payload_len: i32,
) -> *const u8 {
    // SAFETY: Host guarantees valid pointers and lengths
    let user = unsafe {
        String::from_utf8_lossy(slice::from_raw_parts(
            user_ptr as *const u8,
            user_len as usize,
        ))
        .into_owned()
    };
    let key_path = unsafe {
        String::from_utf8_lossy(slice::from_raw_parts(
            key_path_ptr as *const u8,
            key_path_len as usize,
        ))
        .into_owned()
    };
    let action_id = unsafe {
        String::from_utf8_lossy(slice::from_raw_parts(
            action_id_ptr as *const u8,
            action_id_len as usize,
        ))
        .into_owned()
    };
    let payload =
        unsafe { slice::from_raw_parts(payload_ptr as *const u8, payload_len as usize).to_vec() };

    // For now, just return dummy data [0x01, 0x02, 0x03, 0x04]
    let _context = ContextProviderGuestContext {
        user,
        key_path,
        action_id,
        payload,
    };

    let result_data = vec![0x01u8, 0x02, 0x03, 0x04];
    let result_len = result_data.len() as i32;

    let result_vec = [result_len.to_le_bytes().as_ref(), result_data.as_slice()].concat();

    result_vec.as_ptr()
}
