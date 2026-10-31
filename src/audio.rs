use rodio::{OutputStream, OutputStreamHandle, Source};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub enum NodeSoundKind {
    SpineChamber, LegJoint, TentacleBead, OcularCore, Footstep, LaserBeam, WordShatter, ArcZap,
}

#[derive(Clone, Copy, Debug)]
pub enum AudioEvent {
    TriggerVoice { freq: f32, freq_end: f32, pan: f32, amplitude: f32, decay_sec: f32, fm_index: f32, fm_ratio: f32, noise_mix: f32 },
    SetDroneMood { root_freq: f32, fm_ratio: f32, shimmer: f32, warmth: f32, muted: bool },
}

const SCALES: [[f32; 8]; 8] = [
    [1.0, 9.0/8.0, 5.0/4.0, 45.0/32.0, 3.0/2.0, 5.0/3.0, 15.0/8.0, 2.0],
    [1.0, 9.0/8.0, 6.0/5.0, 3.0/2.0, 8.0/5.0, 2.0, 9.0/4.0, 12.0/5.0],
    [1.0, 16.0/15.0, 5.0/4.0, 4.0/3.0, 3.0/2.0, 8.0/5.0, 9.0/5.0, 2.0],
    [1.0, 9.0/8.0, 5.0/4.0, 45.0/32.0, 8.0/5.0, 9.0/5.0, 2.0, 9.0/4.0],
    [1.0, 27.0/25.0, 9.0/7.0, 7.0/5.0, 5.0/3.0, 9.0/5.0, 15.0/7.0, 3.0],
    [1.0, 5.0/4.0, 3.0/2.0, 7.0/4.0, 2.0, 9.0/4.0, 5.0/2.0, 11.0/4.0],
    [1.0, 6.0/5.0, 4.0/3.0, 3.0/2.0, 9.0/5.0, 2.0, 12.0/5.0, 8.0/3.0],
    [1.0, 16.0/15.0, 6.0/5.0, 45.0/32.0, 3.0/2.0, 8.0/5.0, 9.0/5.0, 2.0],
];

#[derive(Clone, Copy)]
struct Voice {
    active: bool, phase: f32, mod_phase: f32, freq_start: f32, freq_end: f32,
    pan: f32, amp: f32, env: f32, attack_inc: f32, decay_mul: f32, attacking: bool,
    fm_index: f32, fm_ratio: f32, noise_mix: f32, rng_state: u32,
}

impl Default for Voice {
    fn default() -> Self {
        Self { active: false, phase: 0.0, mod_phase: 0.0, freq_start: 220.0, freq_end: 220.0, pan: 0.0, amp: 0.0, env: 0.0, attack_inc: 0.02, decay_mul: 0.999, attacking: false, fm_index: 0.5, fm_ratio: 2.0, noise_mix: 0.0, rng_state: 0x9E3779B9 }
    }
}

struct TactileSynthSource {
    rx: Receiver<AudioEvent>, sample_rate: u32, voices: [Voice; 48], next_voice: usize,
    drone_phase_a: f32, drone_phase_b: f32, drone_lfo: f32, drone_root: f32, drone_ratio: f32,
    drone_shimmer: f32, drone_warmth: f32, muted: bool, delay_l: Vec<f32>, delay_r: Vec<f32>,
    delay_idx_l: usize, delay_idx_r: usize, pending_right_sample: Option<f32>,
}

impl TactileSynthSource {
    fn new(rx: Receiver<AudioEvent>) -> Self {
        Self {
            rx, sample_rate: 44100, voices: [Voice::default(); 48], next_voice: 0,
            drone_phase_a: 0.0, drone_phase_b: 0.0, drone_lfo: 0.0, drone_root: 73.42,
            drone_ratio: 1.5, drone_shimmer: 0.25, drone_warmth: 0.4, muted: false,
            delay_l: vec![0.0; 14112], delay_r: vec![0.0; 19845], delay_idx_l: 0, delay_idx_r: 0, pending_right_sample: None,
        }
    }

    fn process_events(&mut self) {
        while let Ok(evt) = self.rx.try_recv() {
            match evt {
                AudioEvent::TriggerVoice { freq, freq_end, pan, amplitude, decay_sec, fm_index, fm_ratio, noise_mix } => {
                    let idx = self.next_voice;
                    self.next_voice = (idx + 1) % self.voices.len();
                    let v = &mut self.voices[idx];
                    let sr = self.sample_rate as f32;
                    v.active = true; v.phase = 0.0; v.mod_phase = 0.0;
                    v.freq_start = freq.clamp(38.0, 5200.0); v.freq_end = freq_end.clamp(32.0, 5200.0);
                    v.pan = pan.clamp(-0.95, 0.95); v.amp = amplitude.clamp(0.004, 0.28);
                    v.env = 0.001; v.attacking = true; v.attack_inc = 1.0 / (0.0025 * sr);
                    v.decay_mul = 0.001_f32.powf(1.0 / (decay_sec.max(0.025) * sr).max(60.0));
                    v.fm_index = fm_index.clamp(0.0, 7.0); v.fm_ratio = fm_ratio.clamp(0.25, 11.0);
                    v.noise_mix = noise_mix.clamp(0.0, 0.92);
                    v.rng_state = v.rng_state.wrapping_add((freq as u32).wrapping_mul(7919));
                }
                AudioEvent::SetDroneMood { root_freq, fm_ratio, shimmer, warmth, muted } => {
                    self.drone_root = root_freq.clamp(42.0, 220.0); self.drone_ratio = fm_ratio.clamp(0.5, 4.0);
                    self.drone_shimmer = shimmer.clamp(0.0, 1.0); self.drone_warmth = warmth.clamp(0.0, 1.0);
                    self.muted = muted;
                }
            }
        }
    }

    fn synthesize_stereo_frame(&mut self) -> (f32, f32) {
        self.process_events();
        if self.muted { return (0.0, 0.0); }
        let sr = self.sample_rate as f32;
        let tau = std::f32::consts::TAU;
        let mut mix_l = 0.0_f32; let mut mix_r = 0.0_f32;

        self.drone_lfo = (self.drone_lfo + 0.16 / sr).fract();
        let lfo = (self.drone_lfo * tau).sin() * 0.5 + 0.5;
        self.drone_phase_a = (self.drone_phase_a + self.drone_root / sr).fract();
        self.drone_phase_b = (self.drone_phase_b + (self.drone_root * self.drone_ratio) / sr).fract();

        let drone = (self.drone_phase_a * tau).sin() * 0.52 + (self.drone_phase_b * tau).sin() * (0.24 * self.drone_warmth);
        let drone_amp = 0.016 * (0.65 + 0.35 * lfo);
        mix_l += drone * drone_amp; mix_r += drone * drone_amp;

        for v in &mut self.voices {
            if !v.active { continue; }
            if v.attacking {
                v.env += v.attack_inc;
                if v.env >= 1.0 { v.env = 1.0; v.attacking = false; }
            } else {
                v.env *= v.decay_mul;
                if v.env < 0.0007 { v.active = false; continue; }
            }
            let inst_freq = v.freq_end + (v.freq_start - v.freq_end) * v.env;
            v.phase = (v.phase + inst_freq / sr).fract();
            v.mod_phase = (v.mod_phase + (inst_freq * v.fm_ratio) / sr).fract();
            let mod_sig = (v.mod_phase * tau).sin() * (v.fm_index * v.env);
            let tone = ((v.phase * tau) + mod_sig).sin();
            v.rng_state ^= v.rng_state << 13; v.rng_state ^= v.rng_state >> 17; v.rng_state ^= v.rng_state << 5;
            let white = ((v.rng_state as f32) / (u32::MAX as f32)) * 2.0 - 1.0;
            let sample = (tone * (1.0 - v.noise_mix * 0.45) + white * v.noise_mix * v.env.powi(2)) * v.env * v.amp;
            let angle = (v.pan + 1.0) * 0.25 * std::f32::consts::PI;
            mix_l += sample * angle.cos(); mix_r += sample * angle.sin();
        }

        let dl = self.delay_l[self.delay_idx_l]; let dr = self.delay_r[self.delay_idx_r];
        self.delay_l[self.delay_idx_l] = (mix_l + dr * 0.36).clamp(-1.0, 1.0);
        self.delay_r[self.delay_idx_r] = (mix_r + dl * 0.36).clamp(-1.0, 1.0);
        self.delay_idx_l = (self.delay_idx_l + 1) % self.delay_l.len();
        self.delay_idx_r = (self.delay_idx_r + 1) % self.delay_r.len();

        ((mix_l + dl * 0.28).tanh() * 0.85, (mix_r + dr * 0.28).tanh() * 0.85)
    }
}

impl Iterator for TactileSynthSource {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(r) = self.pending_right_sample.take() { Some(r) }
        else { let (l, r) = self.synthesize_stereo_frame(); self.pending_right_sample = Some(r); Some(l) }
    }
}

impl Source for TactileSynthSource {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { 2 }
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn total_duration(&self) -> Option<Duration> { None }
}

pub struct SoundEngine {
    _stream: Option<OutputStream>, _handle: Option<OutputStreamHandle>, tx: Option<Sender<AudioEvent>>, pub muted: bool,
}

impl SoundEngine {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        if let Ok((stream, handle)) = OutputStream::try_default() {
            let _ = handle.play_raw(TactileSynthSource::new(rx));
            Self { _stream: Some(stream), _handle: Some(handle), tx: Some(tx), muted: false }
        } else {
            Self { _stream: None, _handle: None, tx: None, muted: true }
        }
    }

    pub fn play_specimen_sound(&self, kind: NodeSoundKind, base_freq: f32, scale_id: usize, fm_ratio: f32, fm_brightness: f32, pitch_idx: usize, pan: f32, intensity: f32) {
        if self.muted { return; }
        let scale = &SCALES[scale_id % SCALES.len()];
        let ratio = scale[pitch_idx % scale.len()];
        let octave = (1 << ((pitch_idx / scale.len()) % 3)) as f32;
        let note_freq = base_freq * ratio * octave;
        let vel = intensity.clamp(0.15, 1.5);

        let (f_start, f_end, amp, decay, fm_idx, mod_ratio, noise) = match kind {
            NodeSoundKind::SpineChamber => (note_freq, note_freq * 0.99, 0.12 * vel, 0.52, fm_brightness * 1.4 * vel, fm_ratio, 0.05),
            NodeSoundKind::LegJoint => (note_freq * 1.5, note_freq * 1.2, 0.09 * vel, 0.20, fm_brightness * 2.2 * vel, fm_ratio * 1.414, 0.18),
            NodeSoundKind::TentacleBead => (note_freq * 2.0, note_freq * 2.02, 0.08 * vel, 0.42, fm_brightness * 1.6, fm_ratio * 0.5, 0.06),
            NodeSoundKind::OcularCore => (note_freq * 1.5, note_freq * 1.5, 0.14 * vel, 0.78, fm_brightness * 0.9, fm_ratio * 2.0, 0.02),
            NodeSoundKind::Footstep => (note_freq * 0.65, note_freq * 0.35, 0.032 * vel, 0.055, 2.5, fm_ratio * 2.3, 0.42),
            NodeSoundKind::LaserBeam => (note_freq * 3.6, note_freq * 0.42, 0.14 * vel, 0.34, 4.2 * vel, fm_ratio * 1.732, 0.26),
            NodeSoundKind::WordShatter => (note_freq * 0.85, note_freq * 0.22, 0.16 * vel, 0.44, 5.0, fm_ratio * 1.33, 0.68),
            NodeSoundKind::ArcZap => (note_freq * 2.4, note_freq * 3.8, 0.10 * vel, 0.16, 5.5, fm_ratio * 3.14, 0.48),
        };

        if let Some(tx) = &self.tx {
            let _ = tx.send(AudioEvent::TriggerVoice { freq: f_start, freq_end: f_end, pan, amplitude: amp, decay_sec: decay, fm_index: fm_idx, fm_ratio: mod_ratio, noise_mix: noise });
        }
    }

    pub fn update_drone(&self, specimen_root_freq: f32, specimen_fm_ratio: f32, curiosity: f32, vitality: f32) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(AudioEvent::SetDroneMood { root_freq: specimen_root_freq * 0.5, fm_ratio: specimen_fm_ratio, shimmer: curiosity, warmth: vitality, muted: self.muted });
        }
    }
}
