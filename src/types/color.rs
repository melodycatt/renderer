use std::ops::Mul;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Color {
        Color {r, g, b}
    }
    pub fn new_rgb(r: f32, g: f32, b: f32) -> Color {
        Color {r: r / 255.0, g: g / 255.0, b: b / 255.0}
    }
    pub fn new_hsv(h:f32, s: f32, v: f32) -> Color {
        let c = v * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
        let m = v - c;
    
        let (r1, g1, b1) = if (0.0..1.0).contains(&h_prime) {
            (c, x, 0.0)
        } else if (1.0..2.0).contains(&h_prime) {
            (x, c, 0.0)
        } else if (2.0..3.0).contains(&h_prime) {
            (0.0, c, x)
        } else if (3.0..4.0).contains(&h_prime) {
            (0.0, x, c)
        } else if (4.0..5.0).contains(&h_prime) {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
    
        let r = (r1 + m).round();
        let g = (g1 + m).round();
        let b = (b1 + m).round();
    
        Color::new(r, g, b)
    }

    pub fn rgb(&self) -> (f32, f32, f32) {
        (self.r * 255.0, self.g * 255.0, self.b * 255.0)
    }

    pub fn buffer(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b
        }
    }
}