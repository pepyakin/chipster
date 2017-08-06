
use std::cell::RefCell;
use std::ptr::null_mut;
use std::os::raw::{c_int, c_void, c_float};
use super::{Step, Result};

#[allow(non_camel_case_types)]
type em_callback_func = unsafe extern "C" fn();

#[link_args = "-s USE_SDL=2"]
extern "C" {
    pub fn emscripten_set_main_loop(
        func: em_callback_func,
        fps: c_int,
        simulate_infinite_loop: c_int,
    );
    pub fn emscripten_cancel_main_loop();
    pub fn emscripten_get_now() -> c_float;
}

// TODO: do better with emscripten_set_main_loop_arg
// https://kripken.github.io/emscripten-site/docs/api_reference/emscripten.h.html#c.emscripten_set_main_loop_arg
thread_local!(static MAIN_LOOP_CALLBACK: RefCell<*mut c_void> = RefCell::new(null_mut()));

pub fn set_main_loop_callback<F>(callback: F)
where
    F: FnMut(),
{
    MAIN_LOOP_CALLBACK.with(|log| {
        *log.borrow_mut() = &callback as *const _ as *mut c_void;
    });

    unsafe {
        emscripten_set_main_loop(wrapper::<F>, 0, 1);
    }

    unsafe extern "C" fn wrapper<F>()
    where
        F: FnMut(),
    {
        MAIN_LOOP_CALLBACK.with(|z| {
            let closure = *z.borrow_mut() as *mut F;
            (*closure)();
        });
    }
}

pub struct EmscriptenLooper;

impl super::Looper for EmscriptenLooper {
    fn start_loop<F>(self, mut f: F) -> !
    where
        F: FnMut() -> Result<super::Step>,
    {
        set_main_loop_callback(|| match f() {
            Ok(Step::Cont) => {}
            Ok(Step::Done) => unsafe {
                emscripten_cancel_main_loop();
            },
            Err(e) => unsafe {
                println!("Error: {}", e);
                emscripten_cancel_main_loop();
            },
        });

        // unreachable because simulate_infinite_loop=1 and
        // emscripten_cancel_main_loop actually doesn't make to
        // set_main_loop_callback return.
        unreachable!()
    }
}
