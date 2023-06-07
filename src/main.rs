///! Midi to RPE json.
///! All the RPE Chart structure definitions are from [prpr](https://github.com/Mivik/prpr).
use std::{
    error,
    ffi::OsStr,
    fmt::Display,
    fs::{self, File},
    io,
    path::PathBuf,
};

use clap::Parser;

use midly::{Smf, Timing};

mod rpe;

#[derive(Parser, Debug)]
#[command(name = "Midi to Rpe", version, author)]
struct Args {
    /// Name of the input file.
    #[arg()]
    midi_path: PathBuf,
    /// Id of the target chart.
    #[arg(long = "id")]
    target_id: Option<i32>,
    /// Song file referred in the chart.
    #[arg(long)]
    song_file: Option<PathBuf>,
    /// Background image referred in the chart.
    #[arg(long)]
    background_file: Option<PathBuf>,
    /// The path of the conversation result.
    #[arg(short, long = "output")]
    output_path: Option<PathBuf>,
    /// seprate the keys.
    #[arg(short, long = "seprate")]
    sepration_rate: Option<f32>,
    #[arg(short = 'v')]
    speed: Option<f32>,
}

#[derive(Debug)]
struct Error(&'static str);
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)?;
        Ok(())
    }
}
impl error::Error for Error {}

fn main() {
    run(Args::parse()).unwrap_or_else(|e| {
        eprintln!("Something is wrong...I can feel it\n");
        eprintln!("Fault: \n {e:#?}");
        detailed_errmsg(e);
    })
}

fn detailed_errmsg(e: Box<dyn error::Error>) {
    e.downcast_ref::<io::Error>()
        .map(|_| eprintln!("..when tried to open the file"));
    e.downcast_ref::<midly::Error>()
        .map(|_| eprintln!("..when tried to read the midi file."));
}

fn run(args: Args) -> Result<(), Box<dyn error::Error>> {
    let (midi_path, sepration_rate, speed, output_path, song_file, background_file) =
        process_args(args);
    let mut chart = rpe::RPEChart::default();
    let file = fs::read(&midi_path)?;
    let smf = Smf::parse(&file)?;
    let ticks_per_beat = match smf.header.timing {
        Timing::Metrical(t) => t.as_int() as u32,
        _ => Err(midly::Error::new(&midly::ErrorKind::Invalid(
            "We support tick per beat times only.",
        )))?,
    };
    conversation::fill_meta(&mut chart.meta, &smf, song_file, background_file);
    conversation::fill_bpm(&mut chart.bpm_list, &smf, ticks_per_beat);
    conversation::fill_lines(&mut chart, &smf, ticks_per_beat);
    post_process(sepration_rate, &mut chart, speed);
    serde_json::to_writer(File::create(output_path)?, &chart)?;
    Ok(())
}

fn post_process(sepration_rate: Option<f32>, chart: &mut rpe::RPEChart, speed: Option<f32>) {
    if let Some(rate) = sepration_rate {
        chart
            .judge_line_list
            .iter_mut()
            .flat_map(|l| l.notes.iter_mut())
            .for_each(|n| n.position_x *= rate)
    }
    if let Some(speed) = speed {
        chart.judge_line_list.iter_mut().for_each(|j| {
            if let Some(Some(rpe::RPEEventLayer {
                speed_events: Some(vec),
                ..
            })) = j.event_layers.get_mut(0)
            {
                if let Some(rpe::RPESpeedEvent { start, end, .. }) = vec.get_mut(0) {
                    *start = speed;
                    *end = speed;
                }
            }
        })
    }
}

fn process_args(args: Args) -> (PathBuf, Option<f32>, Option<f32>, PathBuf, PathBuf, PathBuf) {
    let Args {
        midi_path,
        target_id,
        song_file,
        background_file,
        output_path,
        sepration_rate,
        speed,
    } = args;
    let mut output_path = output_path.unwrap_or_default();
    if let Some(id) = target_id {
        output_path = (id.to_string() + ".json").into();
    } else if output_path.is_dir() || output_path.as_os_str().len() == 0 {
        let file_name = midi_path.file_stem().unwrap_or(OsStr::new("result"));
        output_path.push(file_name);
        output_path.set_extension("json");
    }
    let target_id = target_id.unwrap_or(114514);
    let song_file = song_file.unwrap_or((target_id.to_string() + ".mp3").into());
    let background_file = background_file.unwrap_or((target_id.to_string() + ".png").into());
    (
        midi_path,
        sepration_rate,
        speed,
        output_path,
        song_file,
        background_file,
    )
}

mod conversation;
