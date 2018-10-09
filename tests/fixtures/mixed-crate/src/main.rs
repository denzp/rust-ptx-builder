#![feature(abi_ptx)]
#![no_std]

mod mod1;
mod mod2;

#[cfg(not(target_os = "cuda"))]
fn main() {
    println!("Hello, world!");
}

#[no_mangle]
pub unsafe extern "ptx-kernel" fn the_kernel(x: *const f64, y: *mut f64, a: f64) {
    *y.offset(0) = *x.offset(0) * a;
}

#[panic_handler]
fn dummy_panic_handler(_info: &::core::panic::PanicInfo) -> ! {
    loop {}
}
