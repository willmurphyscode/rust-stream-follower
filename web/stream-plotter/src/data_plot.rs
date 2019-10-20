use plotters::element::{Rectangle, Text};
use plotters::prelude::*;
use plotters::style::ShapeStyle;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Mood {
    keyword: String,
    positive_count: u64,
    neutral_count: u64,
    negative_count: u64,
}

impl Mood {
    fn center(&self, y_rank: usize) -> (i32, i32) {
        let y: i32 = i32::try_from(y_rank).unwrap();
        let y_offset = y * Y_RANK_SCALE;
        (VERTICAL_CENTER_X_COORD, y_offset)
    }
}

const NEUTRAL_SCALEDOWN: i32 = 5;
const NEUTRAL_HEIGHT: i32 = 30;
const Y_RANK_SCALE: i32 = 40;
const VERTICAL_CENTER_X_COORD: i32 = 200;

fn rank_and_item_to_rectangle(y_rank: usize, mood: &Mood) -> Rectangle<(i32, i32)> {
    let center = mood.center(y_rank);

    Rectangle::new(
        [
            center,
            (
                center.0 + i32::try_from(mood.neutral_count).unwrap() / NEUTRAL_SCALEDOWN,
                center.1 + NEUTRAL_HEIGHT,
            ),
        ],
        ShapeStyle {
            filled: true,
            stroke_width: 1,
            color: plotters::style::YELLOW.to_rgba(),
        },
    )
}

fn text_element_to_draw(y_rank: usize, mood: &Mood) -> Text<(i32, i32), String> {
    let center = mood.center(y_rank);
    Text::new(
        format!("{}", mood.keyword),
        (
            center.0 + NEUTRAL_SCALEDOWN * 3,
            center.1 + NEUTRAL_HEIGHT / 3,
        ),
        ("Georiga", 15).into_font(),
    )
}

fn right_whiskers_to_draw(y_rank: usize, mood: &Mood) -> Rectangle<(i32, i32)> {
    let center = mood.center(y_rank);

    let left_bound: i32 = center.0 + i32::try_from(mood.neutral_count).unwrap() / 5;

    Rectangle::new(
        [
            (left_bound, center.1 + NEUTRAL_HEIGHT / 3),
            (
                left_bound + i32::try_from(mood.positive_count).unwrap(),
                (2 * NEUTRAL_HEIGHT / 3) + center.1,
            ),
        ],
        ShapeStyle {
            filled: true,
            stroke_width: 1,
            color: plotters::style::GREEN.to_rgba(),
        },
    )
}

fn left_whiskers_to_draw(y_rank: usize, mood: &Mood) -> Rectangle<(i32, i32)> {
    let center = mood.center(y_rank);

    Rectangle::new(
        [
            (
                center.0 - i32::try_from(mood.negative_count).unwrap(),
                center.1 + NEUTRAL_HEIGHT / 3,
            ),
            (center.0, (2 * NEUTRAL_HEIGHT / 3) + center.1),
        ],
        ShapeStyle {
            filled: true,
            stroke_width: 1,
            color: plotters::style::RED.to_rgba(),
        },
    )
}

fn start_plotting2(
    element: HtmlCanvasElement,
    json_to_draw: &str,
) -> Result<Box<dyn Fn((i32, i32)) -> Option<(u32, u32)>>, Box<dyn std::error::Error>> {
    let backend = CanvasBackend::with_canvas_object(element).unwrap();
    let root = backend.into_drawing_area();

    let mut to_draw: Vec<Mood> = serde_json::from_str(json_to_draw).unwrap();

    to_draw.sort_by(|a, b| {
        (a.positive_count + a.neutral_count + a.negative_count)
            .cmp(&(b.positive_count + b.neutral_count + b.negative_count))
    });

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .build_ranged(0u32..10u32, 0u32..10u32)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .line_style_1(&WHITE.mix(0.3))
        .x_label_offset(30)
        .y_desc("Volume of Tweets")
        .x_desc("Sentiment")
        .axis_desc_style(("Arial", 15).into_font())
        .draw()?;

    for (index, mood) in to_draw.iter().rev().enumerate() {
        root.draw(&rank_and_item_to_rectangle(index, mood))?;
    }

    for (index, mood) in to_draw.iter().rev().enumerate() {
        root.draw(&text_element_to_draw(index, mood))?;
    }

    for (index, mood) in to_draw.iter().rev().enumerate() {
        root.draw(&right_whiskers_to_draw(index, mood))?;
    }

    for (index, mood) in to_draw.iter().rev().enumerate() {
        root.draw(&left_whiskers_to_draw(index, mood))?;
    }

    Ok(Box::new(chart.into_coord_trans()))
}

#[wasm_bindgen]
pub fn draw_data(element: HtmlCanvasElement, json_to_draw: &str) -> JsValue {
    crate::make_coord_mapping_closure(start_plotting2(element, json_to_draw).ok())
}
