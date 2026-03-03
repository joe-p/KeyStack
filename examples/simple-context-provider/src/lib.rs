use keystack_wasm_guest::pre_action_hook;

#[pre_action_hook]
fn pre_action_hook(context: ContextProviderGuestContext) -> Vec<u8> {
    let _user = context.user;
    let _key_path = context.key_path;
    let _action_id = context.action_id;
    let _payload = context.payload;

    // Return test data
    vec![0x01u8, 0x02, 0x03, 0x04]
}
