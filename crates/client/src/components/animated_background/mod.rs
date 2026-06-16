use leptos::prelude::*;

mod drawing;
mod palette;
mod particles;

#[cfg(target_arch = "wasm32")]
use self::particles::{Glyph, Node, Particle, Rng};

/// Home-page animated background. The canvas is inert during SSR and activates
/// only after client hydration when motion is allowed.
#[cfg_attr(all(not(feature = "csr"), feature = "islands"), island)]
#[cfg_attr(any(feature = "csr", not(feature = "islands")), component)]
pub fn AnimatedBackground(preset: String) -> impl IntoView {
    let preset = normalize_preset(&preset).to_string();

    #[cfg(target_arch = "wasm32")]
    {
        let effect_preset = preset.clone();
        Effect::new(move |_| {
            if effect_preset != "none" {
                start_canvas_animation(effect_preset.clone());
            }
        });
    }

    view! {
        <canvas
            id="plinth-animated-background"
            class="plinth-bg-canvas"
            data-preset=preset
            aria-hidden="true"
        ></canvas>
    }
}

pub fn normalize_preset(preset: &str) -> &'static str {
    match preset {
        "none" => "none",
        "constellation" => "constellation",
        "aurora-ribbons" => "aurora-ribbons",
        "orbital-trails" => "orbital-trails",
        "digital-rain" => "digital-rain",
        "topographic-waves" => "topographic-waves",
        "flow-field" => "flow-field",
        _ => "flow-field",
    }
}

#[cfg(target_arch = "wasm32")]
fn start_canvas_animation(preset: String) {
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::{CanvasRenderingContext2d, Event, HtmlCanvasElement};

    let Some(window) = web_sys::window() else {
        return;
    };
    if prefers_reduced_motion(&window) {
        return;
    }

    let Some(document) = window.document() else {
        return;
    };
    let Some(element) = document.get_element_by_id("plinth-animated-background") else {
        return;
    };
    let Ok(canvas) = element.dyn_into::<HtmlCanvasElement>() else {
        return;
    };
    let Ok(Some(context)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(context) = context.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    let runtime = Rc::new(RefCell::new(CanvasRuntime::new(
        preset,
        canvas,
        context,
        window.device_pixel_ratio().clamp(1.0, 2.0),
    )));
    runtime.borrow_mut().resize();

    let resize_runtime = Rc::clone(&runtime);
    let resize = Rc::new(Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
        resize_runtime.borrow_mut().resize();
    })));
    let _ =
        window.add_event_listener_with_callback("resize", resize.as_ref().as_ref().unchecked_ref());

    schedule_frame(window, runtime, resize);
}

#[cfg(target_arch = "wasm32")]
fn schedule_frame(
    window: web_sys::Window,
    runtime: std::rc::Rc<std::cell::RefCell<CanvasRuntime>>,
    resize: std::rc::Rc<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
) {
    use wasm_bindgen::{JsCast, closure::Closure};

    let callback_window = window.clone();
    let callback_runtime = std::rc::Rc::clone(&runtime);
    let callback_resize = std::rc::Rc::clone(&resize);
    let mut callback = Some(Box::new(move || {
        if !callback_runtime.borrow().canvas.is_connected() {
            let _ = callback_window.remove_event_listener_with_callback(
                "resize",
                callback_resize.as_ref().as_ref().unchecked_ref(),
            );
            return;
        }

        callback_runtime.borrow_mut().draw();
        schedule_frame(callback_window, callback_runtime, callback_resize);
    }) as Box<dyn FnOnce()>);

    let closure = Closure::<dyn FnMut()>::new(move || {
        if let Some(callback) = callback.take() {
            callback();
        }
    });
    let callback = closure.into_js_value();
    let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
}

#[cfg(target_arch = "wasm32")]
fn prefers_reduced_motion(window: &web_sys::Window) -> bool {
    window
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|media| media.matches())
}

#[cfg(target_arch = "wasm32")]
struct CanvasRuntime {
    preset: String,
    canvas: web_sys::HtmlCanvasElement,
    context: web_sys::CanvasRenderingContext2d,
    width: f64,
    height: f64,
    dpr: f64,
    tick: f64,
    rng: Rng,
    particles: Vec<Particle>,
    nodes: Vec<Node>,
    glyphs: Vec<Glyph>,
}

#[cfg(target_arch = "wasm32")]
impl CanvasRuntime {
    fn new(
        preset: String,
        canvas: web_sys::HtmlCanvasElement,
        context: web_sys::CanvasRenderingContext2d,
        dpr: f64,
    ) -> Self {
        Self {
            preset,
            canvas,
            context,
            width: 0.0,
            height: 0.0,
            dpr,
            tick: 0.0,
            rng: Rng::new(0x5eed_cafe),
            particles: Vec::new(),
            nodes: Vec::new(),
            glyphs: Vec::new(),
        }
    }

    fn resize(&mut self) {
        let width = f64::from(self.canvas.client_width().max(1));
        let height = f64::from(self.canvas.client_height().max(1));
        if (width - self.width).abs() < 1.0 && (height - self.height).abs() < 1.0 {
            return;
        }

        self.width = width;
        self.height = height;
        self.canvas
            .set_width((self.width * self.dpr).round().max(1.0) as u32);
        self.canvas
            .set_height((self.height * self.dpr).round().max(1.0) as u32);
        let _ = self
            .context
            .set_transform(self.dpr, 0.0, 0.0, self.dpr, 0.0, 0.0);
        self.seed_preset();
    }

    fn seed_preset(&mut self) {
        self.particles.clear();
        self.nodes.clear();
        self.glyphs.clear();

        match self.preset.as_str() {
            "constellation" => {
                for _ in 0..72 {
                    self.nodes.push(Node {
                        x: self.rng.range(0.0, self.width),
                        y: self.rng.range(0.0, self.height),
                        vx: self.rng.range(-0.32, 0.32),
                        vy: self.rng.range(-0.32, 0.32),
                    });
                }
            }
            "orbital-trails" => {
                for i in 0..96 {
                    let center = i % 4;
                    self.particles.push(Particle {
                        x: center as f64,
                        y: self.rng.range(0.0, std::f64::consts::TAU),
                        vx: self.rng.range(30.0, self.width.min(self.height) * 0.38),
                        vy: self.rng.range(0.003, 0.012),
                        life: self.rng.range(0.45, 1.0),
                        color: i % 3,
                    });
                }
            }
            "digital-rain" => {
                let columns = (self.width / 18.0).ceil() as usize;
                for i in 0..columns {
                    self.glyphs.push(Glyph {
                        x: i as f64 * 18.0 + self.rng.range(-2.0, 2.0),
                        y: self.rng.range(-self.height, self.height),
                        speed: self.rng.range(1.2, 3.8),
                        value: self.rng.next_u32(),
                    });
                }
            }
            _ => {
                let count = match self.preset.as_str() {
                    "aurora-ribbons" => 36,
                    "topographic-waves" => 24,
                    _ => 260,
                };
                for i in 0..count {
                    self.particles.push(Particle {
                        x: self.rng.range(0.0, self.width),
                        y: self.rng.range(0.0, self.height),
                        vx: self.rng.range(-0.8, 0.8),
                        vy: self.rng.range(-0.8, 0.8),
                        life: self.rng.range(0.35, 1.0),
                        color: i % 3,
                    });
                }
            }
        }
    }

    fn draw(&mut self) {
        if self.tick as u32 % 45 == 0 {
            self.resize();
        }

        match self.preset.as_str() {
            "constellation" => self.draw_constellation(),
            "aurora-ribbons" => self.draw_aurora(),
            "orbital-trails" => self.draw_orbits(),
            "digital-rain" => self.draw_digital_rain(),
            "topographic-waves" => self.draw_topographic(),
            _ => self.draw_flow_field(),
        }
        self.tick += 1.0;
    }

    fn fade(&self, color: &str) {
        self.context.set_global_alpha(1.0);
        self.context.set_fill_style_str(color);
        self.context.fill_rect(0.0, 0.0, self.width, self.height);
    }
}
