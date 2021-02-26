use nannou::prelude::*;

const MEASURE_LENGTH : usize = 16;
const NOTE_RANGE : usize = 12;

const CELL_DISPLAY_SIZE : f32 = 10.0;
const CELL_DISPLAY_PADDING : f32 = 3.0;

struct Model {
    _window: window::Id,
    cells: [[NoteCell; MEASURE_LENGTH]; NOTE_RANGE],
    active_note: usize
}

#[derive(Copy, Clone, Debug)]
struct NoteCell ( bool );

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}

fn model(_app: &App) -> Model {
    let _window = _app.new_window().view(view).build().unwrap();
    let blank_cells = [[NoteCell(false) ; MEASURE_LENGTH] ; NOTE_RANGE];
    Model { _window, cells: blank_cells, active_note: 0}
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.active_note = (model.active_note + 1) % 16;
}

fn view(_app: &App, model: &Model, _frame: Frame) {
    let draw = _app.draw();
    draw.background().color(PLUM);

    let total_size_x : f32 = (CELL_DISPLAY_SIZE + CELL_DISPLAY_PADDING) * MEASURE_LENGTH as f32;
    let total_size_y : f32 = (CELL_DISPLAY_SIZE + CELL_DISPLAY_PADDING) * NOTE_RANGE as f32;

    let centering_bias_x : f32 = total_size_x / 2.0;
    let centering_bias_y : f32 = total_size_y / 2.0;

    for i in 0..model.cells.len() {
        let row = model.cells[i];
        let y_pos = ((CELL_DISPLAY_SIZE + CELL_DISPLAY_PADDING)
            * i as f32)
            + 0.5 * CELL_DISPLAY_PADDING
            - centering_bias_y;

        for j in 0..row.len() {
            let x_pos = ((CELL_DISPLAY_SIZE + CELL_DISPLAY_PADDING)
                * j as f32)
                + 0.5 * CELL_DISPLAY_PADDING
                - centering_bias_x;

            // TODO fn compute_note_color();
            let note_color = if j == model.active_note {LIGHTSKYBLUE} else {STEELBLUE};

            draw.rect()
                .w_h(CELL_DISPLAY_SIZE, CELL_DISPLAY_SIZE)
                .x_y(x_pos, y_pos)
                .color(note_color);
        }
    }
    draw.to_frame(_app, &_frame).unwrap();
}
