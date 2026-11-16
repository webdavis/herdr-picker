use pwe_domain::Size;
use serde_json::Value;

const REX_FIRST_DRAW: Size = Size {
    columns: 80,
    rows: 24,
};
const WHOLE: (f64, f64) = (1.0, 1.0);

pub(super) fn window_cells(facts: &Value, block: &str) -> Size {
    let cells = |pointer, otherwise| {
        facts
            .pointer(pointer)
            .and_then(Value::as_u64)
            .and_then(|cells| usize::try_from(cells).ok())
            .filter(|cells| *cells > 0)
            .unwrap_or(otherwise)
    };
    let (width, height) = share_of_window(facts, block).unwrap_or(WHOLE);
    Size {
        columns: scaled(cells("/size/columns", REX_FIRST_DRAW.columns), width),
        rows: scaled(cells("/size/rows", REX_FIRST_DRAW.rows), height),
    }
}

fn share_of_window(facts: &Value, block: &str) -> Option<(f64, f64)> {
    facts
        .pointer("/view/windows")?
        .as_array()?
        .iter()
        .filter_map(|window| window["layers"].as_array())
        .flatten()
        .find_map(|layer| {
            let (width, height) = share_in_layout(&layer["layout"], block)?;
            let (layer_width, layer_height) = width_and_height(&layer["bounds"]).unwrap_or(WHOLE);
            Some((width * layer_width, height * layer_height))
        })
}

fn share_in_layout(layout: &Value, block: &str) -> Option<(f64, f64)> {
    if layout["block_id"] == block {
        return width_and_height(&layout["rect"]);
    }
    let split = layout.get("split")?;
    share_in_layout(&split["before"], block).or_else(|| share_in_layout(&split["after"], block))
}

fn width_and_height(rectangle: &Value) -> Option<(f64, f64)> {
    let width = rectangle.get(2)?.as_f64()?;
    let height = rectangle.get(3)?.as_f64()?;
    (width > 0.0 && height > 0.0).then_some((width, height))
}

fn scaled(cells: usize, share: f64) -> usize {
    ((cells as f64 / share).round() as usize).max(1)
}
