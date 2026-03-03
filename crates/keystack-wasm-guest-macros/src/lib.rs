use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, ItemFn, ReturnType, parse_macro_input};

/// Marks a function as the pre-action hook for a keystack WASM context provider.
///
/// The marked function will receive a `ContextProviderGuestContext` containing
/// the user, key_path, action_id, and payload as convenient Rust types.
/// It should return a `Vec<u8>` containing the result data.
///
/// The macro generates the necessary FFI boilerplate to convert between the
/// host's pointer/length arguments and Rust types, and to properly encode
/// the return value with its length prefix.
///
/// # Example
///
/// ```rust,ignore
/// use keystack_wasm_guest::pre_action_hook;
/// use keystack_wasm_guest::ContextProviderGuestContext;
///
/// #[pre_action_hook]
/// fn my_hook(context: ContextProviderGuestContext) -> Vec<u8> {
///     // Access context fields
///     println!("User: {}", context.user);
///     println!("Key path: {:?}", context.key_path);
///     println!("Action ID: {}", context.action_id);
///     println!("Payload: {:?}", context.payload);
///
///     // Return result data
///     vec![0x01, 0x02, 0x03, 0x04]
/// }
/// ```
#[proc_macro_attribute]
pub fn pre_action_hook(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    let fn_vis = &input_fn.vis;

    // Create internal function name to avoid conflict with the extern "C" export
    let internal_fn_name = syn::Ident::new(&format!("__keystack_{}", fn_name), fn_name.span());

    let inputs = &input_fn.sig.inputs;
    if inputs.len() != 1 {
        return syn::Error::new(
            input_fn.sig.ident.span(),
            "#[pre_action_hook] function must take exactly one argument of type ContextProviderGuestContext",
        )
        .to_compile_error()
        .into();
    }

    let context_arg = match &inputs[0] {
        FnArg::Receiver(_) => {
            return syn::Error::new(
                inputs[0].span(),
                "#[pre_action_hook] function must take ContextProviderGuestContext, not self",
            )
            .to_compile_error()
            .into();
        }
        FnArg::Typed(pat_type) => match &*pat_type.pat {
            syn::Pat::Ident(pat_ident) => &pat_ident.ident,
            _ => {
                return syn::Error::new(
                    pat_type.pat.span(),
                    "#[pre_action_hook] function argument must be a simple identifier",
                )
                .to_compile_error()
                .into();
            }
        },
    };

    let return_type_ok = match &input_fn.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => {
            // Check if it looks like Vec<u8> - this is a basic check
            let type_str = quote!(#ty).to_string();
            type_str.contains("Vec") && type_str.contains("u8")
        }
    };

    if !return_type_ok {
        return syn::Error::new(
            input_fn.sig.span(),
            "#[pre_action_hook] function must return Vec<u8>",
        )
        .to_compile_error()
        .into();
    }

    // Generate the extern "C" wrapper function
    let expanded = quote! {
        // Keep the user's function with its original name
        #fn_vis fn #fn_name(#context_arg: keystack_wasm_guest::ContextProviderGuestContext) -> Vec<u8> {
            #fn_block
        }

        /// The FFI entry point called by the host.
        /// This function converts raw pointers/lengths to Rust types,
        /// calls the user-defined hook function, and encodes the result.
        ///
        /// # Safety
        /// This function assumes the host has provided valid pointers and lengths
        /// for UTF-8 strings and binary data.
        #[unsafe(export_name = "pre_action_hook")]
        extern "C" fn #internal_fn_name(
            user_ptr: *const u8,
            user_len: usize,
            key_path_ptr: *const u8,
            key_path_len: usize,
            action_id_ptr: *const u8,
            action_id_len: usize,
            payload_ptr: *const u8,
            payload_len: usize,
        ) -> *const u8 {
            // Create context from raw parts
            let #context_arg = unsafe {
                keystack_wasm_guest::ContextProviderGuestContext::from_parts(
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

            let result_data = #fn_name(#context_arg);

            // Encode the result with a length prefix (little-endian i32)
            let result_len = result_data.len() as i32;
            let result_vec = [
                result_len.to_le_bytes().as_ref(),
                result_data.as_slice(),
            ]
            .concat();

            let ptr = result_vec.as_ptr();
            std::mem::forget(result_vec);
            ptr
        }
    };

    TokenStream::from(expanded)
}
