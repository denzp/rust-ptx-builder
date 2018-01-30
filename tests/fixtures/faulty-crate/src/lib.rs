#![feature(abi_ptx, lang_items)]
#![no_std]

#[no_mangle]
pub unsafe extern "ptx-kernel" fn the_kernel(x: *const f64, y: *mut f64, a: f64) {
    *y.offset(0) = external_fn(*x.offset(0)) * a;
}

// Needed because we compile `dylib`...
#[lang = "panic_fmt"]
fn panic_fmt() -> ! {
    loop {}
}
