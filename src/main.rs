use nannou::prelude::*;

const MEASURE_LENGTH : usize = 16;
const NOTE_RANGE : usize = 12;

struct Model {
    _window: window::Id,
    cells: Vec<Vec<NoteCell>>
}

#[derive(Clone, Debug)]
struct NoteCell {
    position_x: f32,
    position_y: f32,
    velocity: bool
}

fn initial_cells() -> Vec<Vec<NoteCell>> {
    let rows = 0..NOTE_RANGE;
    let cols = 0..MEASURE_LENGTH;
    let blank_cell = NoteCell {
        position_x: 0.0,
        position_y: 0.0,
        velocity: false
    };
    let one_row : Vec<NoteCell> = Vec::new(); // TODO : not blank
    return cols.map(|pos| one_row.clone()).collect();
}

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}

fn model(_app: &App) -> Model {
    let _window = _app.new_window().view(view).build().unwrap();
    let blank_cells : Vec<Vec<NoteCell>> = initial_cells();
    Model { _window, cells: blank_cells}
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(_app: &App, _model: &Model, _frame: Frame) {
    let draw = _app.draw();
    draw.background().color(PLUM);
    draw.rect()
        .x_y(10.0, 10.0)
        .w_h(100.0, 100.0)
        .color(STEELBLUE);
    draw.to_frame(_app, &_frame).unwrap();
}
