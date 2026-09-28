mod math;
mod renderer;
mod scene;

use raylib::prelude::*;

use math::Vec3;
use renderer::{Camera, Renderer};
use scene::Scene;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const INTERACTIVE_WIDTH: usize = 400;
const INTERACTIVE_HEIGHT: usize = 225;
const QUALITY_WIDTH: usize = 800;
const QUALITY_HEIGHT: usize = 450;
const QUALITY_DELAY_SECONDS: f64 = 0.30;

fn main() {
    if std::env::args().any(|argument| argument == "--render-preview") {
        render_preview();
        return;
    }

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("El Santuario del Cristal | Raytracing en Rust")
        .resizable()
        .build();
    rl.set_target_fps(60);

    let interactive_image = Image::gen_image_color(
        INTERACTIVE_WIDTH as i32,
        INTERACTIVE_HEIGHT as i32,
        Color::BLACK,
    );
    let quality_image =
        Image::gen_image_color(QUALITY_WIDTH as i32, QUALITY_HEIGHT as i32, Color::BLACK);
    let mut interactive_texture = rl
        .load_texture_from_image(&thread, &interactive_image)
        .expect("No se pudo crear el framebuffer");
    let mut quality_texture = rl
        .load_texture_from_image(&thread, &quality_image)
        .expect("No se pudo crear el framebuffer de calidad");
    interactive_texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    quality_texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);

    let scene = Scene::sanctuary();
    let mut interactive_renderer = Renderer::new(INTERACTIVE_WIDTH, INTERACTIVE_HEIGHT);
    let mut quality_renderer = Renderer::new(QUALITY_WIDTH, QUALITY_HEIGHT);
    quality_renderer.max_bounces = 4;
    quality_renderer.samples_per_pixel = 2;
    let mut camera = Camera {
        target: Vec3::new(0.3, -0.1, 0.1),
        yaw: -0.72,
        pitch: 0.40,
        distance: 15.0,
        fov_degrees: 43.0,
    };
    let mut auto_rotate = false;
    let mut show_help = true;
    let mut interactive_dirty = true;
    let mut quality_ready = false;
    let mut last_camera_change = rl.get_time();

    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.05);
        let mouse_delta = rl.get_mouse_delta();
        let wheel = rl.get_mouse_wheel_move();
        let mut camera_changed = false;

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            && (mouse_delta.x.abs() > 0.0 || mouse_delta.y.abs() > 0.0)
        {
            camera.yaw -= mouse_delta.x * 0.006;
            camera.pitch = (camera.pitch + mouse_delta.y * 0.006).clamp(-0.05, 1.10);
            auto_rotate = false;
            camera_changed = true;
        }
        if wheel.abs() > 0.0 {
            camera.distance = (camera.distance - wheel * 0.85).clamp(8.5, 24.0);
            camera_changed = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            auto_rotate = !auto_rotate;
            camera_changed = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            show_help = !show_help;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            camera.yaw = -0.72;
            camera.pitch = 0.40;
            camera.distance = 15.0;
            auto_rotate = false;
            camera_changed = true;
        }
        if auto_rotate {
            camera.yaw += dt * 0.13;
            camera_changed = true;
        }

        let time = rl.get_time() as f32;
        if camera_changed {
            interactive_dirty = true;
            quality_ready = false;
            last_camera_change = f64::from(time);
        }
        if interactive_dirty {
            interactive_renderer.render(&scene, &camera, time);
            interactive_texture
                .update_texture(&interactive_renderer.pixels)
                .expect("No se pudo actualizar el framebuffer interactivo");
            interactive_dirty = false;
        }
        if !auto_rotate
            && !quality_ready
            && f64::from(time) - last_camera_change >= QUALITY_DELAY_SECONDS
        {
            quality_renderer.render(&scene, &camera, time);
            quality_texture
                .update_texture(&quality_renderer.pixels)
                .expect("No se pudo actualizar el framebuffer de calidad");
            quality_ready = true;
        }

        let screen_width = rl.get_screen_width() as f32;
        let screen_height = rl.get_screen_height() as f32;
        let (render_width, render_height, render_mode) = if quality_ready {
            (QUALITY_WIDTH, QUALITY_HEIGHT, "CALIDAD")
        } else {
            (INTERACTIVE_WIDTH, INTERACTIVE_HEIGHT, "INTERACTIVO")
        };
        let source = Rectangle::new(0.0, 0.0, render_width as f32, render_height as f32);
        let destination = Rectangle::new(0.0, 0.0, screen_width, screen_height);

        let fps = rl.get_fps();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(11, 12, 22, 255));
        if quality_ready {
            d.draw_texture_pro(
                &quality_texture,
                source,
                destination,
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        } else {
            d.draw_texture_pro(
                &interactive_texture,
                source,
                destination,
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }

        d.draw_rectangle_gradient_v(
            0,
            0,
            screen_width as i32,
            105,
            Color::new(8, 10, 22, 195),
            Color::new(8, 10, 22, 0),
        );
        d.draw_text(
            "EL SANTUARIO DEL CRISTAL",
            32,
            25,
            28,
            Color::new(247, 219, 162, 255),
        );
        d.draw_text(
            "DIORAMA RAYTRACED  /  RUST + RAYLIB",
            34,
            59,
            14,
            Color::new(139, 207, 225, 255),
        );
        d.draw_text(
            &format!(
                "{} FPS  |  {}  {}x{}  |  {} rebotes",
                fps, render_mode, render_width, render_height, quality_renderer.max_bounces
            ),
            32,
            screen_height as i32 - 30,
            15,
            Color::new(235, 239, 244, 220),
        );

        if show_help {
            let panel_x = screen_width as i32 - 310;
            d.draw_rectangle(panel_x, 22, 286, 116, Color::new(7, 10, 20, 190));
            d.draw_rectangle_lines(panel_x, 22, 286, 116, Color::new(103, 177, 192, 130));
            d.draw_text(
                "CONTROLES",
                panel_x + 18,
                36,
                17,
                Color::new(247, 219, 162, 255),
            );
            d.draw_text(
                "Arrastrar     Orbitar camara",
                panel_x + 18,
                63,
                14,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Rueda         Acercar / alejar",
                panel_x + 18,
                83,
                14,
                Color::RAYWHITE,
            );
            d.draw_text(
                "ESPACIO       Rotacion automatica",
                panel_x + 18,
                103,
                14,
                Color::RAYWHITE,
            );
            d.draw_text(
                "R / H         Reiniciar / ayuda",
                panel_x + 18,
                123,
                14,
                Color::RAYWHITE,
            );
        }
    }
}

fn render_preview() {
    let scene = Scene::sanctuary();
    let camera = Camera {
        target: Vec3::new(0.3, -0.1, 0.1),
        yaw: -0.72,
        pitch: 0.40,
        distance: 15.0,
        fov_degrees: 43.0,
    };
    let mut renderer = Renderer::new(960, 540);
    renderer.max_bounces = 4;
    renderer.samples_per_pixel = 4;
    renderer.render(&scene, &camera, 0.0);
    std::fs::create_dir_all("screenshots").expect("No se pudo crear screenshots/");
    renderer
        .save_bmp("screenshots/preview.bmp")
        .expect("No se pudo guardar la captura");
    println!("Captura guardada en screenshots/preview.bmp");
}
