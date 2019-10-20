extern crate serde;
extern crate web_sys;

use wasm_bindgen::prelude::*;

mod data_plot;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub use data_plot::draw_data;

pub fn make_coord_mapping_closure<T: Into<f64> + 'static>(
    map_func: Option<Box<dyn Fn((i32, i32)) -> Option<(T, T)>>>,
) -> JsValue {
    web_sys::console::log_1(&"Hello, world!".into());
    if let Some(mapping_func) = map_func {
        let closure = Closure::wrap(Box::new(move |x: i32, y: i32, idx: u32| {
            if let Some((x, y)) = mapping_func((x, y)) {
                if idx == 0 {
                    return x.into();
                }
                return y.into();
            } else {
                return std::f64::NAN;
            }
        }) as Box<dyn FnMut(i32, i32, u32) -> f64>);

        let js_value = closure.as_ref().clone();
        closure.forget();

        return js_value;
    } else {
        return JsValue::null();
    }
}
