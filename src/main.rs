mod audio;
mod creature;
mod lexicon;
mod web;

use audio::SoundEngine;
use creature::{Crawler, TOTAL_CATALOG_SPECIMENS};
use lexicon::LexiconEngine;
use macroquad::prelude::{
    clear_background, draw_circle, draw_rectangle, draw_rectangle_lines, draw_text,
    get_char_pressed, get_frame_time, is_key_down, is_key_pressed, is_mouse_button_down,
    is_mouse_button_pressed, mouse_position, mouse_wheel, next_frame, screen_height,
    screen_width, set_camera, set_default_camera, Camera2D, Color, Conf, KeyCode, MouseButton,
    Vec2,
};
use ::rand::Rng;
use web::{open_url_in_browser, KnowledgeWeb};

fn window_conf() -> Conf {
    Conf {
        window_title: "NOEMA // Autonomous Wikipedia Webpage".to_owned(),
        window_width: 1480,
        window_height: 940,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut lexicon = LexiconEngine::new();
    let mut web = KnowledgeWeb::new(&mut lexicon);
    let mut audio = SoundEngine::new();

    let mut current_specimen_id: usize = 7;
    let mut crawlers: Vec<Crawler> = vec![Crawler::from_catalog_id(
        current_specimen_id,
        Vec2::new(0.0, -120.0),
    )];

    let mut cam_target = Vec2::new(0.0, -120.0);
    let cam_zoom: f32 = 0.00065; // Wide-angle view so the whole Wikipedia page and infobox are visible
    let mut prev_mouse_world = Vec2::ZERO;
    let mut last_mouse_screen = Vec2::from(mouse_position());

    let mut wiki_input_active = false;
    let mut wiki_query = String::new();

    loop {
        let dt = get_frame_time().min(0.04);
        let sw = screen_width();
        let sh = screen_height();
        let aspect = sw / sh;

        if web.poll_bridge(&mut lexicon) {
            crawlers[0].release_all_tethers(&mut web);
            cam_target = Vec2::new(0.0, -100.0);
        }

        if wiki_input_active {
            while let Some(ch) = get_char_pressed() {
                if !ch.is_control() && wiki_query.len() < 48 {
                    wiki_query.push(ch);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                wiki_query.pop();
            }
            if is_key_pressed(KeyCode::Enter) {
                if !wiki_query.trim().is_empty() {
                    web.bridge.request_topic(&wiki_query);
                    wiki_query.clear();
                }
                wiki_input_active = false;
            }
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Tab) {
                wiki_input_active = false;
            }
        } else {
            let step_fwd = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);
            let step_bwd = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
            let jump = if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                17
            } else {
                1
            };

            if step_fwd {
                crawlers[0].release_all_tethers(&mut web);
                current_specimen_id =
                    ((current_specimen_id - 1 + jump) % TOTAL_CATALOG_SPECIMENS) + 1;
                let pos = crawlers[0].head_pos();
                crawlers[0] = Crawler::from_catalog_id(current_specimen_id, pos);
            }
            if step_bwd {
                crawlers[0].release_all_tethers(&mut web);
                current_specimen_id = ((current_specimen_id + TOTAL_CATALOG_SPECIMENS - 1
                    - (jump % TOTAL_CATALOG_SPECIMENS))
                    % TOTAL_CATALOG_SPECIMENS)
                    + 1;
                let pos = crawlers[0].head_pos();
                crawlers[0] = Crawler::from_catalog_id(current_specimen_id, pos);
            }

            if is_key_pressed(KeyCode::R) {
                let mut rng = ::rand::thread_rng();
                crawlers[0].release_all_tethers(&mut web);
                current_specimen_id = rng.gen_range(1..=TOTAL_CATALOG_SPECIMENS);
                let pos = crawlers[0].head_pos();
                crawlers[0] = Crawler::from_catalog_id(current_specimen_id, pos);
            }

            let preset_keys = [
                (KeyCode::Key1, 2),
                (KeyCode::Key2, 7),
                (KeyCode::Key3, 15),
                (KeyCode::Key4, 4),
                (KeyCode::Key5, 5),
                (KeyCode::Key6, 6),
            ];
            for &(k, target_id) in &preset_keys {
                if is_key_pressed(k) {
                    crawlers[0].release_all_tethers(&mut web);
                    current_specimen_id = target_id;
                    let pos = crawlers[0].head_pos();
                    crawlers[0] = Crawler::from_catalog_id(current_specimen_id, pos);
                }
            }

            if is_key_pressed(KeyCode::P) {
                crawlers[0].release_all_tethers(&mut web);
                web.cycle_preset_article(&mut lexicon);
            }
            if is_key_pressed(KeyCode::O) {
                open_url_in_browser(&web.current_title);
            }
            if is_key_pressed(KeyCode::M) {
                audio.muted = !audio.muted;
            }
            if is_key_pressed(KeyCode::Tab) {
                wiki_input_active = true;
                while get_char_pressed().is_some() {}
            }

            let scroll_speed = 700.0 * dt;
            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                cam_target.y -= scroll_speed;
            }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                cam_target.y += scroll_speed;
            }
        }

        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            cam_target.y -= wheel_y * 70.0;
        }
        cam_target.y = cam_target.y.clamp(-600.0, 1100.0);

        let mouse_screen = Vec2::from(mouse_position());
        if is_mouse_button_down(MouseButton::Right) || is_mouse_button_down(MouseButton::Middle) {
            let delta = mouse_screen - last_mouse_screen;
            cam_target.x -= delta.x / (cam_zoom * sw * 0.5);
            cam_target.y -= delta.y / (cam_zoom * sw * 0.5);
            cam_target.y = cam_target.y.clamp(-600.0, 1100.0);
        }
        last_mouse_screen = mouse_screen;

        let camera = Camera2D {
            target: cam_target,
            zoom: Vec2::new(cam_zoom, cam_zoom * aspect),
            ..Default::default()
        };

        let mouse_world = camera.screen_to_world(mouse_screen);
        let mouse_vel = (mouse_world - prev_mouse_world) / dt.max(0.001);
        prev_mouse_world = mouse_world;

        let camera_half_w = 1.0 / cam_zoom;

        if is_mouse_button_pressed(MouseButton::Left) && !wiki_input_active && mouse_screen.y > 60.0 {
            let ctrl_held =
                is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
            if !web.check_hyperlink_click(mouse_world, ctrl_held) {
                crawlers[0].target_pos = mouse_world;
            }
        }

        web.update_words(dt, mouse_world, mouse_vel);

        for crawler in &mut crawlers {
            crawler.update(
                dt,
                mouse_world,
                mouse_vel,
                &mut web,
                &mut lexicon,
                &audio,
                cam_target,
                camera_half_w,
            );
        }

        audio.update_drone(
            crawlers[0].root_freq,
            crawlers[0].fm_ratio,
            crawlers[0].mood.curiosity,
            crawlers[0].mood.vitality,
        );

        // =====================================================================
        // RENDER WORLD SPACE
        // =====================================================================
        clear_background(Color::new(0.022, 0.020, 0.032, 1.0));
        set_camera(&camera);

        web.draw(&lexicon, crawlers[0].primary_rgb);

        for crawler in &crawlers {
            crawler.draw(&web);
        }

        // =====================================================================
        // RENDER HUD
        // =====================================================================
        set_default_camera();
        let (pr, pg, pb) = crawlers[0].primary_rgb;
        let (lr, lg, lb) = crawlers[0].laser_rgb;

        let badge_x = 22.0;
        let badge_y = sh - 58.0;
        draw_rectangle(
            badge_x,
            badge_y,
            255.0,
            38.0,
            Color::new(0.03, 0.025, 0.05, 0.90),
        );
        draw_rectangle_lines(
            badge_x,
            badge_y,
            255.0,
            38.0,
            1.2,
            Color::new(pr, pg, pb, 0.75),
        );
        draw_text(
            &format!("#{:04}", crawlers[0].specimen_id),
            badge_x + 12.0,
            badge_y + 25.0,
            19.0,
            Color::new(pr, pg, pb, 0.98),
        );
        draw_text(
            &crawlers[0].glyph_tag,
            badge_x + 82.0,
            badge_y + 25.0,
            18.0,
            Color::new(0.92, 0.94, 1.0, 0.95),
        );
        draw_circle(
            badge_x + 234.0,
            badge_y + 19.0,
            6.0,
            Color::new(lr, lg, lb, 0.98),
        );

        draw_rectangle(0.0, 0.0, sw, 52.0, Color::new(0.02, 0.018, 0.035, 0.92));
        draw_text(
            &format!(
                "{} ({})  |  {}  |  {:.0} Hz  |  Wiki: [{}]",
                crawlers[0].name,
                crawlers[0].mood.dominant_mood_name(),
                crawlers[0].behavior.label(),
                crawlers[0].root_freq,
                web.current_title
            ),
            18.0,
            21.0,
            15.5,
            Color::new(pr * 0.35 + 0.65, pg * 0.35 + 0.65, pb * 0.35 + 0.65, 0.98),
        );
        draw_text(
            "[Autonomous Wanderer Mode]: Roams page | [Click]: Waypoint  |  [A/D]: Specimens  |  [Scroll Wheel]: Scroll Wiki  |  [O]: Open Browser  |  [TAB]: Search",
            18.0,
            42.0,
            13.0,
            Color::new(0.68, 0.72, 0.86, 0.88),
        );

        draw_text(
            &web.bridge.last_status,
            292.0,
            sh - 34.0,
            14.0,
            Color::new(0.65, 0.72, 0.88, 0.85),
        );

        if wiki_input_active {
            let mw = 540.0;
            let mh = 112.0;
            let mx = (sw - mw) * 0.5;
            let my = (sh - mh) * 0.5;
            draw_rectangle(mx, my, mw, mh, Color::new(0.04, 0.03, 0.08, 0.96));
            draw_rectangle_lines(mx, my, mw, mh, 1.6, Color::new(pr, pg, pb, 0.92));
            draw_text(
                "OPEN FULL WIKIPEDIA ARTICLE ON THE CANVAS (PRESS ENTER):",
                mx + 20.0,
                my + 34.0,
                15.5,
                Color::new(0.92, 0.94, 1.0, 0.95),
            );
            draw_rectangle(
                mx + 20.0,
                my + 50.0,
                mw - 40.0,
                36.0,
                Color::new(0.015, 0.01, 0.03, 1.0),
            );
            draw_rectangle_lines(
                mx + 20.0,
                my + 50.0,
                mw - 40.0,
                36.0,
                1.2,
                Color::new(pr, pg, pb, 0.75),
            );
            draw_text(
                &format!("{}_", wiki_query),
                mx + 30.0,
                my + 74.0,
                18.0,
                Color::new(pr, pg, pb, 1.0),
            );
        }

        next_frame().await;
    }
}

