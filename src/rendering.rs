use crate::rendering::layout::Widget;
use std::time::Instant;

use skia_safe::{BlendMode, Canvas, Color4f, ColorSpace, ImageInfo, Paint, Rect};
use smithay_client_toolkit::{
    reexports::client::{QueueHandle, protocol::wl_shm},
    shell::WaylandSurface,
};
use tween::{CubicOut, Tweener};

use crate::window::HUDWindow;

pub mod layout;

const BACKGROUND_ALPHA: f32 = 0.5;

pub type FadeTweenType = CubicOut;
pub fn create_app_fade_tween(start: f32, end: f32) -> Tweener<f32, f64, FadeTweenType> {
    Tweener::new(start, end, 0.2, FadeTweenType::new())
}

impl HUDWindow {
    pub fn start_closing_animation(&mut self) {
        if self.in_closing_animation {
            return;
        }

        self.in_closing_animation = true;
        self.app_fade_tweener = create_app_fade_tween(self.app_fade_pos, 0.);
        // will be committed next redraw
        self.layer_surface.set_keyboard_interactivity(
            smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity::None,
        );
    }

    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        // Update animation states before drawing
        let now = Instant::now();
        let frame_delta = now.duration_since(self.last_frame_time).as_secs_f64();
        self.last_frame_time = now;
        self.app_fade_pos = self.app_fade_tweener.move_by(frame_delta);
        if self.in_closing_animation && self.app_fade_tweener.is_finished() {
            self.should_close = true;
            return;
        }

        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool
                .create_buffer(
                    self.width as i32,
                    self.height as i32,
                    self.width as i32 * 4,
                    wl_shm::Format::Argb8888,
                )
                .unwrap()
                .0
        });

        {
            let canvas = match self.pool.canvas(buffer) {
                Some(canvas) => canvas,
                None => {
                    // double-buffer if needed
                    let (second_buffer, canvas) = self
                        .pool
                        .create_buffer(
                            self.width as i32,
                            self.height as i32,
                            self.width as i32 * 4,
                            wl_shm::Format::Argb8888,
                        )
                        .unwrap();
                    *buffer = second_buffer;
                    canvas
                }
            };
            let img_info = ImageInfo::new(
                (self.width as i32, self.height as i32),
                skia_safe::ColorType::BGRA8888,
                skia_safe::AlphaType::Premul,
                ColorSpace::new_srgb(),
            );
            let sk_canvas = Canvas::from_raster_direct(&img_info, canvas, None, None).unwrap();
            let mut paint = Paint::default();
            paint.set_anti_alias(true);

            // draw...
            sk_canvas.clear(Color4f::new(0., 0., 0., BACKGROUND_ALPHA));
            self.app_state.draw(
                &sk_canvas,
                &mut paint,
                Rect::from_wh(self.width as f32, self.height as f32),
            );

            // Multiplies by the fade animation amount.
            sk_canvas.draw_color(
                Color4f::new(1., 1., 1., self.app_fade_pos),
                Some(BlendMode::Modulate),
            );
        }

        let surface = self.layer_surface.wl_surface();

        surface.damage_buffer(0, 0, self.width as i32, self.height as i32);
        // requests another frame for constant redraws
        surface.frame(qh, surface.clone());
        buffer.attach_to(surface).unwrap();
        self.layer_surface.commit();
    }
}
