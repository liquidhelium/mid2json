///! All the RPE Chart structure definitions are from [prpr](https://github.com/Mivik/prpr).
use std::collections::HashMap;

pub const RPE_WIDTH: f32 = 1350.;
use devault::Devault;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Devault)]
#[devault("Triple(0,0,1)")]
pub struct Triple(pub i32, pub u32, pub u32);

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RPEBpmItem {
    pub(crate) bpm: f32,
    pub(crate) start_time: Triple,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RPEEvent<T = f32> {
    // TODO linkgroup
    pub(crate) easing_left: f32,

    pub(crate) easing_right: f32,

    pub(crate) bezier: u8,

    pub(crate) bezier_points: [f32; 4],
    pub(crate) easing_type: i32,
    pub(crate) start: T,
    pub(crate) end: T,
    pub(crate) start_time: Triple,
    pub(crate) end_time: Triple,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RPECtrlEvent {
    pub(crate) easing: u8,
    pub(crate) x: f32,
    #[serde(flatten)]
    pub(crate) value: HashMap<String, f32>,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RPESpeedEvent {
    // TODO linkgroup
    pub(crate) start_time: Triple,
    pub(crate) end_time: Triple,
    pub(crate) start: f32,
    pub(crate) end: f32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RPEEventLayer {
    pub(crate) alpha_events: Option<Vec<RPEEvent>>,
    pub(crate) move_x_events: Option<Vec<RPEEvent>>,
    pub(crate) move_y_events: Option<Vec<RPEEvent>>,
    pub(crate) rotate_events: Option<Vec<RPEEvent>>,
    pub(crate) speed_events: Option<Vec<RPESpeedEvent>>,
}

pub(crate) const DEFAULT_EVENT_LAYER: &str = include_str!("../default_event_layer.json");

impl Default for RPEEventLayer {
    fn default() -> Self {
        serde_json::from_str(DEFAULT_EVENT_LAYER).unwrap()
    }
}

#[derive(Serialize, Default)]
pub(crate) struct RGBColor(u8, u8, u8);

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RPEExtendedEvents {
    pub(crate) color_events: Option<Vec<RPEEvent<RGBColor>>>,
    pub(crate) text_events: Option<Vec<RPEEvent<String>>>,
    pub(crate) scale_x_events: Option<Vec<RPEEvent>>,
    pub(crate) scale_y_events: Option<Vec<RPEEvent>>,
    pub(crate) incline_events: Option<Vec<RPEEvent>>,
    pub(crate) paint_events: Option<Vec<RPEEvent>>,
}

#[derive(Serialize, Devault)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RPENote {
    // TODO above == 0? what does that even mean?
    #[serde(rename = "type")]
    pub(crate) kind: u8,
    #[devault("1")]
    pub(crate) above: u8,
    pub(crate) start_time: Triple,
    pub(crate) end_time: Triple,
    pub(crate) position_x: f32,
    pub(crate) y_offset: f32,
    #[devault("255")]
    pub(crate) alpha: u16, // some alpha has 256...

    #[devault("1.0")]
    pub(crate) size: f32,
    #[devault("1.0")]
    pub(crate) speed: f32,
    pub(crate) is_fake: u8,
    #[devault("999999.0")]
    pub(crate) visible_time: f32,
}

pub(crate) fn default_event_layer() -> Vec<Option<RPEEventLayer>> {
    vec![Some(Default::default())]
}

#[derive(Serialize, Devault)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RPEJudgeLine {
    #[serde(rename = "Group")]
    #[devault("0")]
    pub(crate) group: i32,
    // TODO alphaControl, bpmfactor
    #[serde(rename = "Name")]
    pub(crate) name: String,
    #[serde(rename = "Texture")]
    pub(crate) texture: String,
    #[serde(rename = "father")]
    pub(crate) parent: Option<isize>,
    #[devault("default_event_layer()")]
    pub(crate) event_layers: Vec<Option<RPEEventLayer>>,
    pub(crate) extended: Option<RPEExtendedEvents>,
    pub(crate) notes: Vec<RPENote>,
    pub(crate) num_of_notes: usize,
    #[devault("1")]
    pub(crate) is_cover: u8,

    pub(crate) z_order: i32,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RPEMetadata {
    pub(crate) offset: i32,
    #[serde(rename = "RPEVersion")]
    pub(crate) rpe_version: i32,
    pub(crate) charter: String,
    pub(crate) composer: String,
    pub(crate) name: String,
    pub(crate) song: String,
    pub(crate) background: String,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RPEChart {
    #[serde(rename = "META")]
    pub(crate) meta: RPEMetadata,
    #[serde(rename = "BPMList")]
    pub(crate) bpm_list: Vec<RPEBpmItem>,
    pub(crate) judge_line_list: Vec<RPEJudgeLine>,
}
