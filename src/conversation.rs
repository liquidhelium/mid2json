pub const PIANO_KEY_COUNT: u8 = 88;
pub const C4_POS: u8 = 60;
use crate::rpe;
use midly::{
    num::{u24, u7},
    MetaMessage, MidiMessage, Smf, Track, TrackEvent, TrackEventKind,
};

use std::{borrow::Cow, path::PathBuf};

pub(crate) fn fill_meta(
    meta: &mut rpe::RPEMetadata,
    smf: &Smf,
    song: PathBuf,
    background: PathBuf,
) {
    meta.background = background
        .file_name()
        .map_or("missingno".into(), |a| a.to_string_lossy().into_owned());
    meta.song = song
        .file_name()
        .map_or("missingno".into(), |a| a.to_string_lossy().into_owned());
    let mut meta_tracks = smf.tracks.iter().filter(|t| ismeta(t));
    let name = meta_tracks.find_map(track_name);
    let name = name
        .and_then(|n| Some(Cow::into_owned(n)))
        .unwrap_or("Generated".to_string());
    meta.name = name;
}

pub(crate) fn fill_lines(chart: &mut rpe::RPEChart, smf: &Smf, ticks_per_beat: u32) {
    chart.judge_line_list = smf
        .tracks
        .iter()
        .filter(|t| !ismeta(t))
        .map(|t| midi_track_to_judge_line(t, ticks_per_beat))
        .collect();
}

pub(crate) fn ismeta(t: &Vec<TrackEvent>) -> bool {
    t.iter().all(|e| matches!(e.kind, TrackEventKind::Meta(_)))
}

pub(crate) fn fill_bpm(bpm: &mut Vec<rpe::RPEBpmItem>, smf: &Smf, ticks_per_beat: u32) {
    *bpm = events_to_bpm(smf.tracks.iter().flatten(), ticks_per_beat);
}

pub(crate) fn midi_track_to_judge_line(track: &Track, ticks_per_beat: u32) -> rpe::RPEJudgeLine {
    let mut line = rpe::RPEJudgeLine::default();
    line.name = track_name(track).unwrap_or_default().to_string();
    line.notes = midi_track_to_notes(track, ticks_per_beat);
    line.num_of_notes = line.notes.len();
    line
}

pub(crate) fn midi_track_to_notes(track: &Track, ticks_per_beat: u32) -> Vec<rpe::RPENote> {
    let mut current_tick = 0;
    // only "note_on", "note_off"
    let mut _event_to_note = |event: &TrackEvent| {
        current_tick += event.delta.as_int();
        let mut ret = rpe::RPENote::default();
        match event.kind {
            TrackEventKind::Midi {
                message: MidiMessage::NoteOn { key, .. },
                ..
            } => {
                fill_note(&mut ret, current_tick, ticks_per_beat, key);
            }

            _ => return None,
        };
        Some(ret)
    };
    track.iter().filter_map(|e| _event_to_note(e)).collect()
}

pub(crate) fn fill_note(ret: &mut rpe::RPENote, current_tick: u32, ticks_per_beat: u32, key: u7) {
    ret.start_time = tick_to_rpe_time(current_tick, ticks_per_beat);
    ret.end_time = ret.start_time.clone();
    ret.position_x = key_to_x_value(key);
}

pub(crate) fn track_name<'a>(track: &Track<'a>) -> Option<Cow<'a, str>> {
    track.iter().find_map(|e| match e.kind {
        TrackEventKind::Meta(MetaMessage::TrackName(n)) => Some(String::from_utf8_lossy(n)),
        _ => None,
    })
}

/// assume that the song is 4/4.
pub(crate) fn tempo2bpm(tempo: u24) -> f32 {
    60. * 1e6 / tempo.as_int() as f32 * 4. / 4.
}

pub(crate) fn tick_to_rpe_time(tick: u32, ticks_per_beat: u32) -> rpe::Triple {
    rpe::Triple(
        (tick / ticks_per_beat).try_into().unwrap(),
        tick % ticks_per_beat,
        ticks_per_beat,
    )
}

pub(crate) fn key_to_x_value(key: u7) -> f32 {
    // C4(the middle c) is on center.
    (key.as_int() as i8 - C4_POS as i8) as f32 / (PIANO_KEY_COUNT as f32) * rpe::RPE_WIDTH
}

pub(crate) fn events_to_bpm<'a>(
    track: impl Iterator<Item = &'a TrackEvent<'a>>,
    ticks_per_beat: u32,
) -> Vec<rpe::RPEBpmItem> {
    let mut accumulated_time = 0;
    track
        .filter_map(|a| {
            accumulated_time += a.delta.as_int();
            match a.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(t)) => Some(rpe::RPEBpmItem {
                    bpm: tempo2bpm(t),
                    start_time: tick_to_rpe_time(accumulated_time, ticks_per_beat),
                }),
                _ => None,
            }
        })
        .collect()
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn c4_on_center() {
        assert_eq!(key_to_x_value(u7::from_int_lossy(C4_POS)) as i32, 0)
    }
}
