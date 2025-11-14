use std::sync::{Arc, Mutex};

use std::rc::Rc;

use gdk4::prelude::GdkCairoContextExt;
use gdk_pixbuf::Pixbuf;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, DrawingArea};
use glib::timeout_add_local;

use proyecto1::threadcity::city::City;
use proyecto1::threadcity::entities::VehicleType;

// Alias útil para compartir la ciudad
pub type SharedCity = Arc<Mutex<City>>;

//Struct para los sprites
struct Sprites {
    car: Pixbuf,
    ambulance: Pixbuf,
    boat: Pixbuf,
    truck: Pixbuf,
}


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

    // Cargar sprites desde assets/
    let sprites = Rc::new(Sprites {
        car: Pixbuf::from_file("assets/car.png").expect("No se pudo cargar car.png"),
        ambulance: Pixbuf::from_file("assets/ambulance.png").expect("No se pudo cargar ambulance.png"),
        boat: Pixbuf::from_file("assets/boat.png").expect("No se pudo cargar boat.png"),
        truck: Pixbuf::from_file("assets/truck.png").expect("No se pudo cargar truck.png"),
    });

    let drawing_area = DrawingArea::builder()
        .content_width(600)
        .content_height(600)
        .hexpand(true)
        .vexpand(true)
        .build();

    let city_for_draw = city.clone();
    let sprites_for_draw = sprites.clone();
    drawing_area.set_draw_func(move |_, cr, width, height| {
        let sprites = &*sprites_for_draw;
        // Snapshot de la ciudad
        let (grid_w, grid_h, vehicles) = {
            let c = city_for_draw.lock().unwrap();
            c.snapshot()
        };

        let cell_w = width as f64 / grid_w as f64;
        let cell_h = height as f64 / grid_h as f64;

        // 1) Fondo “césped”
        cr.set_source_rgb(0.9, 1.0, 0.9);
        cr.paint().unwrap();

        // 2) Río en la fila central (donde se mueve el barco)
        let river_row = 2.min(grid_h.saturating_sub(1)); // por si cambias tamaño
        let river_top = river_row as f64 * cell_h;
        cr.set_source_rgb(0.7, 0.85, 1.0); // azul clarito
        cr.rectangle(0.0, river_top, width as f64, cell_h);
        cr.fill().unwrap();

        // 3) Calles norte y sur
        cr.set_source_rgb(0.8, 0.8, 0.8);
        if grid_h >= 2 {
            // calle norte (fila 0)
            cr.rectangle(0.0, 0.0, width as f64, cell_h);
            cr.fill().unwrap();
            // calle sur (última fila)
            let south_top = (grid_h - 1) as f64 * cell_h;
            cr.rectangle(0.0, south_top, width as f64, cell_h);
            cr.fill().unwrap();
        }

        // 4) Puentes (Norte, Central, Sur)
        //
        // Mapeamos:
        // Puente Norte  -> columna 1
        // Puente Central-> columna 2
        // Puente Sur    -> columna 3
        //
        // Visualmente:
        //  - Pte Norte: un carril (rectángulo gris).
        //  - Pte Central: un carril con un “cono” de ceda (triangulito).
        //  - Pte Sur: dos carriles (dos rectángulos paralelos).
        let bridge_cols = [1usize, 2, 3];

        for (idx, col) in bridge_cols.iter().enumerate() {
            if *col >= grid_w {
                continue;
            }

            let x = *col as f64 * cell_w;
            let w = cell_w;

            match idx {
                // Puente Norte: simple
                0 => {
                    cr.set_source_rgb(0.6, 0.6, 0.6);
                    cr.rectangle(
                        x,
                        0.0,
                        w,
                        height as f64,
                    );
                    cr.fill().unwrap();
                }
                // Puente Central: con "ceda" (triangulito rojo en el lado sur)
                1 => {
                    cr.set_source_rgb(0.6, 0.6, 0.6);
                    cr.rectangle(
                        x,
                        0.0,
                        w,
                        height as f64,
                    );
                    cr.fill().unwrap();

                    // Triángulo de ceda en la parte inferior
                    cr.set_source_rgb(1.0, 0.8, 0.8);
                    let base_y = (grid_h as f64 - 0.2) * cell_h;
                    cr.move_to(x + w * 0.1, base_y);
                    cr.line_to(x + w * 0.9, base_y);
                    cr.line_to(x + w * 0.5, base_y - cell_h * 0.6);
                    cr.close_path();
                    cr.fill().unwrap();
                }
                // Puente Sur: dos carriles
                2 => {
                    // carril izquierdo
                    cr.set_source_rgb(0.5, 0.5, 0.5);
                    cr.rectangle(
                        x + w * 0.05,
                        0.0,
                        w * 0.4,
                        height as f64,
                    );
                    cr.fill().unwrap();

                    // carril derecho
                    cr.set_source_rgb(0.5, 0.5, 0.5);
                    cr.rectangle(
                        x + w * 0.55,
                        0.0,
                        w * 0.4,
                        height as f64,
                    );
                    cr.fill().unwrap();
                }
                _ => {}
            }
        }

        // 5) Cuadrícula encima
        cr.set_source_rgb(0.3, 0.3, 0.3);
        cr.set_line_width(1.0);
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

        for v in vehicles {
            let (vx, vy) = v.pos;
            let x = vx as f64 * cell_w;
            let y = vy as f64 * cell_h;

            // Elegir sprite según tipo
            let base_pixbuf = match v.vtype {
                VehicleType::Car => &sprites.car,
                VehicleType::Ambulance => &sprites.ambulance,
                VehicleType::Boat => &sprites.boat,
                VehicleType::SupplyTruck => &sprites.truck,
            };

            // Escalar sprite al tamaño de la celda
            let scaled = base_pixbuf
                .scale_simple(
                    cell_w as i32,
                    cell_h as i32,
                    gdk_pixbuf::InterpType::Bilinear,
                )
                .expect("No se pudo escalar sprite");

            // Dibujar el pixbuf en la celda
            cr.set_source_pixbuf(&scaled, x, y);
            cr.paint().unwrap();
        }
    });

    window.set_child(Some(&drawing_area));

    // Timer de repintado
    let drawing_area_clone = drawing_area.clone();
    timeout_add_local(std::time::Duration::from_millis(100), move || {
        drawing_area_clone.queue_draw();
        glib::ControlFlow::Continue
    });

    window.present();
}