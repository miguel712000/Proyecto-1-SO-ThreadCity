use std::sync::{Arc, Mutex};

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, DrawingArea};
use glib::timeout_add_local;

use proyecto1::threadcity::city::City;
use proyecto1::threadcity::entities::VehicleType;

// Alias útil para compartir la ciudad
pub type SharedCity = Arc<Mutex<City>>;

/// Arranca la aplicación GTK usando la ciudad compartida.
pub fn run_gui(city: SharedCity) {
    let app = Application::builder()
        .application_id("cr.tecdos.threadcity")
        .build();

    let city_for_ui = city.clone();
    app.connect_activate(move |app| {
        build_ui(app, city_for_ui.clone());
    });

    app.run();
}

fn build_ui(app: &Application, city: SharedCity) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("ThreadCity – GTK")
        .default_width(600)
        .default_height(600)
        .build();

    let drawing_area = DrawingArea::builder()
        .content_width(600)
        .content_height(600)
        .hexpand(true)
        .vexpand(true)
        .build();

    let city_for_draw = city.clone();
    drawing_area.set_draw_func(move |_, cr, width, height| {
        // 1. Snapshot de la ciudad bajo lock
        let (grid_w, grid_h, vehicles) = {
            let c = city_for_draw.lock().unwrap();
            c.snapshot()
        };

        let cell_w = width as f64 / grid_w as f64;
        let cell_h = height as f64 / grid_h as f64;

        // Fondo
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().unwrap();

        // Cuadrícula
        cr.set_source_rgb(0.8, 0.8, 0.8);
        for i in 0..=grid_w {
            let x = i as f64 * cell_w;
            cr.move_to(x, 0.0);
            cr.line_to(x, height as f64);
        }
        for j in 0..=grid_h {
            let y = j as f64 * cell_h;
            cr.move_to(0.0, y);
            cr.line_to(width as f64, y);
        }
        cr.stroke().unwrap();

        // Vehículos
        for v in vehicles {
            let (vx, vy) = v.pos;
            let cx = (vx as f64 + 0.5) * cell_w;
            let cy = (vy as f64 + 0.5) * cell_h;
            let r = f64::min(cell_w, cell_h) * 0.3;

            match v.vtype {
                VehicleType::Car =>       cr.set_source_rgb(0.0, 0.0, 1.0), // azul
                VehicleType::Ambulance => cr.set_source_rgb(1.0, 0.0, 0.0), // rojo
                VehicleType::Boat =>      cr.set_source_rgb(0.0, 0.5, 0.0), // verde
                VehicleType::SupplyTruck => cr.set_source_rgb(1.0, 0.5, 0.0), // naranja
            }

            cr.arc(cx, cy, r, 0.0, std::f64::consts::TAU);
            cr.fill().unwrap();
        }
    });

    window.set_child(Some(&drawing_area));

    // Timer para repintar la ventana (no toca mypthreads, solo redibuja)
    let drawing_area_clone = drawing_area.clone();
    timeout_add_local(std::time::Duration::from_millis(100), move || {
        drawing_area_clone.queue_draw();
        glib::ControlFlow::Continue
    });

    window.present();
}