#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Entrypoint (n6g), load this @ 0x083798b4, call as 0x083798b5
/// Entrypoint (n7), load this @ 0x0849696c, call as 0x0849696d
#[no_mangle]
#[link_section = ".text.prologue"]
#[export_name = "_start"]
pub extern "C" fn custom_handler() {
    // This is some random h264 code, we don't need it :3 (n6g)
    let input = 0x082c4754 as *mut u8;
    // n7g
    let input = 0x08492a50 as *mut u8;

    // n6g
    let aes_func = unsafe { core::mem::transmute::<u32, extern "C" fn(u32, u32, *mut u8, *mut u8, *mut u8, u32, u32)>(0x0822215c | 1) };
    // n7g
    let aes_func = unsafe { core::mem::transmute::<u32, extern "C" fn(u32, u32, *mut u8, *mut u8, *mut u8, u32, u32)>(0x0841140c | 1) };

    aes_func(
        0, /* Decrypt*/
        1, /* global key */
        core::ptr::null_mut(), /* no IV */
        input, /* In-place decrypt */
        input,
        512, /* Size*/
        0 /* flags? */
    );
}
