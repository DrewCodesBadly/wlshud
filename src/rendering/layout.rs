use std::{
    cell::{self, Cell},
    rc::Rc,
};

use skia_safe::{
    Canvas, Color, Color4f, Paint, Rect, Shader,
    gradient::{Colors, Gradient, Interpolation},
    gradient_shader::GradientShaderColors,
    shaders::linear_gradient,
};
use tween::{CubicInOut, CubicOut, Linear, SineOut, Tween, Tweener};

use crate::rendering::{AnimatedValue, AppContext};

// TODO: make configurable or something idk
const FOREGROUND_CONTAINER_COLOR: Color = Color::new(0xFFFFFFFF);
const FOREGROUND_CONTAINER_GRADIENT_END: Color = Color::new(0x00888888);

pub trait Widget {
    fn draw(&mut self, canvas: &Canvas, paint: &mut Paint, area: Rect, context: &mut AppContext);
}

pub struct AppLayout {
    search_bar_width: AnimatedValue,
}

impl Default for AppLayout {
    fn default() -> Self {
        Self {
            search_bar_width: Default::default(),
        }
    }
}

impl AppLayout {
    pub fn on_startup(&mut self, context: &mut AppContext) {
        context.add_animation(
            self.search_bar_width.clone(),
            Tweener::new(0.25, 1., 0.5, Box::new(SineOut::new())),
        );
    }
}

impl Widget for AppLayout {
    fn draw(&mut self, canvas: &Canvas, paint: &mut Paint, area: Rect, context: &mut AppContext) {
        // padding at edges of screen
        let padded_area = area.with_inset((32., 32.));

        // Draw search bar
        paint.set_color(FOREGROUND_CONTAINER_COLOR);
        let search_rect = Rect::new(
            padded_area.left,
            padded_area.top,
            padded_area.left + padded_area.width() * self.search_bar_width.get(),
            padded_area.top + 64.,
        );
        canvas.draw_round_rect(search_rect, 10., 10., &paint);

        // Draw media area
        let media_rect = Rect::new(
            padded_area.left,
            padded_area.bottom - 250.,
            padded_area.left + 350.,
            padded_area.bottom,
        );
        canvas.draw_round_rect(media_rect, 10., 10., &paint);

        // Draw notes area
        let notes_rect = Rect::new(
            padded_area.right - 400.,
            padded_area.top + 64. + 32.,
            padded_area.right,
            padded_area.top + 64. + 32. + 600.,
        );
        canvas.draw_round_rect(notes_rect, 10., 10., &paint);

        // Draw left and bottom parts
        // let grad_colors = [
        //     FOREGROUND_CONTAINER_COLOR.into(),
        //     FOREGROUND_CONTAINER_COLOR.into(),
        //     FOREGROUND_CONTAINER_COLOR.into(),
        //     FOREGROUND_CONTAINER_GRADIENT_END.into(),
        //     FOREGROUND_CONTAINER_GRADIENT_END.into(),
        // ];
        let left_rect = Rect::new(
            padded_area.left,
            padded_area.top + 64. + 32.,
            padded_area.left + 64.,
            padded_area.bottom - 250. - 32.,
        );
        let bottom_rect = Rect::new(
            padded_area.left + 350. + 32.,
            padded_area.bottom - 64.,
            padded_area.right,
            padded_area.bottom,
        );
        // let grad = Gradient::new(
        //     Colors::new(&grad_colors, None, skia_safe::TileMode::Repeat, None),
        //     Interpolation::default(),
        // );
        // paint.set_shader(linear_gradient(
        //     ((0., left_rect.top()), (0., left_rect.bottom())),
        //     &grad,
        //     None,
        // ));
        canvas.draw_round_rect(left_rect, 10., 10., &paint);
        // paint.set_shader(linear_gradient(
        //     ((bottom_rect.right(), 0.), (bottom_rect.left(), 0.)),
        //     &grad,
        //     None,
        // ));
        canvas.draw_round_rect(bottom_rect, 10., 10., &paint);
    }
}
