use skia_safe::{Canvas, Color, Paint, Rect};
use tween::{SineOut, Tweener};

use crate::rendering::{AnimatedValue, AppContext, text::TextField};

// TODO: make configurable or something idk
const FOREGROUND_CONTAINER_COLOR: Color = Color::new(0xFFFFFFFF);
const FOREGROUND_CONTAINER_GRADIENT_END: Color = Color::new(0x00888888);

pub trait Widget {
    fn draw(&mut self, canvas: &Canvas, paint: &mut Paint, area: Rect, context: &mut AppContext);
}

pub struct AppLayout {
    search_bar_width: AnimatedValue,
    search_text_field: TextField,
}

impl Default for AppLayout {
    fn default() -> Self {
        Self {
            search_bar_width: Default::default(),
            search_text_field: TextField::new(Color::BLACK, 24.0),
        }
    }
}

impl AppLayout {
    pub fn on_startup(&mut self, context: &mut AppContext) {
        context.add_animation(
            self.search_bar_width.clone(),
            Tweener::new(0.25, 1., 0.5, Box::new(SineOut::new())),
        );
        let send_clone = context.query_send.clone();
        context.set_key_handler(self.search_text_field.input_handler(move |s| {
            let _ = send_clone.send(s.clone());
        }));
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
        self.search_text_field
            .draw(canvas, paint, search_rect.with_inset((8.0, 8.0)), context);

        if context.search_results.is_empty() {
            // Draw media area
            paint.set_color(FOREGROUND_CONTAINER_COLOR);
            let media_rect = Rect::new(
                padded_area.left,
                padded_area.bottom - 250.,
                padded_area.left + 350.,
                padded_area.bottom,
            );
            canvas.draw_round_rect(media_rect, 10., 10., &paint);

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
        } else {
            // Draw search result list
            for res in &context.search_results {}
        }
    }
}
