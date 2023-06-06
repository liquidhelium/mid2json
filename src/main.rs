///! All the RPE Chart structure definitions are from [prpr](https://github.com/Mivik/prpr).
use std::{
    borrow::Cow,
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io,
};

use clap::Parser;
use midly::{num::u24, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[repr(usize)]
pub enum UIElement {
    Bar,
    Pause,
    ComboNumber,
    Combo,
    Score,
    Name,
    Level,
}
#[derive(serde::Deserialize, Serialize)]
pub struct Triple(i32, u32, u32);
impl Default for Triple {
    fn default() -> Self {
        Self(0, 0, 1)
    }
}
#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RPEBpmItem {
    bpm: f32,
    start_time: Triple,
}

// serde is weird...
fn f32_zero() -> f32 {
    0.
}

fn f32_one() -> f32 {
    1.
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RPEEvent<T = f32> {
    // TODO linkgroup
    #[serde(default = "f32_zero")]
    easing_left: f32,
    #[serde(default = "f32_one")]
    easing_right: f32,
    #[serde(default)]
    bezier: u8,
    #[serde(default)]
    bezier_points: [f32; 4],
    easing_type: i32,
    start: T,
    end: T,
    start_time: Triple,
    end_time: Triple,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RPECtrlEvent {
    easing: u8,
    x: f32,
    #[serde(flatten)]
    value: HashMap<String, f32>,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RPESpeedEvent {
    // TODO linkgroup
    start_time: Triple,
    end_time: Triple,
    start: f32,
    end: f32,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RPEEventLayer {
    alpha_events: Option<Vec<RPEEvent>>,
    move_x_events: Option<Vec<RPEEvent>>,
    move_y_events: Option<Vec<RPEEvent>>,
    rotate_events: Option<Vec<RPEEvent>>,
    speed_events: Option<Vec<RPESpeedEvent>>,
}

#[derive(Clone, Deserialize, Serialize, Default)]
struct RGBColor(u8, u8, u8);

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RPEExtendedEvents {
    color_events: Option<Vec<RPEEvent<RGBColor>>>,
    text_events: Option<Vec<RPEEvent<String>>>,
    scale_x_events: Option<Vec<RPEEvent>>,
    scale_y_events: Option<Vec<RPEEvent>>,
    incline_events: Option<Vec<RPEEvent>>,
    paint_events: Option<Vec<RPEEvent>>,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RPENote {
    // TODO above == 0? what does that even mean?
    #[serde(rename = "type")]
    kind: u8,
    above: u8,
    start_time: Triple,
    end_time: Triple,
    position_x: f32,
    y_offset: f32,
    #[serde(default="full_alpha")]
    alpha: u16, // some alpha has 256...
    #[serde(default="f32_one")]
    size: f32,
    speed: f32,
    is_fake: u8,
    visible_time: f32,
}
fn full_alpha() -> u16 {
    255
}
#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RPEJudgeLine {
    // TODO group
    // TODO alphaControl, bpmfactor
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Texture")]
    texture: String,
    #[serde(rename = "father")]
    parent: Option<isize>,
    event_layers: Vec<Option<RPEEventLayer>>,
    extended: Option<RPEExtendedEvents>,
    notes: Vec<RPENote>,
    is_cover: u8,
    #[serde(default)]
    z_order: i32,
    #[serde(rename = "attachUI")]
    attach_ui: Option<UIElement>,

    #[serde(default)]
    pos_control: Vec<RPECtrlEvent>,
    #[serde(default)]
    size_control: Vec<RPECtrlEvent>,
    #[serde(default)]
    alpha_control: Vec<RPECtrlEvent>,
    #[serde(default)]
    y_control: Vec<RPECtrlEvent>,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RPEMetadata {
    offset: i32,
    charter: String,
    composer: String,
    name: String,
    song: String,
    background: String,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RPEChart {
    #[serde(rename = "META")]
    meta: RPEMetadata,
    #[serde(rename = "BPMList")]
    bpm_list: Vec<RPEBpmItem>,
    judge_line_list: Vec<RPEJudgeLine>,
}

#[derive(Parser, Debug)]
#[command(name = "Midi to Rpe", version, author)]
struct Args {
    /// Name of the input file.
    #[arg()]
    midi_path: String,
    /// Id of the target chart.
    #[arg(long = "id")]
    target_id: i32,
    /// Song file referred in the chart.
    #[arg(long)]
    song_file: Option<String>,
    /// Background image referred in the chart.
    #[arg(long)]
    background_file: Option<String>,
    #[arg(short, long = "output")]
    output_path: Option<String>,
}

fn main() {
    run(Args::parse()).unwrap_or_else(|e| {
        eprintln!("Something is wrong...I can feel it\n");
        eprintln!("Fault: \n {e:#?}");
        detailed_errmsg(e);
    })
}

fn detailed_errmsg(e: Box<dyn Error>) {
    e.downcast_ref::<io::Error>()
        .map(|_| eprintln!("..when tried to open the file"));
    e.downcast_ref::<midly::Error>()
        .map(|_| eprintln!("..when tried to read the midi file."));
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let Args {
        midi_path: name,
        target_id,
        song_file,
        background_file,
        output_path,
    } = args;
    let song_file = song_file.unwrap_or(target_id.to_string() + ".mp3");
    let background_file = background_file.unwrap_or(target_id.to_string() + ".png");
    let output_path = output_path.unwrap_or(target_id.to_string() + ".json");
    let mut chart = RPEChart::default();
    let file = fs::read(&name)?;
    let smf = dbg!(Smf::parse(&file)?);
    fill_meta(&mut chart.meta, &smf, song_file, background_file);
    fill_bpm(&mut chart.bpm_list, &smf)?;
    serde_json::to_writer(File::create(output_path)?, &chart)?;
    Ok(())
}

fn fill_meta(meta: &mut RPEMetadata, smf: &Smf, song: String, background: String) {
    meta.background = background;
    meta.song = song;
    let mut meta_tracks = smf.tracks.iter().filter(|t| ismeta(t));
    let name = meta_tracks.find_map(track_name);
    let name = name
        .and_then(|n| Some(Cow::into_owned(n)))
        .unwrap_or("Generated".to_string());
    meta.name = name;
}

fn fill_lines(chart: &mut RPEChart, smf: &Smf) -> Result<(), midly::Error>  {
    let ticks_per_beat = match smf.header.timing {
        Timing::Metrical(t) => t.as_int() as u32,
        _ => Err(midly::Error::new(&midly::ErrorKind::Invalid(
            "We support tick per beat times only.",
        )))?,
    };
    // chart.judge_line_list = smf.tracks.iter().filter(ismeta).map();
    Ok(())
}

fn ismeta(t: &Vec<TrackEvent>) -> bool {
    t.iter().all(|e| matches!(e.kind, TrackEventKind::Meta(_)))
}

fn fill_bpm(bpm: &mut Vec<RPEBpmItem>, smf: &Smf) -> Result<(), midly::Error> {
    *bpm = events_to_bpm(
        smf.tracks.iter().flatten(),
        match smf.header.timing {
            Timing::Metrical(t) => t.as_int() as u32,
            _ => Err(midly::Error::new(&midly::ErrorKind::Invalid(
                "We support tick per beat times only.",
            )))?,
        },
    );
    Ok(())
}

fn midiTrackToJudgeLine(track: &Track,
    ticks_per_beat:u32) -> RPEJudgeLine{
        let mut  line= RPEJudgeLine::default();
        line.name = track_name(track).unwrap_or_default().to_string();
        line.notes = midi_track_to_notes(track, ticks_per_beat);
        line
    }

fn midi_track_to_notes(track: &Track, ticks_per_beat: u32) -> Vec<RPENote> {
    let mut current_tick = 0;
    // only "note_on", "note_off"
    let mut _event_to_note = |event: &TrackEvent| {
        current_tick += event.delta.as_int();
        let mut ret = RPENote::default();
        match event.kind {
            TrackEventKind::Midi {
                message: MidiMessage::NoteOn { key,..},
                ..
            } => {
                ret.start_time = tick_to_rpe_time(current_tick, ticks_per_beat);
                ret.position_x = 0.0; //TODO: midiPitchToXValue
            },
            TrackEventKind::Midi {
                message: MidiMessage::NoteOff { key,..},
                ..
            } =>  {
                ret.start_time = tick_to_rpe_time(current_tick, ticks_per_beat);
                ret.position_x = 0.0; //TODO: midiPitchToXValue
            },

            _ => return None,
        };
        Some(ret)
    };
    track.iter().filter_map(|e| _event_to_note(e)).collect()
}

fn track_name<'a>(track: &Track<'a>) -> Option<Cow<'a, str>> {
    track.iter().find_map(|e| match e.kind {
        TrackEventKind::Meta(MetaMessage::TrackName(n)) => Some(String::from_utf8_lossy(n)),
        _ => None,
    })
}

/// assume that the song is 4/4.
fn tempo2bpm(tempo: u24) -> f32 {
    60. * 1e6 / tempo.as_int() as f32 * 4. / 4.
}

fn tick_to_rpe_time(tick: u32, ticks_per_beat: u32) -> Triple {
    Triple(
        (tick / ticks_per_beat).try_into().unwrap(),
        tick % ticks_per_beat,
        ticks_per_beat,
    )
}

fn events_to_bpm<'a>(
    track: impl Iterator<Item = &'a TrackEvent<'a>>,
    ticks_per_beat: u32,
) -> Vec<RPEBpmItem> {
    let mut accumulated_time = 0;
    track
        .filter_map(|a| {
            accumulated_time += a.delta.as_int();
            match a.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(t)) => Some(RPEBpmItem {
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
    fn test() {
        run(Args {
            midi_path: "test_assets/pi.mid".into(),
            target_id: 0,
            song_file: None,
            background_file: None,
            output_path: Some("generated".into()),
        })
        .unwrap();
    }
}
