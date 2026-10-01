mod audio_fx;
mod character;
mod enemy;
mod math;
mod presentation;
mod renderer;
mod scene;

use raylib::prelude::*;

use character::{Character, DEFAULT_CHARACTER_SCALE};
use enemy::Enemy;
use math::Vec3;
use presentation::{
    ParticleSystem, draw_centered, draw_star, ease_out_back, level_copy, smoothstep,
};
use renderer::{Camera, Renderer};
use scene::{Block, LevelKind, Scene};

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const RENDER_WIDTH: usize = 512;
const RENDER_HEIGHT: usize = 288;
const CHARACTER_SPAWN: Vec3 = Vec3::new(-2.15, -0.8, 2.50);
const HILLS_DEMO_WAYPOINTS: [Vec3; 21] = [
    Vec3::new(-2.15, -0.8, 2.50),
    Vec3::new(-3.05, -0.8, 1.45),
    Vec3::new(-3.15, -0.8, 0.55),
    Vec3::new(-3.70, -0.8, -0.20),
    Vec3::new(-3.70, -0.8, -2.35),
    Vec3::new(-2.15, -0.4, -2.35),
    Vec3::new(-0.60, -0.4, -2.35),
    Vec3::new(3.55, -0.8, -1.30),
    Vec3::new(3.55, -0.72, -2.75),
    Vec3::new(3.55, 1.30, -2.75),
    Vec3::new(4.45, 1.30, -2.75),
    Vec3::new(5.18, 1.30, -3.15),
    Vec3::new(4.45, 1.30, -2.75),
    Vec3::new(3.55, 1.30, -2.75),
    Vec3::new(3.55, -0.72, -2.75),
    Vec3::new(3.55, -0.8, -1.30),
    Vec3::new(2.70, -0.8, -1.55),
    Vec3::new(-0.36, -0.4, -1.68),
    Vec3::new(-0.36, -0.1, -1.02),
    Vec3::new(0.55, -0.1, -0.65),
    Vec3::new(3.15, -0.1, 1.15),
];
const HAUNTED_DEMO_WAYPOINTS: [Vec3; 32] = [
    Vec3::new(-2.15, -0.8, 2.50),
    Vec3::new(-4.05, -0.8, 2.45),
    Vec3::new(-3.25, -0.8, 2.45),
    Vec3::new(-3.25, -0.8, 0.10),
    Vec3::new(-3.35, -0.64, -0.56),
    Vec3::new(-3.35, -0.45, -0.82),
    Vec3::new(-3.35, -0.45, -1.85),
    Vec3::new(-3.35, -0.8, -0.35),
    Vec3::new(-3.65, -0.8, 3.25),
    Vec3::new(-1.20, -0.8, 3.25),
    Vec3::new(-1.20, -0.8, 4.00),
    Vec3::new(5.20, -0.8, 4.00),
    Vec3::new(5.20, -0.8, 3.40),
    Vec3::new(5.20, -0.72, 2.75),
    Vec3::new(5.20, 1.40, 2.75),
    Vec3::new(5.20, 1.40, 1.90),
    Vec3::new(5.18, 1.40, 0.90),
    Vec3::new(5.18, 1.40, 0.05),
    Vec3::new(5.18, 1.40, 0.90),
    Vec3::new(5.20, 1.40, 1.90),
    Vec3::new(5.20, 1.40, 2.75),
    Vec3::new(5.20, -0.72, 2.75),
    Vec3::new(5.20, -0.8, 4.00),
    Vec3::new(3.30, -0.8, 4.00),
    Vec3::new(3.30, -0.8, 3.25),
    Vec3::new(1.70, -0.8, 3.15),
    Vec3::new(1.70, -0.62, 2.96),
    Vec3::new(1.70, -0.10, 2.54),
    Vec3::new(1.70, -0.10, 1.90),
    Vec3::new(2.20, -0.10, 1.85),
    Vec3::new(2.20, -0.10, 0.65),
    Vec3::new(2.60, -0.10, 0.80),
];
const AQUATIC_DEMO_WAYPOINTS: [Vec3; 22] = [
    Vec3::new(-2.15, -0.8, 2.50),
    Vec3::new(-4.05, -0.8, 2.45),
    Vec3::new(-4.05, -0.8, 0.20),
    Vec3::new(-3.10, -0.40, -1.90),
    Vec3::new(-3.10, -0.8, 0.55),
    Vec3::new(-2.00, -0.8, 2.18),
    Vec3::new(-1.45, -0.40, 2.18),
    Vec3::new(0.20, -0.10, 2.18),
    Vec3::new(0.45, -0.10, -0.20),
    Vec3::new(2.75, -0.10, -0.30),
    Vec3::new(2.75, -0.08, -1.35),
    Vec3::new(2.75, 1.20, -1.35),
    Vec3::new(3.65, 1.20, -1.35),
    Vec3::new(4.35, 1.20, -1.35),
    Vec3::new(5.12, 1.20, -2.85),
    Vec3::new(4.35, 1.20, -1.35),
    Vec3::new(3.65, 1.20, -1.35),
    Vec3::new(2.75, 1.20, -1.35),
    Vec3::new(2.75, -0.08, -1.35),
    Vec3::new(2.75, -0.10, -0.30),
    Vec3::new(3.25, -0.10, 0.15),
    Vec3::new(3.15, -0.10, 1.15),
];

fn demo_waypoints(level: LevelKind) -> &'static [Vec3] {
    match level {
        LevelKind::Hills => &HILLS_DEMO_WAYPOINTS,
        LevelKind::Haunted => &HAUNTED_DEMO_WAYPOINTS,
        LevelKind::Aquatic => &AQUATIC_DEMO_WAYPOINTS,
    }
}

const fn music_track_for_level(level: LevelKind) -> audio_fx::MusicTrack {
    match level {
        LevelKind::Hills => audio_fx::MusicTrack::Hills,
        LevelKind::Haunted => audio_fx::MusicTrack::Haunted,
        LevelKind::Aquatic => audio_fx::MusicTrack::Beach,
    }
}

#[derive(Clone, Copy)]
enum GameState {
    Title,
    LevelIntro { started: f64 },
    Playing,
    CrystalSequence { started: f64 },
    GameOver { until: f64 },
    CampaignClear,
}

fn main() {
    if std::env::args().any(|argument| argument == "--export-audio-effects") {
        audio_fx::export_effects(std::path::Path::new("tools/audio_fx"))
            .expect("No se pudieron exportar los efectos de sonido");
        return;
    }
    if std::env::args().any(|argument| argument == "--benchmark") {
        render_benchmark();
        return;
    }
    if std::env::args().any(|argument| argument == "--render-preview") {
        render_preview();
        return;
    }
    let capture_ui = std::env::args().any(|argument| argument == "--capture-ui");
    let record_demo = std::env::args().any(|argument| argument == "--record-demo");

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("El Santuario del Cristal | Raytracing en Rust")
        .resizable()
        .build();
    rl.set_target_fps(60);
    if record_demo {
        rl.hide_cursor();
    }

    let audio = RaylibAudio::init_audio_device().expect("No se pudo inicializar el audio");
    let mut music = audio_fx::MusicDirector::new(&audio);
    let game_sounds = audio_fx::GameSounds::new(&audio);

    let image = Image::gen_image_color(RENDER_WIDTH as i32, RENDER_HEIGHT as i32, Color::BLACK);
    let mut texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("No se pudo crear el framebuffer HD");
    texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    let mut post_shader = rl.load_shader_from_memory(
        &thread,
        None,
        Some(include_str!("../assets/shaders/sanctuary_post.fs")),
    );
    let shader_time_location = post_shader.get_shader_location("uTime");
    let shader_active_location = post_shader.get_shader_location("uSanctuaryActive");
    let shader_theme_location = post_shader.get_shader_location("uTheme");
    let ui_font = rl
        .load_font_from_memory(
            &thread,
            ".ttf",
            include_bytes!("../assets/fonts/Fredoka.ttf"),
            72,
            Some(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 áéíóúüñÁÉÍÓÚÜÑ¿¡.,:;!?()/-+×|",
            ),
        )
        .expect("No se pudo cargar la tipografia Fredoka");

    let mut level = LevelKind::Hills;
    let (mut scene, mut player, mut enemies, mut enemy_blocks) = build_level(level);
    let mut renderer = Renderer::new(RENDER_WIDTH, RENDER_HEIGHT);
    let mut camera = Camera {
        target: Vec3::new(0.3, -0.1, 0.1),
        yaw: -0.72,
        pitch: 0.40,
        distance: 18.5,
        fov_degrees: 43.0,
    };
    let mut shader_enabled = true;
    let mut cinematic_mode = false;
    let mut demo_waypoint = 1_usize;
    let mut lives = 3_u32;
    let mut hits_taken = 0_u32;
    let mut campaign_started_at = 0.0_f64;
    let mut campaign_elapsed = 0.0_f64;
    let mut invulnerable_until = 0.0_f64;
    let mut game_state = GameState::Title;
    let mut notice: Option<(&str, f64)> = None;
    let mut particles = ParticleSystem::new();
    let mut capture_stage = 0_u8;
    let mut capture_stage_frames = 0_u8;
    let mut record_demo_started = false;
    let mut record_demo_exit_at = None;

    renderer.max_bounces = 0;
    renderer.samples_per_pixel = 1;
    renderer.render(&scene, &camera, 0.0);
    texture
        .update_texture(&renderer.pixels)
        .expect("No se pudo cargar el primer cuadro HD");

    while !rl.window_should_close() {
        music.update();
        let dt = rl.get_frame_time().min(0.10);
        let now = rl.get_time();
        let mouse_delta = rl.get_mouse_delta();
        let wheel = rl.get_mouse_wheel_move();
        let screen_width = rl.get_screen_width() as f32;
        let screen_height = rl.get_screen_height() as f32;

        if record_demo_exit_at.is_some_and(|exit_at| now >= exit_at) {
            break;
        }

        if capture_ui {
            capture_stage_frames += 1;
            if capture_stage_frames >= 8 {
                capture_stage_frames = 0;
                match capture_stage {
                    0 => {
                        rl.take_screenshot(&thread, "screenshots/ui_title.png");
                        game_state = GameState::LevelIntro { started: now - 1.0 };
                        capture_stage = 1;
                    }
                    1 => {
                        rl.take_screenshot(&thread, "screenshots/ui_level_intro.png");
                        game_state = GameState::CrystalSequence { started: now - 1.0 };
                        particles.burst(
                            Vector2::new(screen_width * 0.5, screen_height * 0.45),
                            level,
                            72,
                        );
                        capture_stage = 2;
                    }
                    2 => {
                        rl.take_screenshot(&thread, "screenshots/ui_crystal.png");
                        campaign_elapsed = 198.0;
                        hits_taken = 1;
                        game_state = GameState::CampaignClear;
                        capture_stage = 3;
                    }
                    3 => {
                        rl.take_screenshot(&thread, "screenshots/ui_results.png");
                        break;
                    }
                    _ => unreachable!(),
                }
            }
        }

        let pressed_c = rl.is_key_pressed(KeyboardKey::KEY_C);
        let automatic_demo_start = record_demo
            && !record_demo_started
            && matches!(game_state, GameState::Title)
            && now >= 6.0;
        let start_from_title = matches!(game_state, GameState::Title)
            && (rl.is_key_pressed(KeyboardKey::KEY_ENTER) || pressed_c || automatic_demo_start);
        if start_from_title {
            game_sounds.play_begin();
            music.switch_to(audio_fx::MusicTrack::Hills);
            cinematic_mode = pressed_c || automatic_demo_start;
            record_demo_started |= automatic_demo_start;
            level = LevelKind::Hills;
            (scene, player, enemies, enemy_blocks) = build_level(level);
            lives = 3;
            hits_taken = 0;
            campaign_started_at = now;
            demo_waypoint = 1;
            game_state = GameState::LevelIntro { started: now };
            invulnerable_until = now + 1.0;
            camera.yaw = -0.72;
            camera.pitch = 0.44;
            camera.distance = if cinematic_mode { 13.5 } else { 12.0 };
            camera.target = player.position + Vec3::new(0.0, 0.55, 0.0);
            particles.clear();
            notice = None;
        } else if pressed_c && !matches!(game_state, GameState::Title) {
            music.switch_to(audio_fx::MusicTrack::Hills);
            cinematic_mode = !cinematic_mode;
            level = LevelKind::Hills;
            (scene, player, enemies, enemy_blocks) = build_level(level);
            lives = 3;
            hits_taken = 0;
            campaign_started_at = now;
            demo_waypoint = 1;
            game_state = GameState::LevelIntro { started: now };
            invulnerable_until = now + 1.0;
            camera.yaw = -0.72;
            camera.pitch = 0.44;
            camera.distance = if cinematic_mode { 13.5 } else { 12.0 };
            camera.target = player.position + Vec3::new(0.0, 0.55, 0.0);
            particles.clear();
            notice = None;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            if matches!(game_state, GameState::CampaignClear) {
                level = LevelKind::Hills;
                (scene, player, enemies, enemy_blocks) = build_level(level);
                game_state = GameState::Title;
                cinematic_mode = false;
                music.switch_to(audio_fx::MusicTrack::Title);
                camera.distance = 18.5;
                camera.target = Vec3::new(0.3, -0.1, 0.1);
                particles.clear();
            } else if !matches!(game_state, GameState::Title) {
                (scene, player, enemies, enemy_blocks) = build_level(level);
                lives = 3;
                demo_waypoint = 1;
                game_state = GameState::LevelIntro { started: now };
                invulnerable_until = now + 1.0;
                notice = Some(("Listos para intentarlo otra vez", now + 2.0));
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            shader_enabled = !shader_enabled;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            music.toggle_mute();
        }

        match game_state {
            GameState::LevelIntro { started } if now - started >= 3.1 => {
                game_state = GameState::Playing;
            }
            GameState::CrystalSequence { started } if now - started >= 3.4 => {
                if let Some(next_level) = level.next() {
                    level = next_level;
                    music.switch_to(music_track_for_level(level));
                    (scene, player, enemies, enemy_blocks) = build_level(level);
                    demo_waypoint = 1;
                    invulnerable_until = now + 1.5;
                    game_state = GameState::LevelIntro { started: now };
                    camera.distance = 13.5;
                    camera.target = player.position + Vec3::new(0.0, 0.55, 0.0);
                    particles.clear();
                    notice = None;
                } else {
                    campaign_elapsed = now - campaign_started_at;
                    game_sounds.play_victory();
                    music.switch_to(audio_fx::MusicTrack::Title);
                    game_state = GameState::CampaignClear;
                    if record_demo {
                        record_demo_exit_at = Some(now + 8.0);
                    }
                }
            }
            GameState::GameOver { until } if now >= until => {
                (scene, player, enemies, enemy_blocks) = build_level(level);
                lives = 3;
                demo_waypoint = 1;
                invulnerable_until = now + 1.5;
                game_state = GameState::LevelIntro { started: now };
                notice = Some(("Tod vuelve a la aventura", now + 2.0));
            }
            _ => {}
        }

        let riding_elevator = scene.is_riding_elevator(player.position);
        scene.update_mechanisms(now as f32);
        if riding_elevator {
            player.position.y = scene.elevator_top();
            player.rebuild_geometry();
        }

        let camera_forward = Vec3::new(-camera.yaw.sin(), 0.0, -camera.yaw.cos());
        let camera_right = Vec3::new(-camera_forward.z, 0.0, camera_forward.x);
        let mut movement = Vec3::ZERO;
        if matches!(game_state, GameState::Playing) {
            if cinematic_mode {
                let waypoints = demo_waypoints(level);
                let target = waypoints[demo_waypoint];
                let delta = target - player.position;
                if delta.length() < 0.14 {
                    demo_waypoint = (demo_waypoint + 1).min(waypoints.len() - 1);
                }
                movement = Vec3::new(delta.x, 0.0, delta.z).normalized();
            } else {
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
            }

            move_player(&mut player, movement, dt, &scene);
            let collected = scene.collect_near(player.position);
            let activated = scene.try_activate_crystal(player.position);
            if collected {
                game_sounds.play_collect();
                let burst_position = world_to_screen(
                    &camera,
                    player.position + Vec3::new(0.0, 0.65, 0.0),
                    screen_width,
                    screen_height,
                )
                .unwrap_or_else(|| Vector2::new(screen_width * 0.5, screen_height * 0.5));
                particles.burst(burst_position, level, 28);
                if !cinematic_mode {
                    notice = Some(("Fragmento encontrado", now + 1.8));
                }
            }
            if activated {
                game_sounds.play_crystal();
                game_state = GameState::CrystalSequence { started: now };
                let crystal_screen = world_to_screen(
                    &camera,
                    scene.crystal_position() + Vec3::new(0.0, 1.0, 0.0),
                    screen_width,
                    screen_height,
                )
                .unwrap_or_else(|| Vector2::new(screen_width * 0.5, screen_height * 0.45));
                particles.burst(crystal_screen, level, 72);
                notice = None;
            }
        }

        let camera_position = camera.position();
        let camera_view = (camera.target - camera_position).normalized();
        for enemy in &mut enemies {
            let toward_enemy = (enemy.position - camera_position).normalized();
            let observed = camera_view.dot(toward_enemy) > 0.76;
            enemy.update(dt, observed);
        }
        collect_enemy_blocks(&enemies, &mut enemy_blocks);
        scene.update_dynamic(player.blocks(), &enemy_blocks);

        if !cinematic_mode
            && matches!(game_state, GameState::Playing)
            && now >= invulnerable_until
            && enemies.iter().any(|enemy| enemy.touches(player.position))
        {
            game_sounds.play_hit();
            lives = lives.saturating_sub(1);
            hits_taken += 1;
            if lives == 0 {
                game_state = GameState::GameOver { until: now + 2.8 };
                notice = Some(("Tod necesita otro intento", now + 2.8));
            } else {
                player.reset(CHARACTER_SPAWN);
                for enemy in &mut enemies {
                    enemy.reset();
                }
                collect_enemy_blocks(&enemies, &mut enemy_blocks);
                scene.update_dynamic(player.blocks(), &enemy_blocks);
                invulnerable_until = now + 1.8;
                notice = Some(("Cuidado con los guardianes", now + 1.8));
            }
        }

        if matches!(game_state, GameState::Playing)
            && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            && (mouse_delta.x.abs() > 0.0 || mouse_delta.y.abs() > 0.0)
        {
            camera.yaw -= mouse_delta.x * 0.006;
            camera.pitch = (camera.pitch + mouse_delta.y * 0.006).clamp(-0.05, 1.10);
        }
        if matches!(game_state, GameState::Playing) && wheel.abs() > 0.0 {
            camera.distance = (camera.distance - wheel * 0.85).clamp(8.5, 24.0);
        }
        let camera_speed = 1.35 * dt;
        if matches!(game_state, GameState::Playing) && rl.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.yaw -= camera_speed;
        }
        if matches!(game_state, GameState::Playing) && rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.yaw += camera_speed;
        }
        if matches!(game_state, GameState::Playing) && rl.is_key_down(KeyboardKey::KEY_UP) {
            camera.pitch = (camera.pitch + camera_speed * 0.72).clamp(-0.05, 1.10);
        }
        if matches!(game_state, GameState::Playing) && rl.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.pitch = (camera.pitch - camera_speed * 0.72).clamp(-0.05, 1.10);
        }

        match game_state {
            GameState::Title => {
                camera.yaw += dt * 0.16;
                camera.pitch = camera.pitch + (0.42 - camera.pitch) * (dt * 1.2).min(1.0);
                camera.distance = camera.distance + (18.5 - camera.distance) * (dt * 1.2).min(1.0);
                camera.target = camera
                    .target
                    .lerp(Vec3::new(0.2, -0.05, 0.0), (dt * 1.5).min(1.0));
            }
            GameState::CrystalSequence { .. } => {
                camera.yaw += dt * 0.42;
                camera.pitch = camera.pitch + (0.34 - camera.pitch) * (dt * 1.8).min(1.0);
                camera.distance = camera.distance + (8.8 - camera.distance) * (dt * 1.4).min(1.0);
                camera.target = camera.target.lerp(
                    scene.crystal_position() + Vec3::new(0.0, 0.8, 0.0),
                    (dt * 2.5).min(1.0),
                );
            }
            GameState::CampaignClear => {
                camera.yaw += dt * 0.10;
                camera.distance = camera.distance + (13.2 - camera.distance) * (dt * 1.2).min(1.0);
            }
            _ => {
                if cinematic_mode {
                    camera.yaw += dt * 0.10;
                    camera.pitch = camera.pitch + (0.48 - camera.pitch) * (dt * 1.5).min(1.0);
                    camera.distance =
                        camera.distance + (13.5 - camera.distance) * (dt * 1.5).min(1.0);
                }
                let desired_target = player.position + Vec3::new(0.0, 0.78 * player.scale, 0.0);
                let follow_speed = if cinematic_mode { 2.4 } else { 7.0 };
                let next_target = camera
                    .target
                    .lerp(desired_target, (dt * follow_speed).min(1.0));
                if (next_target - camera.target).length() > 0.0001 {
                    camera.target = next_target;
                }
            }
        }

        particles.update(dt, screen_width, screen_height, level);

        let time = now as f32;
        renderer.max_bounces =
            u32::from(cinematic_mode || matches!(game_state, GameState::CrystalSequence { .. }));
        renderer.samples_per_pixel = 1;
        renderer.render(&scene, &camera, time);
        texture
            .update_texture(&renderer.pixels)
            .expect("No se pudo actualizar el framebuffer");

        let render_mode = if cinematic_mode {
            "DEMO RT"
        } else {
            "JUEGO FLUIDO"
        };
        let source = Rectangle::new(0.0, 0.0, RENDER_WIDTH as f32, RENDER_HEIGHT as f32);
        let destination = Rectangle::new(0.0, 0.0, screen_width, screen_height);
        let puzzle = scene.puzzle_status();
        post_shader.set_shader_value(shader_time_location, time);
        post_shader.set_shader_value(shader_theme_location, level.index() as f32);
        post_shader.set_shader_value(
            shader_active_location,
            if puzzle.crystal_activated {
                1.0_f32
            } else {
                0.0_f32
            },
        );

        let fps = rl.get_fps();
        let notice_layout = if !cinematic_mode {
            if let Some((message, expires_at)) = notice {
                if now < expires_at {
                    Some((message, ui_font.measure_text(message, 21.0, 0.6).x))
                } else {
                    notice = None;
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(11, 12, 22, 255));
        if shader_enabled {
            let mut shader_pass = d.begin_shader_mode(&mut post_shader);
            shader_pass.draw_texture_pro(
                &texture,
                source,
                destination,
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        } else {
            d.draw_texture_pro(
                &texture,
                source,
                destination,
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }

        particles.draw(&mut d);

        if !cinematic_mode && matches!(game_state, GameState::Playing) {
            d.draw_rectangle_gradient_v(
                0,
                0,
                screen_width as i32,
                105,
                Color::new(8, 12, 24, 218),
                Color::new(8, 10, 22, 0),
            );
            let (level_name, _, accent) = level_copy(level);
            d.draw_text_ex(
                &ui_font,
                level_name,
                Vector2::new(30.0, 19.0),
                27.0,
                0.8,
                Color::new(255, 239, 204, 255),
            );
            d.draw_text_ex(
                &ui_font,
                &format!("Vidas  {}    |    C  Cinemática    |    M  Música", lives),
                Vector2::new(32.0, 56.0),
                16.0,
                0.4,
                Color::new(160, 225, 232, 255),
            );
            d.draw_text_ex(
                &ui_font,
                &format!(
                    "{} FPS   |   {}  {}×{}   |   {} rebotes",
                    fps, render_mode, RENDER_WIDTH, RENDER_HEIGHT, renderer.max_bounces
                ),
                Vector2::new(30.0, screen_height - 27.0),
                14.0,
                0.3,
                Color::new(235, 239, 244, 220),
            );

            let objective_x = screen_width - 390.0;
            let objective_y = screen_height - 104.0;
            let objective_color = if puzzle.crystal_activated {
                Color::new(112, 238, 222, 255)
            } else {
                accent
            };
            let objective_text = if puzzle.crystal_activated {
                "Nivel completado"
            } else if puzzle.collected == puzzle.total {
                "Regresa al cristal central"
            } else {
                "Encuentra los fragmentos"
            };
            d.draw_rectangle_rounded(
                Rectangle::new(objective_x, objective_y, 360.0, 66.0),
                0.22,
                8,
                Color::new(7, 12, 24, 220),
            );
            d.draw_rectangle_rounded_lines(
                Rectangle::new(objective_x, objective_y, 360.0, 66.0),
                0.22,
                8,
                Color::new(accent.r, accent.g, accent.b, 165),
            );
            d.draw_text_ex(
                &ui_font,
                objective_text,
                Vector2::new(objective_x + 17.0, objective_y + 9.0),
                18.0,
                0.3,
                objective_color,
            );
            d.draw_text_ex(
                &ui_font,
                &format!("Fragmentos  {}/{}", puzzle.collected, puzzle.total),
                Vector2::new(objective_x + 17.0, objective_y + 38.0),
                15.0,
                0.3,
                Color::RAYWHITE,
            );
            for index in 0..puzzle.total {
                let color = if index < puzzle.collected {
                    Color::new(83, 231, 226, 255)
                } else {
                    Color::new(68, 88, 106, 255)
                };
                d.draw_circle(
                    (objective_x + 288.0 + index as f32 * 22.0) as i32,
                    (objective_y + 47.0) as i32,
                    6.0,
                    color,
                );
            }

            if let Some((message, text_width)) = notice_layout {
                let banner_x = (screen_width - text_width) * 0.5;
                d.draw_rectangle_rounded(
                    Rectangle::new(banner_x - 20.0, 104.0, text_width + 40.0, 45.0),
                    0.35,
                    8,
                    Color::new(7, 10, 20, 215),
                );
                d.draw_text_ex(
                    &ui_font,
                    message,
                    Vector2::new(banner_x, 113.0),
                    21.0,
                    0.6,
                    Color::new(193, 255, 245, 255),
                );
            }
        }

        match game_state {
            GameState::Title => {
                d.draw_rectangle_gradient_h(
                    0,
                    0,
                    (screen_width * 0.64) as i32,
                    screen_height as i32,
                    Color::new(5, 9, 20, 242),
                    Color::new(5, 9, 20, 12),
                );
                let pulse = 1.0 + (time * 2.2).sin() * 0.018;
                d.draw_text_ex(
                    &ui_font,
                    "EL SANTUARIO",
                    Vector2::new(69.0, 111.0),
                    55.0 * pulse,
                    1.2,
                    Color::new(55, 28, 12, 190),
                );
                d.draw_text_ex(
                    &ui_font,
                    "EL SANTUARIO",
                    Vector2::new(65.0, 106.0),
                    55.0 * pulse,
                    1.2,
                    Color::new(255, 222, 139, 255),
                );
                d.draw_text_ex(
                    &ui_font,
                    "DEL CRISTAL",
                    Vector2::new(65.0, 164.0),
                    68.0 * pulse,
                    1.0,
                    Color::new(122, 247, 226, 255),
                );
                d.draw_text_ex(
                    &ui_font,
                    "Una aventura en miniatura trazada con rayos",
                    Vector2::new(69.0, 247.0),
                    22.0,
                    0.4,
                    Color::new(225, 236, 235, 235),
                );

                let button_y = 330.0 + (time * 2.6).sin() * 3.0;
                d.draw_rectangle_rounded(
                    Rectangle::new(65.0, button_y, 360.0, 61.0),
                    0.32,
                    10,
                    Color::new(246, 213, 132, 238),
                );
                d.draw_text_ex(
                    &ui_font,
                    "ENTER   Comenzar aventura",
                    Vector2::new(90.0, button_y + 15.0),
                    24.0,
                    0.4,
                    Color::new(28, 25, 31, 255),
                );
                d.draw_text_ex(
                    &ui_font,
                    "C   Ver demostración cinematográfica",
                    Vector2::new(72.0, button_y + 83.0),
                    19.0,
                    0.3,
                    Color::new(180, 242, 234, 240),
                );
                d.draw_text_ex(
                    &ui_font,
                    "Juan José Rivas Álvarez   |   24856   |   Rust + raylib",
                    Vector2::new(68.0, screen_height - 54.0),
                    16.0,
                    0.2,
                    Color::new(226, 231, 233, 190),
                );
            }
            GameState::LevelIntro { started } => {
                let progress = ((now - started) as f32 / 3.1).clamp(0.0, 1.0);
                let entrance = ease_out_back((progress / 0.34).clamp(0.0, 1.0));
                let exit = 1.0 - smoothstep(((progress - 0.80) / 0.20).clamp(0.0, 1.0));
                let alpha = (255.0 * exit) as u8;
                let (name, subtitle, accent) = level_copy(level);
                let card_width = (screen_width * 0.62).min(760.0);
                let card_x = (screen_width - card_width) * 0.5
                    + (1.0 - entrance) * -(screen_width + card_width);
                let card_y = screen_height * 0.34;
                d.draw_rectangle_rounded(
                    Rectangle::new(card_x, card_y, card_width, 176.0),
                    0.20,
                    12,
                    Color::new(6, 11, 24, (230.0 * exit) as u8),
                );
                d.draw_rectangle_rounded_lines(
                    Rectangle::new(card_x, card_y, card_width, 176.0),
                    0.20,
                    12,
                    Color::new(accent.r, accent.g, accent.b, alpha),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    &format!("DIORAMA  {}", level.index() + 1),
                    screen_width * 0.5 + (1.0 - entrance) * -screen_width,
                    card_y + 23.0,
                    18.0,
                    1.6,
                    Color::new(accent.r, accent.g, accent.b, alpha),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    name,
                    screen_width * 0.5 + (1.0 - entrance) * -screen_width,
                    card_y + 54.0,
                    40.0,
                    0.8,
                    Color::new(255, 243, 214, alpha),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    subtitle,
                    screen_width * 0.5 + (1.0 - entrance) * -screen_width,
                    card_y + 116.0,
                    20.0,
                    0.4,
                    Color::new(220, 235, 236, alpha),
                );
            }
            GameState::CrystalSequence { started } => {
                let progress = ((now - started) as f32 / 3.4).clamp(0.0, 1.0);
                let reveal = smoothstep((progress / 0.32).clamp(0.0, 1.0));
                let flash = (1.0 - ((progress - 0.60).abs() / 0.18)).clamp(0.0, 1.0);
                if flash > 0.0 {
                    d.draw_rectangle(
                        0,
                        0,
                        screen_width as i32,
                        screen_height as i32,
                        Color::new(196, 255, 246, (flash * 155.0) as u8),
                    );
                }
                draw_centered(
                    &mut d,
                    &ui_font,
                    "CRISTAL DESPIERTO",
                    screen_width * 0.5,
                    screen_height * 0.18,
                    48.0 + reveal * 6.0,
                    1.0,
                    Color::new(235, 255, 248, (reveal * 255.0) as u8),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    "El santuario recupera su luz",
                    screen_width * 0.5,
                    screen_height * 0.18 + 66.0,
                    21.0,
                    0.4,
                    Color::new(142, 250, 230, (reveal * 240.0) as u8),
                );
            }
            GameState::GameOver { .. } => {
                d.draw_rectangle(
                    0,
                    0,
                    screen_width as i32,
                    screen_height as i32,
                    Color::new(10, 5, 16, 145),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    "TOD PERDIÓ EL RUMBO",
                    screen_width * 0.5,
                    screen_height * 0.40,
                    43.0,
                    0.8,
                    Color::new(255, 196, 178, 255),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    "La aventura continúa...",
                    screen_width * 0.5,
                    screen_height * 0.40 + 63.0,
                    22.0,
                    0.4,
                    Color::new(239, 227, 229, 240),
                );
            }
            GameState::CampaignClear => {
                let stars = campaign_stars(hits_taken, campaign_elapsed);
                d.draw_rectangle_gradient_v(
                    0,
                    0,
                    screen_width as i32,
                    screen_height as i32,
                    Color::new(5, 11, 24, 170),
                    Color::new(5, 11, 24, 238),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    "AVENTURA COMPLETADA",
                    screen_width * 0.5,
                    82.0,
                    53.0,
                    1.0,
                    Color::new(255, 226, 146, 255),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    "Los tres santuarios vuelven a brillar",
                    screen_width * 0.5,
                    151.0,
                    22.0,
                    0.4,
                    Color::new(171, 247, 235, 255),
                );
                for index in 0..3 {
                    let earned = index < stars;
                    let color = if earned {
                        Color::new(255, 218, 105, 255)
                    } else {
                        Color::new(104, 124, 139, 225)
                    };
                    let radius = if index == 1 { 43.0 } else { 35.0 };
                    draw_star(
                        &mut d,
                        Vector2::new(
                            screen_width * 0.5 + (index as f32 - 1.0) * 90.0,
                            238.0 - if index == 1 { 14.0 } else { 0.0 },
                        ),
                        radius,
                        radius * 0.46,
                        color,
                    );
                }
                let minutes = (campaign_elapsed / 60.0).floor() as u32;
                let seconds = (campaign_elapsed as u32) % 60;
                d.draw_rectangle_rounded(
                    Rectangle::new(screen_width * 0.5 - 245.0, 318.0, 490.0, 165.0),
                    0.16,
                    10,
                    Color::new(8, 15, 30, 225),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    "Cristales restaurados       3 / 3",
                    screen_width * 0.5,
                    342.0,
                    21.0,
                    0.4,
                    Color::new(232, 241, 240, 255),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    &format!("Tiempo de expedición       {minutes:02}:{seconds:02}"),
                    screen_width * 0.5,
                    384.0,
                    21.0,
                    0.4,
                    Color::new(232, 241, 240, 255),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    &format!("Vidas restantes                 {lives}"),
                    screen_width * 0.5,
                    426.0,
                    21.0,
                    0.4,
                    Color::new(232, 241, 240, 255),
                );
                draw_centered(
                    &mut d,
                    &ui_font,
                    "R   Volver a la portada",
                    screen_width * 0.5,
                    screen_height - 78.0,
                    21.0,
                    0.5,
                    Color::new(151, 241, 228, 255),
                );
            }
            GameState::Playing => {}
        }
    }
}

fn world_to_screen(camera: &Camera, point: Vec3, width: f32, height: f32) -> Option<Vector2> {
    let origin = camera.position();
    let forward = (camera.target - origin).normalized();
    let right = forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
    let up = right.cross(forward).normalized();
    let delta = point - origin;
    let depth = delta.dot(forward);
    if depth <= 0.05 {
        return None;
    }
    let vertical_scale = (camera.fov_degrees.to_radians() * 0.5).tan();
    let horizontal_scale = vertical_scale * (width / height.max(1.0));
    let normalized_x = delta.dot(right) / (depth * horizontal_scale);
    let normalized_y = delta.dot(up) / (depth * vertical_scale);
    Some(Vector2::new(
        (normalized_x * 0.5 + 0.5) * width,
        (0.5 - normalized_y * 0.5) * height,
    ))
}

fn campaign_stars(hits_taken: u32, elapsed: f64) -> usize {
    if hits_taken == 0 && elapsed <= 300.0 {
        3
    } else if hits_taken <= 3 {
        2
    } else {
        1
    }
}

fn build_level(level: LevelKind) -> (Scene, Character, Vec<Enemy>, Vec<Block>) {
    let mut scene = Scene::for_level(level);
    let player = Character::new(CHARACTER_SPAWN, 0.0, DEFAULT_CHARACTER_SCALE);
    let enemies = enemy::for_theme(level.index());
    let mut enemy_blocks = Vec::new();
    collect_enemy_blocks(&enemies, &mut enemy_blocks);
    scene.set_enemy_blocks(&enemy_blocks);
    (scene, player, enemies, enemy_blocks)
}

fn collect_enemy_blocks(enemies: &[Enemy], output: &mut Vec<Block>) {
    output.clear();
    for enemy in enemies {
        output.extend_from_slice(enemy.blocks());
    }
}

fn move_player(player: &mut Character, movement: Vec3, dt: f32, scene: &Scene) {
    let previous_position = player.position;
    player.update(dt, movement);
    if !player.is_moving {
        return;
    }

    let desired_position = player.position;
    let x_ground = scene.ground_height_at(desired_position.x, previous_position.z);
    player.position = if x_ground - previous_position.y > 0.50 {
        previous_position
    } else {
        Vec3::new(desired_position.x, x_ground, previous_position.z)
    };
    player.rebuild_geometry();
    let (x_min, x_max) = player.bounding_box();
    if scene.collides_with_world(x_min, x_max) {
        player.position.x = previous_position.x;
    }

    let z_ground = scene.ground_height_at(player.position.x, desired_position.z);
    if z_ground - player.position.y <= 0.50 {
        player.position.z = desired_position.z;
        player.position.y = z_ground;
    }
    player.rebuild_geometry();
    let (z_min, z_max) = player.bounding_box();
    if scene.collides_with_world(z_min, z_max) {
        player.position.z = previous_position.z;
        player.position.y = scene.ground_height_at(player.position.x, player.position.z);
    }
    player.rebuild_geometry();
}

fn render_preview() {
    let camera = Camera {
        target: Vec3::new(0.3, -0.1, 0.1),
        yaw: -0.72,
        pitch: 0.40,
        distance: 18.5,
        fov_degrees: 43.0,
    };
    std::fs::create_dir_all("screenshots").expect("No se pudo crear screenshots/");
    for (level, filename) in [
        (LevelKind::Hills, "screenshots/preview.bmp"),
        (LevelKind::Haunted, "screenshots/preview_haunted.bmp"),
        (LevelKind::Aquatic, "screenshots/preview_aquatic.bmp"),
    ] {
        let (scene, _, _, _) = build_level(level);
        let mut renderer = Renderer::new(960, 540);
        renderer.max_bounces = 3;
        renderer.samples_per_pixel = 3;
        renderer.render(&scene, &camera, level.index() as f32 * 0.7);
        renderer
            .save_bmp(filename)
            .expect("No se pudo guardar la captura");
        println!("Captura guardada en {filename}");
    }
}

fn render_benchmark() {
    use std::time::Instant;

    let (scene, _, _, _) = build_level(LevelKind::Hills);
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

#[cfg(test)]
mod campaign_tests {
    use super::*;

    #[test]
    fn campaign_results_reward_clean_runs() {
        assert_eq!(campaign_stars(0, 240.0), 3);
        assert_eq!(campaign_stars(2, 360.0), 2);
        assert_eq!(campaign_stars(5, 180.0), 1);
    }

    #[test]
    fn cinematic_route_can_complete_the_level() {
        for level in [LevelKind::Hills, LevelKind::Haunted, LevelKind::Aquatic] {
            let (mut scene, mut player, _, _) = build_level(level);
            let waypoints = demo_waypoints(level);
            let mut waypoint = 1_usize;
            let mut activated = false;

            for frame in 0..30_000 {
                let riding_elevator = scene.is_riding_elevator(player.position);
                scene.update_mechanisms(frame as f32 / 60.0);
                if riding_elevator {
                    player.position.y = scene.elevator_top();
                    player.rebuild_geometry();
                }
                let target = waypoints[waypoint];
                let delta = target - player.position;
                if delta.length() < 0.14 {
                    waypoint = (waypoint + 1).min(waypoints.len() - 1);
                }
                move_player(
                    &mut player,
                    Vec3::new(delta.x, 0.0, delta.z).normalized(),
                    1.0 / 60.0,
                    &scene,
                );
                scene.collect_near(player.position);
                activated |= scene.try_activate_crystal(player.position);
                if activated {
                    break;
                }
            }

            let status = scene.puzzle_status();
            assert!(
                activated,
                "{level:?} route stopped at waypoint {waypoint}, position {:?}, fragments {}/{}",
                player.position, status.collected, status.total
            );
            assert!(scene.puzzle_status().crystal_activated);
        }
    }

    #[test]
    fn every_level_builds_with_three_enemies() {
        for level in [LevelKind::Hills, LevelKind::Haunted, LevelKind::Aquatic] {
            let (scene, _, enemies, enemy_blocks) = build_level(level);
            assert_eq!(enemies.len(), 3);
            assert_eq!(enemy_blocks.len(), 21);
            assert_eq!(scene.level, level);
        }
    }
}
