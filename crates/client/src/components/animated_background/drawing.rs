#[cfg(target_arch = "wasm32")]
use super::palette::{palette, smooth_noise};

#[cfg(target_arch = "wasm32")]
impl super::CanvasRuntime {
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
