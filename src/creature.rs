use crate::audio::{NodeSoundKind, SoundEngine};
use crate::lexicon::{LexiconEngine, SemanticValence};
use crate::web::{KnowledgeWeb, WordState, WordStyle};
use ::rand::rngs::StdRng;
use ::rand::{Rng, SeedableRng};
use macroquad::prelude::{draw_circle, draw_circle_lines, draw_line, draw_triangle, is_mouse_button_down, Color, MouseButton, Vec2};

pub const TOTAL_CATALOG_SPECIMENS: usize = 1_111;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorMode {
    LaserIncinerator,
    BerserkerBreaker,
    SilkWeaver,
    GravityVortex,
    ArcCorruptor,
    HarmonicArchitect,
}

impl BehaviorMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::LaserIncinerator => "LASER INCINERATOR [Vaporizer]",
            Self::BerserkerBreaker => "BERSERKER BREAKER [Paragraph Shatter]",
            Self::SilkWeaver => "SILK WEAVER [Tether & Pluck]",
            Self::GravityVortex => "GRAVITY SINGULARITY [Orbital Vortex]",
            Self::ArcCorruptor => "TESLA ARC CORRUPTOR [Syntax Mutator]",
            Self::HarmonicArchitect => "HARMONIC ARCHITECT [Cymatic Crystallizer]",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChamberStyle {
    HollowRect,
    DualCell,
    CrossCore,
    NestedDiamond,
    HexFrame,
    RibbedCompartment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadStyle {
    TwinOcular,
    CyclopsLaserLens,
    ArachnidMultiEye,
    MandiblePincers,
    CrescentCrest,
    TriProngCrown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DorsalStyle {
    None,
    WireframeWings,
    LateralFins,
    OrbitingSatellites,
    DorsalSpikes,
}

#[derive(Clone, Debug)]
pub struct SpineSegment {
    pub pos: Vec2,
    pub angle: f32,
    pub width: f32,
    pub length: f32,
    pub style: ChamberStyle,
    pub excite: f32,
    pub pitch_idx: usize,
}

#[derive(Clone, Debug)]
pub struct IkLeg {
    pub seg_idx: usize,
    pub side: f32,
    pub hip_pos: Vec2,
    pub knee_pos: Vec2,
    pub ankle_pos: Vec2,
    pub foot_pos: Vec2,
    pub step_start: Vec2,
    pub step_target: Vec2,
    pub step_prog: f32,
    pub is_stepping: bool,
    pub rest_reach: f32,
    pub rest_forward: f32,
    pub joint_box_shape: u8,
    pub excite_knee: f32,
    pub excite_ankle: f32,
    pub excite_foot: f32,
    pub touch_offset: Vec2,
    pub pitch_idx: usize,
    pub tethered_word_id: Option<usize>,
    pub tether_timer: f32,
}

#[derive(Clone, Debug)]
pub struct TentacleNode {
    pub pos: Vec2,
    pub vel: Vec2,
    pub box_size: f32,
    pub excite: f32,
    pub pitch_idx: usize,
}

#[derive(Clone, Debug)]
pub struct Tentacle {
    pub attach_seg: usize,
    pub side_offset: Vec2,
    pub base_angle_offset: f32,
    pub nodes: Vec<TentacleNode>,
}

#[derive(Clone, Debug)]
pub struct ActiveBeam {
    pub from: Vec2,
    pub to: Vec2,
    pub color: (f32, f32, f32),
    pub life: f32,
    pub max_life: f32,
    pub is_jagged_arc: bool,
}

pub struct Crawler {
    pub specimen_id: usize,
    pub name: String,
    pub glyph_tag: String,
    pub behavior: BehaviorMode,
    pub head_style: HeadStyle,
    pub dorsal_style: DorsalStyle,
    pub primary_rgb: (f32, f32, f32),
    pub secondary_rgb: (f32, f32, f32),
    pub laser_rgb: (f32, f32, f32),
    pub mood: SemanticValence,
    pub root_freq: f32,
    pub scale_id: usize,
    pub fm_ratio: f32,
    pub fm_brightness: f32,
    pub speed_mul: f32,
    pub spine: Vec<SpineSegment>,
    pub legs: Vec<IkLeg>,
    pub tentacles: Vec<Tentacle>,
    pub active_beams: Vec<ActiveBeam>,
    pub vel: Vec2,
    pub target_pos: Vec2,
    pub phase: f32,
    pub action_cooldown: f32,
    wander_timer: f32,
}

const PREFIXES: [&str; 16] = [
    "Xeno", "Chitin", "Pyro", "Vortex", "Cryo", "Tesla", "Aether", "Nyx",
    "Lumen", "Kronos", "Sclero", "Mycel", "Stridul", "Phasmo", "Vex", "Orphic",
];

const SUFFIXES: [&str; 16] = [
    "weaver", "clast", "mantis", "leviathan", "scythe", "wyrm", "arachne", "cephalon",
    "seraph", "isopod", "stalker", "locust", "hydra", "scolopendra", "drifter", "architect",
];

const GLYPHS: [&str; 12] = [
    "~*ALAWN*~", "~*KAIROS*~", "~*NOEMA*~", "~*SYLPH*~", "~*VEXIL*~", "~*ORPHIC*~",
    "~*XENON*~", "~*AETHER*~", "~*CHITIN*~", "~*LYRA*~", "~*THALAS*~", "~*MNEME*~",
];

impl Crawler {
    pub fn from_catalog_id(specimen_id: usize, spawn_pos: Vec2) -> Self {
        let id = specimen_id.clamp(1, TOTAL_CATALOG_SPECIMENS);
        let mut rng = StdRng::seed_from_u64((id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));

        let behavior = match id {
            2 => BehaviorMode::SilkWeaver,
            7 => BehaviorMode::ArcCorruptor,
            15 => BehaviorMode::GravityVortex,
            4 => BehaviorMode::LaserIncinerator,
            5 => BehaviorMode::BerserkerBreaker,
            6 => BehaviorMode::HarmonicArchitect,
            _ => match rng.gen_range(0..6) {
                0 => BehaviorMode::LaserIncinerator,
                1 => BehaviorMode::BerserkerBreaker,
                2 => BehaviorMode::SilkWeaver,
                3 => BehaviorMode::GravityVortex,
                4 => BehaviorMode::ArcCorruptor,
                _ => BehaviorMode::HarmonicArchitect,
            },
        };

        let (primary_rgb, secondary_rgb, laser_rgb) = match id {
            2 => ((0.42, 0.96, 0.24), (0.18, 0.85, 0.92), (0.98, 0.25, 0.45)),
            7 => ((0.95, 0.92, 0.22), (0.32, 0.48, 0.98), (0.45, 0.95, 1.0)),
            15 => ((0.22, 0.58, 0.98), (0.18, 0.92, 0.96), (0.85, 0.35, 1.0)),
            _ => {
                let hue_a = rng.gen_range(0.0_f32..360.0_f32);
                let hue_offset = match rng.gen_range(0..4) {
                    0 => 180.0,
                    1 => 120.0,
                    2 => 55.0,
                    _ => 210.0,
                };
                let hue_b = (hue_a + hue_offset) % 360.0;
                let hue_c = (hue_a + 300.0) % 360.0;
                (
                    hsv_to_rgb(hue_a, rng.gen_range(0.75..1.0), 0.98),
                    hsv_to_rgb(hue_b, rng.gen_range(0.70..0.98), 0.95),
                    hsv_to_rgb(hue_c, 0.95, 1.0),
                )
            }
        };

        let root_freq: f32 = rng.gen_range(58.0_f32..380.0_f32);
        let scale_id: usize = rng.gen_range(0..8);
        let fm_ratios = [0.5_f32, 1.0, 1.414, 1.5, 2.0, 2.718, 3.0, 4.0];
        let fm_ratio = fm_ratios[rng.gen_range(0..fm_ratios.len())];
        let fm_brightness: f32 = rng.gen_range(0.45_f32..2.2_f32);

        let speed_mul = match behavior {
            BehaviorMode::BerserkerBreaker => rng.gen_range(1.45_f32..1.85_f32),
            BehaviorMode::LaserIncinerator => rng.gen_range(0.95_f32..1.35_f32),
            _ => rng.gen_range(0.85_f32..1.30_f32),
        };

        let head_style = match rng.gen_range(0..6) {
            0 => HeadStyle::TwinOcular,
            1 => HeadStyle::CyclopsLaserLens,
            2 => HeadStyle::ArachnidMultiEye,
            3 => HeadStyle::MandiblePincers,
            4 => HeadStyle::CrescentCrest,
            _ => HeadStyle::TriProngCrown,
        };

        let dorsal_style = match rng.gen_range(0..5) {
            0 => DorsalStyle::None,
            1 => DorsalStyle::WireframeWings,
            2 => DorsalStyle::LateralFins,
            3 => DorsalStyle::OrbitingSatellites,
            _ => DorsalStyle::DorsalSpikes,
        };

        let seg_count: usize = match id {
            2 | 15 => 2,
            7 => 12,
            _ => rng.gen_range(1..=15),
        };

        let taper_mode: u8 = rng.gen_range(0..6);
        let base_width: f32 = if seg_count <= 3 {
            rng.gen_range(48.0_f32..72.0_f32)
        } else {
            rng.gen_range(22.0_f32..40.0_f32)
        };
        let base_len: f32 = if seg_count <= 3 {
            rng.gen_range(42.0_f32..68.0_f32)
        } else {
            rng.gen_range(20.0_f32..30.0_f32)
        };

        let primary_chamber = match rng.gen_range(0..6) {
            0 => ChamberStyle::HollowRect,
            1 => ChamberStyle::DualCell,
            2 => ChamberStyle::CrossCore,
            3 => ChamberStyle::NestedDiamond,
            4 => ChamberStyle::HexFrame,
            _ => ChamberStyle::RibbedCompartment,
        };
        let secondary_chamber = match rng.gen_range(0..6) {
            0 => ChamberStyle::DualCell,
            1 => ChamberStyle::NestedDiamond,
            2 => ChamberStyle::CrossCore,
            3 => ChamberStyle::HexFrame,
            4 => ChamberStyle::RibbedCompartment,
            _ => ChamberStyle::HollowRect,
        };

        let mut spine = Vec::with_capacity(seg_count);
        let mut accum_x = 0.0_f32;

        for s in 0..seg_count {
            let frac = s as f32 / (seg_count.max(1) as f32);
            let scale_w = match taper_mode {
                0 => 1.0 - frac * 0.55,
                1 => (frac * std::f32::consts::PI).sin() * 0.6 + 0.55,
                2 => if s % 2 == 0 { 1.15 } else { 0.72 },
                3 => 1.0 - ((frac - 0.5).abs() * 1.1) + 0.35,
                4 => if s == 0 { 1.35 } else { 0.85 - frac * 0.3 },
                _ => 0.95,
            };

            let w = (base_width * scale_w).max(14.0);
            let l = (base_len * scale_w.max(0.75)).max(16.0);
            let seg_pos = spawn_pos + Vec2::new(accum_x, 0.0);
            accum_x += l * 0.88;

            spine.push(SpineSegment {
                pos: seg_pos,
                angle: std::f32::consts::PI,
                width: w,
                length: l,
                style: if s % 2 == 0 {
                    primary_chamber
                } else {
                    secondary_chamber
                },
                excite: 0.0,
                pitch_idx: s % 16,
            });
        }

        let mut legs = Vec::new();
        let joint_box_shape: u8 = rng.gen_range(0..4);
        let leg_stride_step = if seg_count > 8 {
            rng.gen_range(1..=2)
        } else {
            1
        };

        for s in (0..seg_count).step_by(leg_stride_step) {
            let pairs_on_seg = if seg_count == 1 {
                rng.gen_range(2..=4)
            } else if seg_count <= 3 {
                rng.gen_range(1..=2)
            } else {
                1
            };

            for p_i in 0..pairs_on_seg {
                for &side in &[-1.0_f32, 1.0_f32] {
                    let reach: f32 = rng.gen_range(72.0_f32..150.0_f32);
                    let forward: f32 =
                        (0.45 - (s as f32 / seg_count.max(1) as f32)) * 52.0 + (p_i as f32 * 24.0 - 12.0);
                    let seg_pos = spine[s].pos;
                    let foot = seg_pos + Vec2::new(forward, side * reach);

                    legs.push(IkLeg {
                        seg_idx: s,
                        side,
                        hip_pos: seg_pos,
                        knee_pos: seg_pos + Vec2::new(forward * 0.4, side * reach * 0.45),
                        ankle_pos: seg_pos + Vec2::new(forward * 0.75, side * reach * 0.78),
                        foot_pos: foot,
                        step_start: foot,
                        step_target: foot,
                        step_prog: 1.0,
                        is_stepping: false,
                        rest_reach: reach,
                        rest_forward: forward,
                        joint_box_shape,
                        excite_knee: 0.0,
                        excite_ankle: 0.0,
                        excite_foot: 0.0,
                        touch_offset: Vec2::ZERO,
                        pitch_idx: (s * 2 + p_i + if side > 0.0 { 1 } else { 0 }) % 16,
                        tethered_word_id: None,
                        tether_timer: 0.0,
                    });
                }
            }
        }

        let mut tentacles = Vec::new();
        let tentacle_cluster = rng.gen_range(0..4);
        match tentacle_cluster {
            0 => {
                let count = rng.gen_range(4..=8);
                for t_i in 0..count {
                    let side = if t_i % 2 == 0 { -1.0 } else { 1.0 };
                    let ang = side * (0.65 + (t_i as f32 * 0.28));
                    tentacles.push(Self::build_tentacle(
                        0,
                        Vec2::new(0.0, side * 16.0),
                        ang,
                        rng.gen_range(9..=15),
                        12.5,
                        rng.gen_range(15.0..26.0),
                    ));
                }
            }
            1 => {
                for &side in &[-1.0_f32, 1.0_f32] {
                    tentacles.push(Self::build_tentacle(
                        0,
                        Vec2::new(12.0, side * 9.0),
                        side * 0.42,
                        rng.gen_range(5..=9),
                        13.5,
                        14.0,
                    ));
                    tentacles.push(Self::build_tentacle(
                        seg_count - 1,
                        Vec2::new(-10.0, side * 7.0),
                        std::f32::consts::PI + side * 0.32,
                        rng.gen_range(6..=11),
                        13.0,
                        15.0,
                    ));
                }
            }
            2 => {
                tentacles.push(Self::build_tentacle(
                    seg_count - 1,
                    Vec2::new(-14.0, 0.0),
                    std::f32::consts::PI,
                    rng.gen_range(8..=14),
                    14.5,
                    22.0,
                ));
            }
            _ => {
                let mid = seg_count / 2;
                for &side in &[-1.0_f32, 1.0_f32] {
                    tentacles.push(Self::build_tentacle(
                        mid,
                        Vec2::new(0.0, side * 14.0),
                        side * 1.35,
                        rng.gen_range(7..=12),
                        12.0,
                        18.0,
                    ));
                }
            }
        }

        let species_title = format!(
            "{}{}",
            PREFIXES[id % PREFIXES.len()],
            SUFFIXES[(id / 7) % SUFFIXES.len()]
        );
        let name = format!("Specimen #{:04} // {}", id, species_title);
        let glyph_tag = GLYPHS[id % GLYPHS.len()].to_string();

        Self {
            specimen_id: id,
            name,
            glyph_tag,
            behavior,
            head_style,
            dorsal_style,
            primary_rgb,
            secondary_rgb,
            laser_rgb,
            mood: SemanticValence::new(0.82, 0.92),
            root_freq,
            scale_id,
            fm_ratio,
            fm_brightness,
            speed_mul,
            spine,
            legs,
            tentacles,
            active_beams: Vec::new(),
            vel: Vec2::ZERO,
            target_pos: spawn_pos + Vec2::new(rng.gen_range(-160.0..160.0), rng.gen_range(-90.0..90.0)),
            phase: 0.0,
            action_cooldown: 0.0,
            wander_timer: 0.0,
        }
    }

    fn build_tentacle(
        attach_seg: usize,
        side_offset: Vec2,
        base_angle_offset: f32,
        count: usize,
        spacing: f32,
        start_box_size: f32,
    ) -> Tentacle {
        let mut nodes = Vec::with_capacity(count);
        for i in 0..count {
            let frac = i as f32 / (count as f32);
            let box_size = start_box_size * (1.0 - frac * 0.42);
            nodes.push(TentacleNode {
                pos: Vec2::new(i as f32 * spacing, 0.0),
                vel: Vec2::ZERO,
                box_size,
                excite: 0.0,
                pitch_idx: i % 16,
            });
        }
        Tentacle {
            attach_seg,
            side_offset,
            base_angle_offset,
            nodes,
        }
    }

    pub fn head_pos(&self) -> Vec2 {
        self.spine.first().map(|s| s.pos).unwrap_or(Vec2::ZERO)
    }

    pub fn release_all_tethers(&mut self, web: &mut KnowledgeWeb) {
        for leg in &mut self.legs {
            if let Some(wid) = leg.tethered_word_id.take() {
                if wid < web.words.len() {
                    web.words[wid].state = WordState::Plucked {
                        timer: 0.0,
                        max_life: 5.5,
                        filled_highlight: false,
                    };
                }
            }
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        mouse_world: Vec2,
        mouse_vel: Vec2,
        web: &mut KnowledgeWeb,
        lexicon: &mut LexiconEngine,
        audio: &SoundEngine,
        camera_center: Vec2,
        camera_half_width: f32,
    ) {
        let dt_c = dt.min(0.033);
        self.phase += dt_c * 3.5 * self.speed_mul;
        self.action_cooldown = (self.action_cooldown - dt_c).max(0.0);
        self.wander_timer -= dt_c;

        for b in &mut self.active_beams {
            b.life += dt_c;
        }
        self.active_beams.retain(|b| b.life < b.max_life);

        if self.wander_timer <= 0.0 || self.head_pos().distance(self.target_pos) < 60.0 {
            let mut rng = ::rand::thread_rng();
            self.wander_timer = rng.gen_range(5.0..12.0);
            let off_x = rng.gen_range(-420.0..420.0);
            let off_y = rng.gen_range(-280.0..280.0);
            self.target_pos = self.head_pos() + Vec2::new(off_x, off_y);
        }

        if mouse_world.length_squared() > 1.0 && is_mouse_button_down(MouseButton::Left) {
            self.target_pos = mouse_world;
        }

        let head_pos = self.spine[0].pos;
        let to_target = self.target_pos - head_pos;
        let dist = to_target.length();

        if dist > 16.0 {
            let dir = to_target / dist;
            let perp = Vec2::new(-dir.y, dir.x);
            let slither = (self.phase * 1.35).sin() * (dist.min(140.0) * 0.24);
            let max_spd = 225.0 * self.speed_mul;
            let desired_vel =
                dir * (dist * 2.0 * self.speed_mul).clamp(32.0, max_spd) + perp * slither;
            self.vel = self.vel.lerp(desired_vel, (dt_c * 6.0).min(1.0));
        } else {
            let idle = Vec2::new((self.phase * 0.7).cos(), (self.phase * 0.9).sin()) * 14.0;
            self.vel = self.vel.lerp(idle, (dt_c * 4.0).min(1.0));
        }

        self.spine[0].pos += self.vel * dt_c;
        if self.vel.length_squared() > 4.0 {
            let target_ang = self.vel.y.atan2(self.vel.x);
            self.spine[0].angle =
                lerp_angle(self.spine[0].angle, target_ang, (dt_c * 10.0).min(1.0));
        }
        self.spine[0].excite = (self.spine[0].excite - dt_c * 2.4).max(0.0);

        for i in 1..self.spine.len() {
            let prev_pos = self.spine[i - 1].pos;
            let spacing = (self.spine[i - 1].length + self.spine[i].length) * 0.54;
            let delta = self.spine[i].pos - prev_pos;
            let d = delta.length().max(0.001);
            let dir = delta / d;
            let lateral = Vec2::new(-dir.y, dir.x) * ((self.phase - i as f32 * 0.55).sin() * 1.8);
            self.spine[i].pos = prev_pos + dir * spacing + lateral * dt_c * 12.0;
            self.spine[i].angle = (-dir.y).atan2(-dir.x);
            self.spine[i].excite = (self.spine[i].excite - dt_c * 2.4).max(0.0);
        }

        let mut active_steps_count = self.legs.iter().filter(|l| l.is_stepping).count();

        for l_idx in 0..self.legs.len() {
            let seg = &self.spine[self.legs[l_idx].seg_idx];
            let seg_pos = seg.pos;
            let seg_ang = seg.angle;
            let forward_dir = Vec2::new(seg_ang.cos(), seg_ang.sin());
            let right_dir = Vec2::new(-forward_dir.y, forward_dir.x);

            let leg = &mut self.legs[l_idx];
            leg.hip_pos = seg_pos + right_dir * (leg.side * seg.width * 0.48);
            leg.touch_offset *= (1.0 - dt_c * 8.0).max(0.0);
            leg.excite_knee = (leg.excite_knee - dt_c * 2.5).max(0.0);
            leg.excite_ankle = (leg.excite_ankle - dt_c * 2.5).max(0.0);
            leg.excite_foot = (leg.excite_foot - dt_c * 2.5).max(0.0);

            let ideal_foot = leg.hip_pos
                + right_dir * (leg.side * leg.rest_reach)
                + forward_dir * leg.rest_forward
                + self.vel * 0.15;

            if !leg.is_stepping {
                let err = leg.foot_pos.distance(ideal_foot);
                if err > leg.rest_reach * 0.50 && active_steps_count < 5 {
                    leg.is_stepping = true;
                    leg.step_prog = 0.0;
                    leg.step_start = leg.foot_pos;
                    leg.step_target = ideal_foot + self.vel * 0.12;
                    active_steps_count += 1;
                }
            } else {
                leg.step_prog += dt_c * 7.2 * self.speed_mul;
                if leg.step_prog >= 1.0 {
                    leg.step_prog = 1.0;
                    leg.foot_pos = leg.step_target;
                    leg.is_stepping = false;
                    leg.excite_foot = 0.7;

                    let pan = ((leg.foot_pos.x - camera_center.x) / camera_half_width.max(200.0))
                        .clamp(-0.9, 0.9);
                    audio.play_specimen_sound(
                        NodeSoundKind::Footstep,
                        self.root_freq,
                        self.scale_id,
                        self.fm_ratio,
                        self.fm_brightness,
                        leg.pitch_idx,
                        pan,
                        0.5,
                    );

                    if self.behavior == BehaviorMode::HarmonicArchitect && l_idx % 2 == 0 {
                        web.spawn_shockwave(leg.foot_pos, 46.0, self.primary_rgb);
                    }
                } else {
                    let t = leg.step_prog;
                    let smooth = t * t * (3.0 - 2.0 * t);
                    let lift = (t * std::f32::consts::PI).sin() * 15.0;
                    leg.foot_pos = leg.step_start.lerp(leg.step_target, smooth)
                        + right_dir * (leg.side * lift);
                }
            }

            let span = (leg.foot_pos + leg.touch_offset) - leg.hip_pos;
            let span_len = span.length().max(1.0);
            let span_dir = span / span_len;
            let bend_perp = Vec2::new(-span_dir.y, span_dir.x) * leg.side;
            let arch = ((leg.rest_reach * 1.15 - span_len).max(12.0)) * 0.42;

            leg.knee_pos = leg.hip_pos + span * 0.38 + bend_perp * arch + leg.touch_offset * 0.6;
            leg.ankle_pos =
                leg.hip_pos + span * 0.74 + bend_perp * (arch * 0.45) + leg.touch_offset * 0.3;

            if let Some(wid) = leg.tethered_word_id {
                leg.tether_timer += dt_c;
                if wid < web.words.len() {
                    let target_hold = leg.foot_pos + right_dir * (leg.side * 24.0);
                    web.words[wid].pos =
                        web.words[wid].pos.lerp(target_hold, (dt_c * 9.0).min(1.0));
                    web.words[wid].angle = (self.phase * 1.5 + l_idx as f32).sin() * 0.38;

                    if leg.tether_timer > 1.75 {
                        let mut rng = ::rand::thread_rng();
                        let fling_dir = right_dir * leg.side
                            + forward_dir * rng.gen_range(-0.6_f32..0.6_f32);
                        web.words[wid].vel =
                            fling_dir.normalize_or_zero() * rng.gen_range(70.0_f32..150.0_f32);
                        web.words[wid].ang_vel = rng.gen_range(-2.2_f32..2.2_f32);
                        web.words[wid].state = WordState::Plucked {
                            timer: 0.0,
                            max_life: rng.gen_range(5.5_f32..9.0_f32),
                            filled_highlight: rng.gen_bool(0.28),
                        };
                        let absorbed = web.words[wid].clean_word.clone();
                        lexicon.absorb_plucked_word(&mut rng, &absorbed);
                        leg.tethered_word_id = None;
                    }
                } else {
                    leg.tethered_word_id = None;
                }
            }
        }

        for t_idx in 0..self.tentacles.len() {
            let attach_seg = &self.spine[self.tentacles[t_idx].attach_seg];
            let seg_pos = attach_seg.pos;
            let seg_ang = attach_seg.angle;
            let forward = Vec2::new(seg_ang.cos(), seg_ang.sin());
            let right = Vec2::new(-forward.y, forward.x);

            let tent = &mut self.tentacles[t_idx];
            let root_pos = seg_pos + forward * tent.side_offset.x + right * tent.side_offset.y;
            let world_base_ang = seg_ang + tent.base_angle_offset;
            let base_dir = Vec2::new(world_base_ang.cos(), world_base_ang.sin());

            if let Some(first) = tent.nodes.first_mut() {
                first.pos = root_pos;
                first.excite = (first.excite - dt_c * 2.4).max(0.0);
            }

            for n_i in 1..tent.nodes.len() {
                let prev = tent.nodes[n_i - 1].pos;
                let frac = n_i as f32 / (tent.nodes.len() as f32);
                let wave =
                    ((self.phase * 1.4 - n_i as f32 * 0.48 + t_idx as f32).sin()) * 30.0 * frac;
                let wave_perp = Vec2::new(-base_dir.y, base_dir.x);

                let desired = prev + base_dir * 13.5 + wave_perp * (wave * dt_c);
                let cur = &mut tent.nodes[n_i];
                cur.pos += cur.vel * dt_c;
                cur.vel *= (1.0 - dt_c * 6.0).max(0.0);

                let d = (cur.pos - desired).length().max(0.001);
                let target_spacing = 12.5;
                cur.pos = prev
                    + (cur.pos - prev).normalize_or_zero() * target_spacing
                    + (desired - cur.pos) * (dt_c * 5.2).min(1.0);
                if d > 85.0 {
                    cur.pos = prev + base_dir * target_spacing;
                }
                cur.excite = (cur.excite - dt_c * 2.4).max(0.0);
            }
        }

        let mut rng = ::rand::thread_rng();
        let h_pos = self.head_pos();
        let speed = self.vel.length();
        let pan_head =
            ((h_pos.x - camera_center.x) / camera_half_width.max(200.0)).clamp(-0.9, 0.9);

        for seg in &self.spine {
            let shadow_rad = (seg.width * 2.4).max(65.0);
            for w in &mut web.words {
                if w.state == WordState::InPage {
                    let d = w.pos.distance(seg.pos);
                    if d < shadow_rad {
                        let carve = (1.0 - d / shadow_rad).powf(1.2);
                        if carve > w.shadow_carve {
                            w.shadow_carve = carve;
                        }
                    }
                }
            }
        }

        match self.behavior {
            BehaviorMode::LaserIncinerator => {
                if self.action_cooldown <= 0.0 {
                    let mut target_idx = None;
                    let mut best_d = 220.0_f32;
                    for (i, w) in web.words.iter().enumerate() {
                        if w.state == WordState::InPage && w.style != WordStyle::SectionHeader {
                            let d = w.pos.distance(h_pos);
                            if d > 35.0 && d < best_d && rng.gen_bool(0.38) {
                                best_d = d;
                                target_idx = Some(i);
                            }
                        }
                    }

                    if let Some(idx) = target_idx {
                        let w_pos = web.words[idx].pos;
                        web.words[idx].heat += 0.7;
                        self.spine[0].excite = 1.0;
                        self.active_beams.push(ActiveBeam {
                            from: h_pos,
                            to: w_pos,
                            color: self.laser_rgb,
                            life: 0.0,
                            max_life: 0.26,
                            is_jagged_arc: false,
                        });
                        web.spawn_burst(w_pos, self.laser_rgb, 16, 190.0);

                        if web.words[idx].heat >= 1.0 {
                            let word_str = web.words[idx].clean_word.clone();
                            lexicon.absorb_plucked_word(&mut rng, &word_str);
                            web.words[idx].state = WordState::Vaporized {
                                timer: 0.0,
                                respawn_time: rng.gen_range(8.0_f32..14.0_f32),
                            };
                            web.spawn_shockwave(w_pos, 50.0, self.laser_rgb);
                        }

                        audio.play_specimen_sound(
                            NodeSoundKind::LaserBeam,
                            self.root_freq,
                            self.scale_id,
                            self.fm_ratio,
                            self.fm_brightness,
                            idx % 16,
                            pan_head,
                            0.95,
                        );
                        self.action_cooldown = rng.gen_range(0.18_f32..0.42_f32);
                    }
                }
            }

            BehaviorMode::BerserkerBreaker => {
                if speed > 45.0 && self.action_cooldown <= 0.0 {
                    let mut shattered_any = false;
                    let mut bursts_to_spawn = Vec::new();
                    for w in &mut web.words {
                        if w.state == WordState::InPage && w.pos.distance(h_pos) < 65.0 {
                            let push_dir = (w.pos - h_pos).normalize_or_zero();
                            w.vel = push_dir * rng.gen_range(180.0_f32..360.0_f32) + self.vel * 0.5;
                            w.ang_vel = rng.gen_range(-5.5_f32..5.5_f32);
                            w.chip_rgb = if rng.gen_bool(0.5) {
                                self.primary_rgb
                            } else {
                                self.laser_rgb
                            };
                            w.state = WordState::Plucked {
                                timer: 0.0,
                                max_life: rng.gen_range(4.5_f32..7.5_f32),
                                filled_highlight: rng.gen_bool(0.35),
                            };
                            lexicon.absorb_plucked_word(&mut rng, &w.clean_word);
                            bursts_to_spawn.push(w.pos);
                            shattered_any = true;
                        }
                    }

                    for b_pos in bursts_to_spawn {
                        web.spawn_burst(b_pos, self.primary_rgb, 8, 140.0);
                    }

                    if shattered_any {
                        web.spawn_shockwave(h_pos, 90.0, self.primary_rgb);
                        audio.play_specimen_sound(
                            NodeSoundKind::WordShatter,
                            self.root_freq,
                            self.scale_id,
                            self.fm_ratio,
                            self.fm_brightness,
                            rng.gen_range(0..8),
                            pan_head,
                            1.15,
                        );
                        self.action_cooldown = 0.22;
                    }
                }
            }

            BehaviorMode::GravityVortex => {
                let center = self.spine[self.spine.len() / 2].pos;
                for (i, w) in web.words.iter_mut().enumerate() {
                    let d = w.pos.distance(center);
                    if w.state == WordState::InPage
                        && d < 115.0
                        && self.action_cooldown <= 0.0
                        && w.style != WordStyle::SectionHeader
                        && rng.gen_bool(0.28)
                    {
                        w.chip_rgb = if i % 2 == 0 {
                            self.primary_rgb
                        } else {
                            self.secondary_rgb
                        };
                        w.state = WordState::Plucked {
                            timer: 0.0,
                            max_life: 7.5,
                            filled_highlight: i % 5 == 0,
                        };
                        lexicon.absorb_plucked_word(&mut rng, &w.clean_word);
                        self.action_cooldown = 0.12;
                        audio.play_specimen_sound(
                            NodeSoundKind::TentacleBead,
                            self.root_freq,
                            self.scale_id,
                            self.fm_ratio,
                            self.fm_brightness,
                            i % 16,
                            pan_head,
                            0.75,
                        );
                    }

                    if let WordState::Plucked { .. } = w.state {
                        if d < 195.0 && d > 12.0 {
                            let to_c = (center - w.pos) / d;
                            let tangent = Vec2::new(-to_c.y, to_c.x);
                            let orbit_force = tangent * 250.0 + to_c * ((d - 90.0) * 3.5);
                            w.vel = w.vel.lerp(orbit_force, (dt_c * 4.5).min(1.0));
                            w.angle = tangent.y.atan2(tangent.x);
                        }
                    }
                }
            }

            BehaviorMode::ArcCorruptor => {
                if self.action_cooldown <= 0.0 {
                    let word_count = web.words.len();
                    for w_i in 0..word_count {
                        if web.words[w_i].state == WordState::InPage
                            && web.words[w_i].style == WordStyle::BodyText
                            && web.words[w_i].pos.distance(h_pos) < 145.0
                            && rng.gen_bool(0.25)
                        {
                            let w_pos = web.words[w_i].pos;
                            let prev_w = if w_i >= 1 {
                                Some(web.words[w_i - 1].text.as_str())
                            } else {
                                None
                            };
                            let pp_w = if w_i >= 2 {
                                Some(web.words[w_i - 2].text.as_str())
                            } else {
                                None
                            };
                            let curr_w = web.words[w_i].text.clone();
                            let mutated =
                                lexicon.mutate_word_in_context(&mut rng, pp_w, prev_w, &curr_w);
                            web.words[w_i].text = mutated;
                            web.words[w_i].mutate_flash = 1.0;

                            self.active_beams.push(ActiveBeam {
                                from: h_pos,
                                to: w_pos,
                                color: self.secondary_rgb,
                                life: 0.0,
                                max_life: 0.2,
                                is_jagged_arc: true,
                            });
                            web.spawn_burst(w_pos, self.secondary_rgb, 10, 130.0);

                            audio.play_specimen_sound(
                                NodeSoundKind::ArcZap,
                                self.root_freq,
                                self.scale_id,
                                self.fm_ratio,
                                self.fm_brightness,
                                w_i % 16,
                                pan_head,
                                0.85,
                            );
                            self.action_cooldown = 0.16;
                            break;
                        }
                    }
                }
            }

            BehaviorMode::SilkWeaver | BehaviorMode::HarmonicArchitect => {
                let active_tethers = self
                    .legs
                    .iter()
                    .filter(|l| l.tethered_word_id.is_some())
                    .count();

                for (l_idx, leg) in self.legs.iter_mut().enumerate() {
                    if leg.tethered_word_id.is_some() || active_tethers >= 5 || self.action_cooldown > 0.0
                    {
                        continue;
                    }
                    for w_i in 0..web.words.len() {
                        if web.words[w_i].state == WordState::InPage
                            && web.words[w_i].style != WordStyle::SectionHeader
                            && web.words[w_i].pos.distance(leg.foot_pos) < 35.0
                            && rng.gen_bool(0.35)
                        {
                            let palette = [self.primary_rgb, self.secondary_rgb, self.laser_rgb];
                            web.words[w_i].chip_rgb = palette[(w_i + l_idx) % palette.len()];

                            if self.behavior == BehaviorMode::HarmonicArchitect {
                                web.words[w_i].vel = Vec2::new(
                                    rng.gen_range(-35.0_f32..35.0_f32),
                                    rng.gen_range(-45.0_f32..-15.0_f32),
                                );
                                web.words[w_i].state = WordState::Plucked {
                                    timer: 0.0,
                                    max_life: 6.5,
                                    filled_highlight: true,
                                };
                            } else {
                                web.words[w_i].state = WordState::Tethered {
                                    crawler_id: self.specimen_id,
                                    limb_idx: l_idx,
                                };
                                leg.tethered_word_id = Some(w_i);
                                leg.tether_timer = 0.0;
                            }

                            leg.excite_foot = 1.0;
                            self.action_cooldown = 0.18;
                            audio.play_specimen_sound(
                                NodeSoundKind::TentacleBead,
                                self.root_freq,
                                self.scale_id,
                                self.fm_ratio,
                                self.fm_brightness,
                                w_i % 16,
                                pan_head,
                                0.85,
                            );
                            break;
                        }
                    }
                }
            }
        }

        let mouse_speed = mouse_vel.length();
        let brush_intensity = (0.35 + mouse_speed * 0.0025).clamp(0.35, 1.45);
        let pan_mouse =
            ((mouse_world.x - camera_center.x) / camera_half_width.max(200.0)).clamp(-0.9, 0.9);

        for (s_i, seg) in self.spine.iter_mut().enumerate() {
            let hit_r = (seg.width * 0.55).max(16.0);
            if seg.pos.distance(mouse_world) < hit_r && seg.excite < 0.45 {
                seg.excite = 1.0;
                seg.pos += mouse_vel * 0.015;
                let kind = if s_i == 0 {
                    NodeSoundKind::OcularCore
                } else {
                    NodeSoundKind::SpineChamber
                };
                audio.play_specimen_sound(
                    kind,
                    self.root_freq,
                    self.scale_id,
                    self.fm_ratio,
                    self.fm_brightness,
                    seg.pitch_idx,
                    pan_mouse,
                    brush_intensity,
                );
            }
        }

        for leg in &mut self.legs {
            if leg.knee_pos.distance(mouse_world) < 16.0 && leg.excite_knee < 0.45 {
                leg.excite_knee = 1.0;
                leg.touch_offset += mouse_vel * 0.025 + Vec2::new(0.0, leg.side * 8.0);
                audio.play_specimen_sound(
                    NodeSoundKind::LegJoint,
                    self.root_freq,
                    self.scale_id,
                    self.fm_ratio,
                    self.fm_brightness,
                    leg.pitch_idx,
                    pan_mouse,
                    brush_intensity,
                );
            }
            if leg.ankle_pos.distance(mouse_world) < 16.0 && leg.excite_ankle < 0.45 {
                leg.excite_ankle = 1.0;
                leg.touch_offset += mouse_vel * 0.03 + Vec2::new(0.0, leg.side * 10.0);
                audio.play_specimen_sound(
                    NodeSoundKind::LegJoint,
                    self.root_freq,
                    self.scale_id,
                    self.fm_ratio,
                    self.fm_brightness,
                    (leg.pitch_idx + 3) % 16,
                    pan_mouse,
                    brush_intensity,
                );
            }
            if leg.foot_pos.distance(mouse_world) < 16.0 && leg.excite_foot < 0.45 {
                leg.excite_foot = 1.0;
                audio.play_specimen_sound(
                    NodeSoundKind::LegJoint,
                    self.root_freq,
                    self.scale_id,
                    self.fm_ratio,
                    self.fm_brightness,
                    (leg.pitch_idx + 7) % 16,
                    pan_mouse,
                    brush_intensity,
                );
            }
        }

        for tent in &mut self.tentacles {
            for node in &mut tent.nodes {
                if node.pos.distance(mouse_world) < 16.0 && node.excite < 0.45 {
                    node.excite = 1.0;
                    node.vel += mouse_vel * 0.42;
                    audio.play_specimen_sound(
                        NodeSoundKind::TentacleBead,
                        self.root_freq,
                        self.scale_id,
                        self.fm_ratio,
                        self.fm_brightness,
                        node.pitch_idx,
                        pan_mouse,
                        brush_intensity,
                    );
                }
            }
        }
    }

    pub fn draw(&self, web: &KnowledgeWeb) {
        let (pr, pg, pb) = self.primary_rgb;
        let (sr, sg, sb) = self.secondary_rgb;
        let (lr, lg, lb) = self.laser_rgb;

        for b in &self.active_beams {
            let alpha = (1.0 - b.life / b.max_life).clamp(0.0, 1.0);
            let (br, bg, bb) = b.color;
            if b.is_jagged_arc {
                let steps = 6;
                let mut prev = b.from;
                let dir = b.to - b.from;
                let perp = Vec2::new(-dir.y, dir.x).normalize_or_zero();
                for s in 1..=steps {
                    let frac = s as f32 / (steps as f32);
                    let jitter = if s == steps {
                        0.0
                    } else {
                        ((self.phase * 19.0 + s as f32 * 4.3).sin()) * 16.0
                    };
                    let next = b.from + dir * frac + perp * jitter;
                    draw_line(prev.x, prev.y, next.x, next.y, 2.8 * alpha, Color::new(br, bg, bb, alpha));
                    draw_line(prev.x, prev.y, next.x, next.y, 1.2, Color::new(1.0, 1.0, 1.0, alpha));
                    prev = next;
                }
            } else {
                draw_line(b.from.x, b.from.y, b.to.x, b.to.y, 10.0 * alpha, Color::new(br, bg, bb, 0.35 * alpha));
                draw_line(b.from.x, b.from.y, b.to.x, b.to.y, 4.0 * alpha, Color::new(br, bg, bb, 0.98 * alpha));
                draw_line(b.from.x, b.from.y, b.to.x, b.to.y, 1.6, Color::new(1.0, 1.0, 1.0, alpha));
            }
        }

        for leg in &self.legs {
            if let Some(wid) = leg.tethered_word_id {
                if wid < web.words.len() {
                    let w_pos = web.words[wid].pos;
                    let (cr, cg, cb) = web.words[wid].chip_rgb;
                    draw_line(leg.foot_pos.x, leg.foot_pos.y, w_pos.x, w_pos.y, 2.4, Color::new(cr, cg, cb, 0.95));
                }
            }
        }

        for tent in &self.tentacles {
            for i in 0..tent.nodes.len() {
                let n = &tent.nodes[i];
                let ang = if i + 1 < tent.nodes.len() {
                    let d = tent.nodes[i + 1].pos - n.pos;
                    d.y.atan2(d.x)
                } else if i > 0 {
                    let d = n.pos - tent.nodes[i - 1].pos;
                    d.y.atan2(d.x)
                } else {
                    0.0
                };

                let box_s = n.box_size * (1.0 + n.excite * 0.38);
                draw_rotated_rect(
                    n.pos,
                    box_s,
                    box_s * 0.88,
                    ang,
                    Color::new(pr, pg, pb, 0.08 + n.excite * 0.22),
                    Color::new(sr, sg, sb, 0.30 + n.excite * 0.65),
                    1.4,
                );

                if i + 1 < tent.nodes.len() {
                    let next = &tent.nodes[i + 1];
                    draw_line(n.pos.x, n.pos.y, next.pos.x, next.pos.y, 1.8, Color::new(sr, sg, sb, 0.65 + n.excite * 0.35));
                }

                let dot_r = 2.4 + n.excite * 2.8;
                draw_circle(n.pos.x, n.pos.y, dot_r, Color::new(1.0, 1.0, 1.0, 0.98));
            }
        }

        for leg in &self.legs {
            let limb_col = Color::new(sr, sg, sb, 0.78);
            draw_line(leg.hip_pos.x, leg.hip_pos.y, leg.knee_pos.x, leg.knee_pos.y, 2.0, limb_col);
            draw_line(leg.knee_pos.x, leg.knee_pos.y, leg.ankle_pos.x, leg.ankle_pos.y, 1.6, limb_col);
            draw_line(leg.ankle_pos.x, leg.ankle_pos.y, leg.foot_pos.x, leg.foot_pos.y, 1.4, limb_col);

            let joints = [
                (leg.knee_pos, leg.excite_knee, 15.5),
                (leg.ankle_pos, leg.excite_ankle, 13.0),
                (leg.foot_pos, leg.excite_foot, 10.5),
            ];
            for &(j_pos, exc, b_size) in &joints {
                let s = b_size * (1.0 + exc * 0.48);
                let rot = match leg.joint_box_shape {
                    0 => 0.785,
                    1 => 0.0,
                    2 => self.phase * 0.5,
                    _ => 0.39,
                };
                draw_rotated_rect(
                    j_pos,
                    s,
                    s,
                    rot,
                    Color::new(pr, pg, pb, 0.06 + exc * 0.25),
                    Color::new(sr, sg, sb, 0.28 + exc * 0.75),
                    1.2,
                );
                let r = 2.6 + exc * 2.8;
                draw_circle(j_pos.x, j_pos.y, r, Color::new(pr, pg, pb, 1.0));
            }
        }

        for (i, seg) in self.spine.iter().enumerate() {
            let forward = Vec2::new(seg.angle.cos(), seg.angle.sin());
            let right = Vec2::new(-forward.y, forward.x);

            match self.dorsal_style {
                DorsalStyle::WireframeWings if i == 0 || (i == 1 && self.spine.len() > 2) => {
                    for &side in &[-1.0_f32, 1.0_f32] {
                        let flap = (self.phase * 2.2).sin() * 0.18;
                        let wing_ang = seg.angle + side * (1.25 + flap);
                        let w_dir = Vec2::new(wing_ang.cos(), wing_ang.sin());
                        let tip = seg.pos + w_dir * (seg.width * 2.1);
                        let mid = seg.pos + w_dir * (seg.width * 1.2) - forward * 20.0;
                        draw_line(seg.pos.x, seg.pos.y, tip.x, tip.y, 1.8, Color::new(lr, lg, lb, 0.65));
                        draw_line(tip.x, tip.y, mid.x, mid.y, 1.4, Color::new(sr, sg, sb, 0.55));
                        draw_line(mid.x, mid.y, seg.pos.x, seg.pos.y, 1.4, Color::new(sr, sg, sb, 0.55));
                    }
                }
                DorsalStyle::OrbitingSatellites if i % 3 == 0 => {
                    let orb_ang = self.phase * 1.8 + i as f32 * 2.1;
                    let sat_pos = seg.pos
                        + Vec2::new(orb_ang.cos(), orb_ang.sin()) * (seg.width * 1.25 + 16.0);
                    draw_rotated_rect(
                        sat_pos,
                        12.0,
                        12.0,
                        orb_ang,
                        Color::new(lr, lg, lb, 0.3),
                        Color::new(lr, lg, lb, 0.95),
                        1.4,
                    );
                    draw_circle(sat_pos.x, sat_pos.y, 2.8, Color::new(1.0, 1.0, 1.0, 1.0));
                }
                DorsalStyle::LateralFins if i % 2 == 0 => {
                    for &side in &[-1.0_f32, 1.0_f32] {
                        let fin = seg.pos + right * (side * seg.width * 0.9) - forward * 9.0;
                        draw_line(seg.pos.x, seg.pos.y, fin.x, fin.y, 2.2, Color::new(pr, pg, pb, 0.75));
                    }
                }
                DorsalStyle::DorsalSpikes => {
                    for &side in &[-1.0_f32, 1.0_f32] {
                        let base = seg.pos + right * (side * seg.width * 0.55);
                        let tip = base + right * (side * 14.0) - forward * 7.0;
                        draw_line(base.x, base.y, tip.x, tip.y, 1.8, Color::new(lr, lg, lb, 0.9));
                    }
                }
                _ => {}
            }
        }

        for (i, seg) in self.spine.iter().enumerate() {
            let exc = seg.excite;
            let outer_w = seg.width * (1.0 + exc * 0.18);
            let outer_l = seg.length * (1.0 + exc * 0.18);
            let forward = Vec2::new(seg.angle.cos(), seg.angle.sin());
            let right = Vec2::new(-forward.y, forward.x);

            draw_rotated_rect(
                seg.pos,
                outer_l + 6.0,
                outer_w + 6.0,
                seg.angle,
                Color::new(sr, sg, sb, 0.18 + exc * 0.25),
                Color::new(sr, sg, sb, 0.92),
                1.8,
            );

            match seg.style {
                ChamberStyle::HollowRect => {
                    draw_rotated_rect(
                        seg.pos,
                        outer_l * 0.75,
                        outer_w * 0.75,
                        seg.angle,
                        Color::new(0.02, 0.02, 0.04, 0.95),
                        Color::new(pr, pg, pb, 0.98),
                        2.2,
                    );
                }
                ChamberStyle::DualCell => {
                    for &side in &[-1.0_f32, 1.0_f32] {
                        let sub = seg.pos + right * (side * outer_w * 0.23);
                        draw_rotated_rect(
                            sub,
                            outer_l * 0.75,
                            outer_w * 0.38,
                            seg.angle,
                            Color::new(pr, pg, pb, 0.32 + exc * 0.4),
                            Color::new(pr, pg, pb, 0.98),
                            2.0,
                        );
                    }
                }
                ChamberStyle::CrossCore => {
                    draw_rotated_rect(
                        seg.pos,
                        outer_l * 0.85,
                        outer_w * 0.85,
                        seg.angle,
                        Color::new(pr, pg, pb, 0.26 + exc * 0.35),
                        Color::new(pr, pg, pb, 0.98),
                        1.8,
                    );
                    draw_circle(
                        seg.pos.x,
                        seg.pos.y,
                        (outer_w * 0.24).min(8.0),
                        Color::new(lr, lg, lb, 0.96),
                    );
                }
                ChamberStyle::NestedDiamond => {
                    draw_rotated_rect(
                        seg.pos,
                        outer_l * 0.72,
                        outer_w * 0.72,
                        seg.angle + 0.785,
                        Color::new(lr, lg, lb, 0.28 + exc * 0.4),
                        Color::new(pr, pg, pb, 0.98),
                        1.8,
                    );
                }
                ChamberStyle::HexFrame | ChamberStyle::RibbedCompartment => {
                    let back = seg.pos - forward * (outer_l * 0.20);
                    let front = seg.pos + forward * (outer_l * 0.22);
                    draw_rotated_rect(
                        back,
                        outer_l * 0.46,
                        outer_w * 0.85,
                        seg.angle,
                        Color::new(pr, pg, pb, 0.36 + exc * 0.4),
                        Color::new(sr, sg, sb, 0.95),
                        1.8,
                    );
                    draw_rotated_rect(
                        front,
                        outer_l * 0.36,
                        outer_w * 0.85,
                        seg.angle,
                        Color::new(sr, sg, sb, 0.5 + exc * 0.4),
                        Color::new(1.0, 1.0, 1.0, 0.92),
                        1.6,
                    );
                }
            }

            if i == 0 {
                match self.head_style {
                    HeadStyle::TwinOcular => {
                        for &side in &[-1.0_f32, 1.0_f32] {
                            let eye = seg.pos
                                + forward * (outer_l * 0.24)
                                + right * (side * outer_w * 0.24);
                            draw_circle(eye.x, eye.y, 4.8 + exc * 2.2, Color::new(1.0, 1.0, 1.0, 1.0));
                        }
                    }
                    HeadStyle::CyclopsLaserLens => {
                        let lens = seg.pos + forward * (outer_l * 0.28);
                        draw_circle_lines(lens.x, lens.y, 8.5 + exc * 3.5, 2.0, Color::new(lr, lg, lb, 0.98));
                        draw_circle(lens.x, lens.y, 4.5 + exc * 2.2, Color::new(1.0, 1.0, 1.0, 1.0));
                    }
                    HeadStyle::ArachnidMultiEye => {
                        for e_i in 0..3 {
                            for &side in &[-1.0_f32, 1.0_f32] {
                                let eye = seg.pos
                                    + forward * (outer_l * (0.32 - e_i as f32 * 0.12))
                                    + right * (side * outer_w * (0.16 + e_i as f32 * 0.10));
                                draw_circle(eye.x, eye.y, 2.8, Color::new(lr, lg, lb, 1.0));
                            }
                        }
                    }
                    HeadStyle::MandiblePincers => {
                        let pinch = (self.phase * 2.5).sin().abs() * 0.28;
                        for &side in &[-1.0_f32, 1.0_f32] {
                            let base = seg.pos
                                + forward * (outer_l * 0.48)
                                + right * (side * outer_w * 0.32);
                            let mid = base + forward * 18.0 + right * (side * 9.0);
                            let tip =
                                mid + forward * 14.0 - right * (side * (11.0 - pinch * 20.0));
                            draw_line(base.x, base.y, mid.x, mid.y, 2.6, Color::new(pr, pg, pb, 0.98));
                            draw_line(mid.x, mid.y, tip.x, tip.y, 2.0, Color::new(1.0, 1.0, 1.0, 0.98));
                        }
                    }
                    HeadStyle::CrescentCrest | HeadStyle::TriProngCrown => {
                        for p_i in -1..=1 {
                            let prong = seg.pos
                                + forward * (outer_l * 0.52 + 14.0)
                                + right * (p_i as f32 * outer_w * 0.34);
                            draw_line(seg.pos.x, seg.pos.y, prong.x, prong.y, 2.0, Color::new(lr, lg, lb, 0.96));
                            draw_circle(prong.x, prong.y, 3.0, Color::new(1.0, 1.0, 1.0, 1.0));
                        }
                    }
                }
            }
        }
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (r + m, g + m, b + m)
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut diff = (b - a) % std::f32::consts::TAU;
    if diff > std::f32::consts::PI {
        diff -= std::f32::consts::TAU;
    } else if diff < -std::f32::consts::PI {
        diff += std::f32::consts::TAU;
    }
    a + diff * t
}

pub fn draw_rotated_rect(
    center: Vec2,
    length: f32,
    width: f32,
    angle: f32,
    fill: Color,
    stroke: Color,
    stroke_thickness: f32,
) {
    let c = angle.cos();
    let s = angle.sin();
    let forward = Vec2::new(c, s) * (length * 0.5);
    let right = Vec2::new(-s, c) * (width * 0.5);

    let p0 = center + forward + right;
    let p1 = center + forward - right;
    let p2 = center - forward - right;
    let p3 = center - forward + right;

    if fill.a > 0.01 {
        draw_triangle(p0, p1, p2, fill);
        draw_triangle(p0, p2, p3, fill);
    }

    draw_line(p0.x, p0.y, p1.x, p1.y, stroke_thickness, stroke);
    draw_line(p1.x, p1.y, p2.x, p2.y, stroke_thickness, stroke);
    draw_line(p2.x, p2.y, p3.x, p3.y, stroke_thickness, stroke);
    draw_line(p3.x, p3.y, p0.x, p0.y, stroke_thickness, stroke);
}