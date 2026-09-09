use iced::Rectangle;

use crate::widget::{Color, Gradient};

#[derive(Clone, Copy)]
pub(super) struct TextureGradient {
    gradient: Gradient,
    bounds: Rectangle,
}

pub(super) fn frame_gradient(
    frame: &crate::widget::Frame,
    bounds: Rectangle,
) -> Option<TextureGradient> {
    frame
        .gradient
        .map(|gradient| TextureGradient { gradient, bounds })
}

impl TextureGradient {
    pub(super) fn colors(self, bounds: Rectangle, tint: [f32; 4]) -> [[f32; 4]; 4] {
        [
            self.color_at(bounds.x, bounds.y, tint),
            self.color_at(bounds.x + bounds.width, bounds.y, tint),
            self.color_at(bounds.x + bounds.width, bounds.y + bounds.height, tint),
            self.color_at(bounds.x, bounds.y + bounds.height, tint),
        ]
    }

    fn color_at(self, x: f32, y: f32, tint: [f32; 4]) -> [f32; 4] {
        let amount = if self.gradient.vertical {
            1.0 - normalized(y, self.bounds.y, self.bounds.height)
        } else {
            normalized(x, self.bounds.x, self.bounds.width)
        };
        multiply_color(
            interpolate(self.gradient.min_color, self.gradient.max_color, amount),
            tint,
        )
    }
}

fn normalized(value: f32, start: f32, length: f32) -> f32 {
    if length > 0.0 {
        ((value - start) / length).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn interpolate(min: Color, max: Color, amount: f32) -> Color {
    Color::new(
        min.r + (max.r - min.r) * amount,
        min.g + (max.g - min.g) * amount,
        min.b + (max.b - min.b) * amount,
        min.a + (max.a - min.a) * amount,
    )
}

fn multiply_color(color: Color, tint: [f32; 4]) -> [f32; 4] {
    [
        color.r * tint[0],
        color.g * tint[1],
        color.b * tint[2],
        color.a * tint[3],
    ]
}
