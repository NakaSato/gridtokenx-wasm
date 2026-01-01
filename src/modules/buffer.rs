// Simple buffer allocation for creating shared memory space if needed from JS
static mut BUFFER: [f64; 20000] = [0.0; 20000]; // Increased buffer size for points

#[no_mangle]
pub extern "C" fn get_buffer_ptr() -> *mut f64 {
    unsafe { BUFFER.as_mut_ptr() }
}
