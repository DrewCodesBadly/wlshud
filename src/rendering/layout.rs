use skia_safe::{Canvas, Color, Paint, Rect};

// TODO: make configurable or something idk
const FOREGROUND_CONTAINER_COLOR: Color = Color::new(0xFFFFFFFF);
const FOREGROUND_CONTAINER_GRADIENT_END: Color = Color::new(0x88888800);

pub trait Widget {
    fn draw(&self, canvas: &Canvas, paint: &mut Paint, area: Rect);
}

#[derive(Default)]
pub struct AppState {}

impl Widget for AppState {
    fn draw(&self, canvas: &Canvas, paint: &mut Paint, area: Rect) {
        // padding at edges of screen
        let padded_area = area.with_inset((32., 32.));

        // Draw search bar
        paint.set_color(FOREGROUND_CONTAINER_COLOR);
        let search_rect = Rect::new(
            padded_area.left,
            padded_area.top,
            padded_area.right,
            padded_area.top + 64.,
        );
        canvas.draw_round_rect(search_rect, 10., 10., &paint);

        // Draw left side bar
        let media_rect = Rect::new(
            padded_area.left,
            padded_area.bottom - 250.,
            padded_area.left + 350.,
            padded_area.bottom,
        );
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
        let notes_rect = Rect::new(
            padded_area.right - 400.,
            padded_area.top + 64. + 32.,
            padded_area.right,
            padded_area.top + 64. + 32. + 600.,
        );
        canvas.draw_round_rect(media_rect, 10., 10., &paint);
        canvas.draw_round_rect(left_rect, 10., 10., &paint);
        canvas.draw_round_rect(bottom_rect, 10., 10., &paint);
        canvas.draw_round_rect(notes_rect, 10., 10., &paint);

        // Draw media display area

        // Draw bottom bar
    }
}
