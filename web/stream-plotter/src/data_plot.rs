use plotters::element::Rectangle;
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

fn rank_and_item_to_rectangle(
    y_rank: usize,
    mood: &Mood,
) -> EmptyElement<(i32, i32), CanvasBackend> {
    let y: i32 = i32::try_from(y_rank).unwrap();

    let y_offset = y * 20;
    Rectangle::new(
        [
            (0, 0 + y_offset),
            (
                i32::try_from(mood.neutral_count).unwrap() / 5,
                15 + y_offset,
            ),
        ],
        ShapeStyle {
            filled: true,
            stroke_width: 1,
            color: plotters::style::GREEN.to_rgba(),
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
        .caption("Histogram Test", ("Arial", 50.0).into_font())
        .build_ranged(0u32..10u32, 0u32..10u32)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .line_style_1(&WHITE.mix(0.3))
        .x_label_offset(30)
        .y_desc("Volume")
        .x_desc("Sentiment")
        .axis_desc_style(("Arial", 15).into_font())
        .draw()?;

    for (index, mood) in to_draw.iter().rev().enumerate() {
        root.draw(&rank_and_item_to_rectangle(index, mood))?;
    }

    Ok(Box::new(chart.into_coord_trans()))
}

#[wasm_bindgen]
pub fn draw_data(element: HtmlCanvasElement, json_to_draw: &str) -> JsValue {
    crate::make_coord_mapping_closure(start_plotting2(element, json_to_draw).ok())
}
