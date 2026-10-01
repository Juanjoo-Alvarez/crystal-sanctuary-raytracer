use raylib::prelude::{Music, RaylibAudio, Sound};
use std::path::Path;

const SAMPLE_RATE: u32 = 22_050;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MusicTrack {
    Title,
    Hills,
    Haunted,
    Beach,
}

pub struct MusicDirector<'audio> {
    title: Music<'audio>,
    hills: Music<'audio>,
    haunted: Music<'audio>,
    beach: Music<'audio>,
    active: MusicTrack,
    muted: bool,
}

impl<'audio> MusicDirector<'audio> {
    pub fn new(audio: &'audio RaylibAudio) -> Self {
        let title = load_music(
            audio,
            include_bytes!("../assets/audio/CaptainTodTitleScreen.ogg"),
            0.34,
        );
        let hills = load_music(
            audio,
            include_bytes!("../assets/audio/CapitanTodNoCopy.ogg"),
            0.32,
        );
        let haunted = load_music(
            audio,
            include_bytes!("../assets/audio/CaptainTodBooMansion.ogg"),
            0.31,
        );
        let beach = load_music(
            audio,
            include_bytes!("../assets/audio/CaptainTodBeach.ogg"),
            0.33,
        );
        title.play_stream();
        Self {
            title,
            hills,
            haunted,
            beach,
            active: MusicTrack::Title,
            muted: false,
        }
    }

    fn active_music(&self) -> &Music<'audio> {
        match self.active {
            MusicTrack::Title => &self.title,
            MusicTrack::Hills => &self.hills,
            MusicTrack::Haunted => &self.haunted,
            MusicTrack::Beach => &self.beach,
        }
    }

    pub fn update(&self) {
        self.active_music().update_stream();
    }

    pub fn switch_to(&mut self, track: MusicTrack) {
        if self.active == track {
            return;
        }
        self.active_music().stop_stream();
        self.active = track;
        self.active_music().play_stream();
        if self.muted {
            self.active_music().pause_stream();
        }
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            self.active_music().pause_stream();
        } else {
            self.active_music().resume_stream();
        }
    }
}

fn load_music<'audio>(audio: &'audio RaylibAudio, bytes: &[u8], volume: f32) -> Music<'audio> {
    let mut music = audio
        .new_music_from_memory(".ogg", bytes)
        .expect("No se pudo cargar una pista musical");
    music.set_looping(true);
    music.set_volume(volume);
    music
}

#[derive(Clone, Copy)]
enum Effect {
    Begin,
    Collect,
    Crystal,
    Hit,
    Victory,
}

pub struct GameSounds<'audio> {
    begin: Sound<'audio>,
    collect: Sound<'audio>,
    crystal: Sound<'audio>,
    hit: Sound<'audio>,
    victory: Sound<'audio>,
}

impl<'audio> GameSounds<'audio> {
    pub fn new(audio: &'audio RaylibAudio) -> Self {
        let begin = load_effect(audio, Effect::Begin);
        let collect = load_effect(audio, Effect::Collect);
        let crystal = load_effect(audio, Effect::Crystal);
        let hit = load_effect(audio, Effect::Hit);
        let victory = load_effect(audio, Effect::Victory);
        begin.set_volume(0.48);
        collect.set_volume(0.58);
        crystal.set_volume(0.58);
        hit.set_volume(0.52);
        victory.set_volume(0.60);
        Self {
            begin,
            collect,
            crystal,
            hit,
            victory,
        }
    }

    pub fn play_begin(&self) {
        self.begin.play();
    }

    pub fn play_collect(&self) {
        self.collect.play();
    }

    pub fn play_crystal(&self) {
        self.crystal.play();
    }

    pub fn play_hit(&self) {
        self.hit.play();
    }

    pub fn play_victory(&self) {
        self.victory.play();
    }
}

fn load_effect<'audio>(audio: &'audio RaylibAudio, effect: Effect) -> Sound<'audio> {
    let wave_bytes = synthesize_wav(effect);
    let wave = audio
        .new_wave_from_memory(".wav", &wave_bytes)
        .expect("No se pudo sintetizar un efecto de sonido");
    audio
        .new_sound_from_wave(&wave)
        .expect("No se pudo cargar un efecto de sonido")
}

fn synthesize_wav(effect: Effect) -> Vec<u8> {
    let duration = match effect {
        Effect::Begin => 0.62,
        Effect::Collect => 0.34,
        Effect::Crystal => 1.35,
        Effect::Hit => 0.30,
        Effect::Victory => 1.55,
    };
    let sample_count = (duration * SAMPLE_RATE as f32) as usize;
    let mut pcm = Vec::with_capacity(sample_count);
    let mut noise = 0x51A7_2485_u32;

    for index in 0..sample_count {
        let time = index as f32 / SAMPLE_RATE as f32;
        let progress = time / duration;
        let attack = (time / 0.018).clamp(0.0, 1.0);
        let release = (1.0 - progress).clamp(0.0, 1.0).powf(1.7);
        let envelope = attack * release;
        let tau = std::f32::consts::TAU;
        let sample = match effect {
            Effect::Begin => {
                let note = if progress < 0.32 {
                    392.0
                } else if progress < 0.64 {
                    523.25
                } else {
                    659.25
                };
                ((tau * note * time).sin() + (tau * note * 2.0 * time).sin() * 0.22)
                    * envelope
                    * 0.42
            }
            Effect::Collect => {
                let note = if progress < 0.42 { 783.99 } else { 1174.66 };
                ((tau * note * time).sin() + (tau * note * 1.5 * time).sin() * 0.25)
                    * envelope
                    * 0.52
            }
            Effect::Crystal => {
                let frequency = 310.0 + progress * 720.0;
                let shimmer = (tau * (frequency * 2.01) * time).sin() * 0.22
                    + (tau * (frequency * 3.98) * time).sin() * 0.10;
                ((tau * frequency * time).sin() * 0.58 + shimmer) * envelope * 0.48
            }
            Effect::Hit => {
                noise = noise.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let random = ((noise >> 9) as f32 / 8_388_607.0) * 2.0 - 1.0;
                let frequency = 135.0 - progress * 76.0;
                ((tau * frequency * time).sin() * 0.72 + random * 0.28) * envelope * 0.55
            }
            Effect::Victory => {
                let chord = if progress < 0.25 {
                    (523.25, 659.25, 783.99)
                } else if progress < 0.50 {
                    (587.33, 739.99, 880.0)
                } else if progress < 0.75 {
                    (659.25, 783.99, 987.77)
                } else {
                    (783.99, 987.77, 1174.66)
                };
                ((tau * chord.0 * time).sin() * 0.48
                    + (tau * chord.1 * time).sin() * 0.31
                    + (tau * chord.2 * time).sin() * 0.21)
                    * envelope
                    * 0.40
            }
        };
        pcm.push((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
    }

    encode_pcm_wav(&pcm)
}

pub fn export_effects(directory: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(directory)?;
    for (name, effect) in [
        ("begin.wav", Effect::Begin),
        ("collect.wav", Effect::Collect),
        ("crystal.wav", Effect::Crystal),
        ("hit.wav", Effect::Hit),
        ("victory.wav", Effect::Victory),
    ] {
        std::fs::write(directory.join(name), synthesize_wav(effect))?;
    }
    Ok(())
}

fn encode_pcm_wav(samples: &[i16]) -> Vec<u8> {
    let data_size = std::mem::size_of_val(samples) as u32;
    let mut bytes = Vec::with_capacity(44 + data_size as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_effect_is_a_valid_pcm_wav() {
        let wav = synthesize_wav(Effect::Collect);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert!(wav.len() > 44);
    }
}
