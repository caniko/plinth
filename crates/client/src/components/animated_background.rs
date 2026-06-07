use leptos::prelude::*;

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

    fn draw_flow_field(&mut self) {
        self.fade("rgba(8, 12, 22, 0.075)");
        for particle in &mut self.particles {
            let angle = smooth_noise(
                particle.x * 0.004 + self.tick * 0.0009,
                particle.y * 0.004 + 19.0,
            ) * std::f64::consts::TAU
                * 3.0;
            let speed = 0.8 + smooth_noise(particle.x * 0.003, particle.y * 0.003 + 50.0) * 1.8;
            particle.x += angle.cos() * speed;
            particle.y += angle.sin() * speed;
            particle.life -= 0.0014;

            if particle.life <= 0.0
                || particle.x < 0.0
                || particle.x > self.width
                || particle.y < 0.0
                || particle.y > self.height
            {
                particle.x = self.rng.range(0.0, self.width);
                particle.y = self.rng.range(0.0, self.height);
                particle.life = 1.0;
            }

            self.context.begin_path();
            self.context
                .set_global_alpha((particle.life * 0.2).min(0.22));
            self.context.set_fill_style_str(palette(particle.color));
            let _ = self
                .context
                .arc(particle.x, particle.y, 1.15, 0.0, std::f64::consts::TAU);
            self.context.fill();
        }
        self.context.set_global_alpha(1.0);
    }

    fn draw_constellation(&mut self) {
        self.fade("rgba(5, 9, 18, 0.28)");
        for node in &mut self.nodes {
            node.x += node.vx;
            node.y += node.vy;
            if node.x < 0.0 || node.x > self.width {
                node.vx *= -1.0;
            }
            if node.y < 0.0 || node.y > self.height {
                node.vy *= -1.0;
            }
        }

        self.context.set_line_width(1.0);
        for a in 0..self.nodes.len() {
            for b in (a + 1)..self.nodes.len() {
                let dx = self.nodes[a].x - self.nodes[b].x;
                let dy = self.nodes[a].y - self.nodes[b].y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < 130.0 {
                    self.context.set_global_alpha((1.0 - dist / 130.0) * 0.18);
                    self.context.set_stroke_style_str("#9cdef2");
                    self.context.begin_path();
                    self.context.move_to(self.nodes[a].x, self.nodes[a].y);
                    self.context.line_to(self.nodes[b].x, self.nodes[b].y);
                    self.context.stroke();
                }
            }
        }

        for node in &self.nodes {
            self.context.set_global_alpha(0.5);
            self.context.set_fill_style_str("#f59e0b");
            self.context.begin_path();
            let _ = self
                .context
                .arc(node.x, node.y, 1.35, 0.0, std::f64::consts::TAU);
            self.context.fill();
        }
        self.context.set_global_alpha(1.0);
    }

    fn draw_aurora(&mut self) {
        self.fade("rgba(4, 9, 18, 0.18)");
        for band in 0..6 {
            let base = self.height * (0.18 + band as f64 * 0.105);
            let amp = 36.0 + band as f64 * 9.0;
            self.context.begin_path();
            self.context.move_to(0.0, self.height);
            let mut x = 0.0;
            while x <= self.width + 32.0 {
                let y = base
                    + (x * 0.009 + self.tick * 0.018 + band as f64).sin() * amp
                    + (x * 0.021 - self.tick * 0.011).cos() * amp * 0.32;
                self.context.line_to(x, y);
                x += 24.0;
            }
            self.context.line_to(self.width, self.height);
            self.context.close_path();
            self.context.set_global_alpha(0.07 + band as f64 * 0.012);
            self.context
                .set_fill_style_str(["#9cdef2", "#e06c75", "#50fa7b"][band % 3]);
            self.context.fill();
        }
        self.context.set_global_alpha(1.0);
    }

    fn draw_orbits(&mut self) {
        self.fade("rgba(7, 10, 19, 0.09)");
        let centers = [
            (self.width * 0.25, self.height * 0.35),
            (self.width * 0.72, self.height * 0.32),
            (self.width * 0.43, self.height * 0.68),
            (self.width * 0.82, self.height * 0.75),
        ];

        for particle in &mut self.particles {
            let center = centers[particle.x as usize % centers.len()];
            particle.y += particle.vy;
            let wobble = (self.tick * 0.01 + particle.vx * 0.04).sin() * 14.0;
            let x = center.0 + particle.y.cos() * (particle.vx + wobble);
            let y = center.1 + particle.y.sin() * (particle.vx * 0.42 + wobble);
            self.context.set_global_alpha(0.16 * particle.life);
            self.context.set_fill_style_str(palette(particle.color));
            self.context.begin_path();
            let _ = self.context.arc(x, y, 1.4, 0.0, std::f64::consts::TAU);
            self.context.fill();
        }
        self.context.set_global_alpha(1.0);
    }

    fn draw_digital_rain(&mut self) {
        self.fade("rgba(0, 0, 0, 0.16)");
        self.context.set_font("13px 'Fira Code', monospace");
        for glyph in &mut self.glyphs {
            glyph.y += glyph.speed;
            if glyph.y > self.height + 40.0 {
                glyph.y = self.rng.range(-160.0, -20.0);
                glyph.value = self.rng.next_u32();
            }
            let ch = char::from_digit(glyph.value % 16, 16).unwrap_or('0');
            self.context.set_global_alpha(0.18 + glyph.speed * 0.05);
            self.context.set_fill_style_str("#f59e0b");
            let _ = self.context.fill_text(&ch.to_string(), glyph.x, glyph.y);
            self.context.set_global_alpha(0.08);
            self.context.set_fill_style_str("#9cdef2");
            let _ = self.context.fill_text("|", glyph.x + 2.0, glyph.y - 18.0);
        }
        self.context.set_global_alpha(1.0);
    }

    fn draw_topographic(&mut self) {
        self.fade("rgba(8, 12, 20, 0.2)");
        self.context.set_line_width(1.0);
        for row in 0..18 {
            let y_base = row as f64 * self.height / 17.0;
            self.context.begin_path();
            let mut x = -20.0;
            while x <= self.width + 20.0 {
                let noise = (x * 0.018 + self.tick * 0.012).sin() * 14.0
                    + (x * 0.041 - self.tick * 0.009 + row as f64).cos() * 9.0;
                let y = y_base + noise;
                if x < -10.0 {
                    self.context.move_to(x, y);
                } else {
                    self.context.line_to(x, y);
                }
                x += 18.0;
            }
            self.context.set_global_alpha(0.1 + row as f64 * 0.004);
            self.context
                .set_stroke_style_str(if row % 3 == 0 { "#e06c75" } else { "#9cdef2" });
            self.context.stroke();
        }
        self.context.set_global_alpha(1.0);
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    color: usize,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
struct Node {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
struct Glyph {
    x: f64,
    y: f64,
    speed: f64,
    value: u32,
}

#[cfg(target_arch = "wasm32")]
struct Rng {
    state: u32,
}

#[cfg(target_arch = "wasm32")]
impl Rng {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        f64::from(self.next_u32()) / f64::from(u32::MAX)
    }

    fn range(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min).max(0.0)
    }
}

#[cfg(target_arch = "wasm32")]
fn palette(index: usize) -> &'static str {
    ["#9cdef2", "#e06c75", "#5fb6cc"][index % 3]
}

#[cfg(target_arch = "wasm32")]
fn smooth_noise(x: f64, y: f64) -> f64 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = x - ix;
    let fy = y - iy;
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let a = hash_noise(ix, iy);
    let b = hash_noise(ix + 1.0, iy);
    let c = hash_noise(ix, iy + 1.0);
    let d = hash_noise(ix + 1.0, iy + 1.0);
    a + (b - a) * ux + (c - a) * uy + (a - b - c + d) * ux * uy
}

#[cfg(target_arch = "wasm32")]
fn hash_noise(x: f64, y: f64) -> f64 {
    let value = (x * 12.9898 + y * 78.233).sin() * 43_758.545_3;
    value - value.floor()
}
