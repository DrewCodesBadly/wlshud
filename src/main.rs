use std::{
    sync::mpsc,
    thread::{self},
    time::{Duration, Instant},
};

use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::{
        calloop::EventLoop,
        calloop_wayland_source::WaylandSource,
        client::{Connection, globals::registry_queue_init},
    },
    registry::RegistryState,
    seat::SeatState,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, LayerShell},
    },
    shm::{Shm, slot::SlotPool},
};

use crate::{
    rendering::{AppContext, create_app_fade_tween, layout::AppLayout},
    searching::{SearchResults, searching_thread},
    window::HUDWindow,
};

mod config;
mod rendering;
mod searching;
mod window;

fn main() {
    // Spawn searcher thread
    // let search_results = SearchResults::default();
    let (results_send, results_recv) = mpsc::channel();
    let (query_send, query_recv) = mpsc::channel();

    let conn = Connection::connect_to_env().unwrap();
    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();
    let mut event_loop: EventLoop<HUDWindow> = EventLoop::try_new().unwrap();
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue)
        .insert(loop_handle.clone())
        .unwrap();

    let compositor = CompositorState::bind(&globals, &qh).unwrap();
    let layer_shell =
        LayerShell::bind(&globals, &qh).expect("Compositor does not support layer shells");
    let surface = compositor.create_surface(&qh);
    let layer_surface = layer_shell.create_layer_surface(
        &qh,
        surface,
        smithay_client_toolkit::shell::wlr_layer::Layer::Top,
        Some("WLSHUD"),
        None,
    );
    layer_surface.set_anchor(Anchor::all());
    layer_surface.set_keyboard_interactivity(
        smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity::Exclusive,
    );

    layer_surface.commit();

    let shm = Shm::bind(&globals, &qh).unwrap();
    let pool = SlotPool::new(256 * 256 * 4, &shm).unwrap();

    let mut window = HUDWindow {
        shm,
        pool,
        layer_surface,

        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        keyboard: None,
        pointer: None,

        width: 256,
        height: 256,
        buffer: None,

        should_close: false,
        in_closing_animation: false,

        last_frame_time: Instant::now(),

        app_fade_tweener: create_app_fade_tween(0., 1.),
        app_fade_pos: 0.,

        app_layout: AppLayout::default(),
        app_context: AppContext::new(query_send),
        search_results_receiver: results_recv,
    };

    // Start searcher thread
    let _searching_thread = thread::spawn(move || searching_thread(results_send, query_recv));

    // Run startup callback
    window.app_layout.on_startup(&mut window.app_context);

    // Run main event loop
    loop {
        event_loop
            .dispatch(Duration::from_millis(10), &mut window)
            .unwrap();

        if window.should_close {
            break;
        }
    }
}
