use skia_safe::{Canvas, Color4f, ColorSpace, ImageInfo};
use smithay_client_toolkit::{
    reexports::client::{QueueHandle, protocol::wl_shm},
    shell::WaylandSurface,
};

use crate::window::HUDWindow;

impl HUDWindow {
    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
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
            let canvas = self.pool.canvas(buffer).unwrap();
            let img_info = ImageInfo::new(
                (self.width as i32, self.height as i32),
                skia_safe::ColorType::BGRA8888,
                skia_safe::AlphaType::Premul,
                ColorSpace::new_srgb(),
            );
            let sk_canvas = Canvas::from_raster_direct(&img_info, canvas, None, None).unwrap();

            // draw...
            sk_canvas.clear(Color4f::new(0., 0., 0., 0.5));
        }

        self.layer_surface
            .wl_surface()
            .damage_buffer(0, 0, self.width as i32, self.height as i32);
        self.layer_surface
            .wl_surface()
            .frame(qh, self.layer_surface.wl_surface().clone());
        buffer.attach_to(self.layer_surface.wl_surface()).unwrap();
        self.layer_surface.commit();
    }
}
