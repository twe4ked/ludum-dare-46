use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

struct State {
    context: web_sys::CanvasRenderingContext2d,
}

#[wasm_bindgen(start)]
pub fn start() {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let state = State { context };

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        state.context.clear_rect(
            0.,
            0.,
            state.context.canvas().unwrap().width() as f64,
            state.context.canvas().unwrap().height() as f64,
        );

        state.context.begin_path();

        // Draw the outer circle.
        state
            .context
            .arc(75.0, 75.0, 50.0, 0.0, f64::consts::PI * 2.0)
            .unwrap();

        // Draw the mouth.
        state.context.move_to(110.0, 75.0);
        state
            .context
            .arc(75.0, 75.0, 35.0, 0.0, f64::consts::PI)
            .unwrap();

        // Draw the left eye.
        state.context.move_to(65.0, 65.0);
        state
            .context
            .arc(60.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
            .unwrap();

        // Draw the right eye.
        state.context.move_to(95.0, 65.0);
        state
            .context
            .arc(90.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
            .unwrap();

        state.context.stroke();

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}
