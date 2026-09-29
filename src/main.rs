mod character;
mod math;
mod renderer;
mod scene;

use raylib::prelude::*;

use character::{Character, DEFAULT_CHARACTER_SCALE};
use math::Vec3;
use renderer::{Camera, Renderer};
use scene::Scene;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const RENDER_WIDTH: usize = 512;
const RENDER_HEIGHT: usize = 288;
const QUALITY_DELAY_SECONDS: f64 = 0.25;

fn main() {
    if std::env::args().any(|argument| argument == "--benchmark") {
        render_benchmark();
        return;
    }
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

    let image = Image::gen_image_color(RENDER_WIDTH as i32, RENDER_HEIGHT as i32, Color::BLACK);
    let mut texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("No se pudo crear el framebuffer HD");
    texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);

    let mut scene = Scene::sanctuary();
    let mut renderer = Renderer::new(RENDER_WIDTH, RENDER_HEIGHT);
    let mut camera = Camera {
        target: Vec3::new(0.3, -0.1, 0.1),
        yaw: -0.72,
        pitch: 0.40,
        distance: 15.0,
        fov_degrees: 43.0,
    };
    let mut auto_rotate = false;
    let mut show_help = true;
    let mut last_camera_change = rl.get_time();
    let character_spawn = Vec3::new(-2.15, -0.8, 2.50);
    let mut player = Character::new(character_spawn, 0.0, DEFAULT_CHARACTER_SCALE);
    let mut follow_character = false;
    let mut render_dirty = false;
    let mut quality_ready = false;
    let mut notice: Option<(&str, f64)> = Some(("ENCUENTRA LOS 3 FRAGMENTOS", 3.5));

    // Always present complete frames. No interlacing or mixed camera positions.
    renderer.max_bounces = 0;
    renderer.samples_per_pixel = 1;
    renderer.render(&scene, &camera, 0.0);
    texture
        .update_texture(&renderer.pixels)
        .expect("No se pudo cargar el primer cuadro HD");

    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.10);
        let mouse_delta = rl.get_mouse_delta();
        let wheel = rl.get_mouse_wheel_move();
        let mut camera_changed = false;

        let camera_forward = Vec3::new(-camera.yaw.sin(), 0.0, -camera.yaw.cos());
        let camera_right = Vec3::new(-camera_forward.z, 0.0, camera_forward.x);
        let mut movement = Vec3::ZERO;
        if rl.is_key_down(KeyboardKey::KEY_W) {
            movement += camera_forward;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            movement += -camera_forward;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            movement += camera_right;
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            movement += -camera_right;
        }
        let player_was_moving = player.is_moving;
        if movement.length() > 0.0 {
            movement = movement.normalized();
        }
        let previous_position = player.position;
        player.update(dt, movement);
        if player.is_moving || player_was_moving {
            let desired_position = player.position;

            // Resolve X and Z separately so the character slides along walls.
            player.position = Vec3::new(
                desired_position.x,
                Scene::ground_height(desired_position.x, previous_position.z),
                previous_position.z,
            );
            player.rebuild_geometry();
            let (x_min, x_max) = player.bounding_box();
            if scene.collides_with_world(x_min, x_max) {
                player.position.x = previous_position.x;
            }

            player.position.z = desired_position.z;
            player.position.y = Scene::ground_height(player.position.x, player.position.z);
            player.rebuild_geometry();
            let (z_min, z_max) = player.bounding_box();
            if scene.collides_with_world(z_min, z_max) {
                player.position.z = previous_position.z;
                player.position.y = Scene::ground_height(player.position.x, player.position.z);
            }

            player.rebuild_geometry();
            scene.update_character(player.blocks());
            let collected = scene.collect_near(player.position);
            let activated = scene.try_activate_crystal(player.position);
            if collected {
                notice = Some(("FRAGMENTO RECUPERADO", rl.get_time() + 1.8));
            }
            if activated {
                notice = Some(("EL SANTUARIO HA DESPERTADO", rl.get_time() + 3.2));
            }
            if collected || activated {
                quality_ready = false;
            }
            camera_changed = true;
        }

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            && (mouse_delta.x.abs() > 0.0 || mouse_delta.y.abs() > 0.0)
        {
            camera.yaw -= mouse_delta.x * 0.006;
            camera.pitch = (camera.pitch + mouse_delta.y * 0.006).clamp(-0.05, 1.10);
            auto_rotate = false;
            notice = Some(("ENCUENTRA LOS 3 FRAGMENTOS", rl.get_time() + 3.5));
            camera_changed = true;
        }
        if wheel.abs() > 0.0 {
            camera.distance = (camera.distance - wheel * 0.85).clamp(8.5, 24.0);
            camera_changed = true;
        }
        let camera_speed = 1.35 * dt;
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.yaw -= camera_speed;
            auto_rotate = false;
            camera_changed = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.yaw += camera_speed;
            auto_rotate = false;
            camera_changed = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            camera.pitch = (camera.pitch + camera_speed * 0.72).clamp(-0.05, 1.10);
            auto_rotate = false;
            camera_changed = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.pitch = (camera.pitch - camera_speed * 0.72).clamp(-0.05, 1.10);
            auto_rotate = false;
            camera_changed = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            auto_rotate = !auto_rotate;
            camera_changed = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            show_help = !show_help;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            follow_character = !follow_character;
            camera_changed = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            player.reset(character_spawn);
            scene.reset_puzzle();
            scene.update_character(player.blocks());
            camera.yaw = -0.72;
            camera.pitch = 0.40;
            camera.distance = 15.0;
            camera.target = Vec3::new(0.3, -0.1, 0.1);
            follow_character = false;
            auto_rotate = false;
            camera_changed = true;
        }
        if auto_rotate {
            camera.yaw += dt * 0.13;
            camera_changed = true;
        }
        if follow_character {
            let desired_target = player.position + Vec3::new(0.0, 0.85 * player.scale, 0.0);
            let next_target = camera.target.lerp(desired_target, (dt * 6.0).min(1.0));
            if (next_target - camera.target).length() > 0.0001 {
                camera.target = next_target;
                camera_changed = true;
            }
        }

        let time = rl.get_time() as f32;
        if camera_changed {
            last_camera_change = f64::from(time);
            render_dirty = true;
            quality_ready = false;
        }
        if render_dirty {
            renderer.max_bounces = 0;
            renderer.samples_per_pixel = 1;
            renderer.render(&scene, &camera, time);
            texture
                .update_texture(&renderer.pixels)
                .expect("No se pudo actualizar el framebuffer estable");
            render_dirty = false;
        }
        let settled = !auto_rotate && f64::from(time) - last_camera_change >= QUALITY_DELAY_SECONDS;
        if settled && !quality_ready {
            renderer.max_bounces = 3;
            renderer.samples_per_pixel = 3;
            renderer.render(&scene, &camera, time);
            texture
                .update_texture(&renderer.pixels)
                .expect("No se pudo actualizar el cuadro de calidad");
            quality_ready = true;
        }

        let screen_width = rl.get_screen_width() as f32;
        let screen_height = rl.get_screen_height() as f32;
        let render_mode = if quality_ready {
            "CALIDAD ESTABLE"
        } else {
            "INTERACTIVO ESTABLE"
        };
        let source = Rectangle::new(0.0, 0.0, RENDER_WIDTH as f32, RENDER_HEIGHT as f32);
        let destination = Rectangle::new(0.0, 0.0, screen_width, screen_height);
        let puzzle = scene.puzzle_status();

        let fps = rl.get_fps();
        let notice_layout = if let Some((message, expires_at)) = notice {
            if f64::from(time) < expires_at {
                Some((message, rl.measure_text(message, 18)))
            } else {
                notice = None;
                None
            }
        } else {
            None
        };
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(11, 12, 22, 255));
        d.draw_texture_pro(
            &texture,
            source,
            destination,
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );

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
                fps, render_mode, RENDER_WIDTH, RENDER_HEIGHT, renderer.max_bounces
            ),
            32,
            screen_height as i32 - 30,
            15,
            Color::new(235, 239, 244, 220),
        );

        let objective_x = screen_width as i32 - 382;
        let objective_y = screen_height as i32 - 94;
        let objective_color = if puzzle.crystal_activated {
            Color::new(112, 238, 222, 255)
        } else {
            Color::new(247, 219, 162, 255)
        };
        let objective_text = if puzzle.crystal_activated {
            "SANTUARIO ACTIVADO"
        } else if puzzle.collected == puzzle.total {
            "REGRESA AL CRISTAL CENTRAL"
        } else {
            "RECOGE LOS FRAGMENTOS"
        };
        d.draw_rectangle(
            objective_x,
            objective_y,
            350,
            56,
            Color::new(7, 10, 20, 205),
        );
        d.draw_rectangle_lines(
            objective_x,
            objective_y,
            350,
            56,
            Color::new(103, 177, 192, 150),
        );
        d.draw_text(
            objective_text,
            objective_x + 14,
            objective_y + 10,
            15,
            objective_color,
        );
        d.draw_text(
            &format!("FRAGMENTOS  {}/{}", puzzle.collected, puzzle.total),
            objective_x + 14,
            objective_y + 32,
            14,
            Color::RAYWHITE,
        );
        for index in 0..puzzle.total {
            let color = if index < puzzle.collected {
                Color::new(83, 231, 226, 255)
            } else {
                Color::new(68, 88, 106, 255)
            };
            d.draw_circle(
                objective_x + 274 + index as i32 * 22,
                objective_y + 38,
                5.0,
                color,
            );
        }

        if let Some((message, text_width)) = notice_layout {
            let banner_x = (screen_width as i32 - text_width) / 2;
            d.draw_rectangle(
                banner_x - 18,
                104,
                text_width + 36,
                40,
                Color::new(7, 10, 20, 205),
            );
            d.draw_rectangle_lines(
                banner_x - 18,
                104,
                text_width + 36,
                40,
                Color::new(83, 231, 226, 170),
            );
            d.draw_text(message, banner_x, 115, 18, Color::new(178, 255, 244, 255));
        }

        if show_help {
            let panel_x = screen_width as i32 - 310;
            d.draw_rectangle(panel_x, 22, 286, 176, Color::new(7, 10, 20, 190));
            d.draw_rectangle_lines(panel_x, 22, 286, 176, Color::new(103, 177, 192, 130));
            d.draw_text(
                "CONTROLES",
                panel_x + 18,
                36,
                17,
                Color::new(247, 219, 162, 255),
            );
            d.draw_text(
                "WASD          Mover explorador",
                panel_x + 18,
                63,
                14,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Arrastrar     Orbitar camara",
                panel_x + 18,
                83,
                14,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Rueda         Acercar / alejar",
                panel_x + 18,
                103,
                14,
                Color::RAYWHITE,
            );
            d.draw_text(
                "F / ESPACIO   Seguir / autorrotar",
                panel_x + 18,
                123,
                14,
                Color::RAYWHITE,
            );
            d.draw_text(
                "Flechas       Orbitar camara",
                panel_x + 18,
                143,
                14,
                Color::RAYWHITE,
            );
            d.draw_text(
                "R / H         Reiniciar / ayuda",
                panel_x + 18,
                163,
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

fn render_benchmark() {
    use std::time::Instant;

    let scene = Scene::sanctuary();
    let mut camera = Camera {
        target: Vec3::new(0.3, -0.1, 0.1),
        yaw: -0.72,
        pitch: 0.40,
        distance: 15.0,
        fov_degrees: 43.0,
    };
    let mut renderer = Renderer::new(RENDER_WIDTH, RENDER_HEIGHT);
    renderer.max_bounces = 0;
    renderer.samples_per_pixel = 1;
    renderer.render(&scene, &camera, 0.0);

    const FRAMES: usize = 12;
    let started = Instant::now();
    for frame in 0..FRAMES {
        camera.yaw += 0.012;
        renderer.render(&scene, &camera, frame as f32 / 20.0);
    }
    let elapsed = started.elapsed().as_secs_f64();
    let fps = FRAMES as f64 / elapsed;
    println!(
        "Benchmark interactivo: {fps:.1} FPS ({:.2} ms/cuadro, {}x{}, 1 muestra, 0 rebotes)",
        elapsed * 1000.0 / FRAMES as f64,
        RENDER_WIDTH,
        RENDER_HEIGHT
    );
}
