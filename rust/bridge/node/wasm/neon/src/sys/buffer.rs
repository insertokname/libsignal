#[cfg(feature = "external-buffers")]
use std::os::raw::c_void;
use std::{mem::MaybeUninit, slice};

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn new(env: Env, len: usize) -> Result<Local, napi::Status> {
    let (buf, bytes) = uninitialized(env, len)?;

    std::ptr::write_bytes(bytes, 0, len);

    Ok(buf)
}

pub unsafe fn uninitialized(env: Env, len: usize) -> Result<(Local, *mut u8), napi::Status> {
    let mut buf = MaybeUninit::uninit();
    let mut bytes = MaybeUninit::uninit();
    let status = napi::create_buffer(env, len, bytes.as_mut_ptr(), buf.as_mut_ptr());

    match status {
        Err(err @ napi::Status::PendingException) => return Err(err),
        status => status.unwrap(),
    };

    Ok((buf.assume_init(), bytes.assume_init().cast()))
}

#[cfg(feature = "external-buffers")]
pub unsafe fn new_external<T>(env: Env, data: T) -> Local
where
    T: AsMut<[u8]> + Send,
{
    // Safety: Boxing could move the data; must box before grabbing a raw pointer
    let mut data = Box::new(data);
    let buf = data.as_mut().as_mut();
    let length = buf.len();
    let mut result = MaybeUninit::uninit();

    napi::create_external_buffer(
        env,
        length,
        buf.as_mut_ptr() as *mut _,
        Some(drop_external::<T>),
        Box::into_raw(data) as *mut _,
        result.as_mut_ptr(),
    )
    .unwrap();

    result.assume_init()
}

#[cfg(feature = "external-buffers")]
unsafe extern "C" fn drop_external<T>(_env: Env, _data: *mut c_void, hint: *mut c_void) {
    drop(Box::<T>::from_raw(hint as *mut _));
}

/// # Safety
/// * Caller must ensure `env` and `buf` are valid
/// * The lifetime `'a` does not exceed the lifetime of `Env` or `buf`
pub unsafe fn as_mut_slice<'a>(env: Env, buf: Local) -> &'a mut [u8] {
    let mut data = MaybeUninit::uninit();
    let mut size = 0usize;

    napi::get_buffer_info(env, buf, data.as_mut_ptr(), &mut size as *mut _).unwrap();

    if size == 0 {
        return &mut [];
    }

    slice::from_raw_parts_mut(data.assume_init().cast(), size)
}

/// # Safety
/// * Caller must ensure `env` and `buf` are valid
pub unsafe fn size(env: Env, buf: Local) -> usize {
    let mut data = MaybeUninit::uninit();
    let mut size = 0usize;

    napi::get_buffer_info(env, buf, data.as_mut_ptr(), &mut size as *mut _).unwrap();

    size
}

/// WEB-FORK: under emnapi, a JavaScript buffer's bytes are mirrored in wasm
/// memory; after writing to the mirror they must be copied back to JavaScript.
/// Neon's `from_slice` constructors do this; other writes through
/// `as_mut_slice` are not reflected on wasm.
///
/// # Safety
/// `env` must be valid for the current thread and `value` must be an
/// `ArrayBuffer` or an `ArrayBufferView`.
#[cfg(target_family = "wasm")]
pub unsafe fn sync_to_js(env: Env, value: Local, len: usize) {
    #[link(wasm_import_module = "emnapi")]
    extern "C" {
        fn emnapi_sync_memory(
            env: Env,
            js_to_wasm: bool,
            arraybuffer_or_view: *mut Local,
            offset: usize,
            len: usize,
        ) -> napi::Status;
    }

    // An empty buffer has no mirror (and emnapi rejects syncing it).
    if len == 0 {
        return;
    }
    let mut value = value;
    assert_eq!(
        emnapi_sync_memory(env, false, &mut value, 0, len),
        napi::Status::Ok
    );
}
