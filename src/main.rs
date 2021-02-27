use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use std::f64::consts::PI;

const MEASURE_LENGTH : usize = 16;
const NOTE_RANGE : usize = 12;

const CELL_DISPLAY_SIZE : f32 = 10.0;
const CELL_DISPLAY_PADDING : f32 = 3.0;

const BPM : f64 = 135.0;

struct Model {
    _window: window::Id,
    cells: [[NoteCell; MEASURE_LENGTH]; NOTE_RANGE],
    active_note: usize,
    cooldown: f64,
    frames_since_tick: f64,
    stream: audio::Stream<Audio>,
}

struct Audio {
    phase: f64,
    hz: f64,
}

#[derive(Copy, Clone, Debug)]
struct NoteCell ( bool );

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();
    let blank_cells = [[NoteCell(false) ; MEASURE_LENGTH] ; NOTE_RANGE];

    let beats_per_second = BPM / 60.0;
    let seconds_per_frame = 1.0 / 60.0; // TODO: Figure out why  app.fps(); is completely broken
    let frames_per_beat = 1.0 / (beats_per_second * seconds_per_frame as f64);

    // Initialise the audio API so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Initialise the state that we want to live on the audio thread.
    let model = Audio {
        phase: 0.0,
        hz: 440.0,
    };

    let stream = audio_host
        .new_output_stream(model)
        .render(audio)
        .build()
        .unwrap();

    Model {
        _window,
        cells: blank_cells,
        active_note: 0,
        cooldown: frames_per_beat,
        frames_since_tick: 0.0,
        stream
    }
}

// A function that renders the given `Audio` to the given `Buffer`.
// In this case we play a simple sine wave at the audio's current frequency in `hz`.
fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let sample_rate = buffer.sample_rate() as f64;
    let volume = 0.5;
    for frame in buffer.frames_mut() {
        let sine_amp = (2.0 * PI * audio.phase).sin() as f32;
        audio.phase += audio.hz / sample_rate;
        audio.phase %= sample_rate;
        for channel in frame {
            *channel = sine_amp * volume;
        }
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if !model.stream.is_playing() {
        model.stream.play().unwrap();
    }

    if model.frames_since_tick < model.cooldown {
        model.frames_since_tick += 1.0;
        return;
    }

    model
        .stream
        .send(|audio| {
            audio.hz += 10.0;
        })
        .unwrap();

    model.active_note = (model.active_note + 1) % 16;
    model.frames_since_tick = 0.0;
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
