use nannou::prelude::*;
use std::f64::consts::PI;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use cpal::traits::{EventLoopTrait, HostTrait};

const MEASURE_LENGTH : usize = 16;
const NOTE_RANGE : usize = 12;

const CELL_DISPLAY_SIZE : f32 = 10.0;
const CELL_DISPLAY_PADDING : f32 = 3.0;

const SAMPLE_RATE: usize = 44_100;

const BPM : f64 = 135.0;

const CHROMATIC : [usize; NOTE_RANGE] = [440, 466, 494, 523, 554, 587, 622, 659, 698, 740, 783, 831];

struct Model {
    _window: window::Id,
    cells: [[NoteCell; MEASURE_LENGTH]; NOTE_RANGE],
    active_beat: usize,
    cooldown: f64,
    frames_since_tick: f64,
    audio: Audio,
}

#[derive(Default)]
pub struct Audio {
    mixer: Arc<Mutex<usfx::Mixer>>,
}

impl Audio {
    // Initialize a new audio object
    pub fn new() -> Self {
        Self {
            mixer: Arc::new(Mutex::new(usfx::Mixer::new(SAMPLE_RATE))),
        }
    }

    pub fn play(&mut self, sample: usfx::Sample) {
        // Add the sample to the mixer
        self.mixer.lock().unwrap().play(sample);
    }

    pub fn run(&mut self) {
        let mixer = self.mixer.clone();
        // Set up the audio system
        let host = cpal::default_host();
        let event_loop = host.event_loop();

        let device = host
            .default_output_device()
            .expect("no output device available");

        let format = cpal::Format {
            channels: 1,
            sample_rate: cpal::SampleRate(SAMPLE_RATE as u32),
            data_type: cpal::SampleFormat::F32,
        };

        let stream_id = event_loop
            .build_output_stream(&device, &format)
            .expect("could not play stream");

        thread::spawn(move || {
            event_loop.run(move |stream_id, stream_result| {
                let stream_data = match stream_result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error ocurred on stream {:?}: {}", stream_id, err);
                        return;
                    }
                };

                match stream_data {
                    cpal::StreamData::Output {
                        buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                    } => mixer.lock().unwrap().generate(&mut buffer),
                    _ => panic!("output type buffer can not be used"),
                }
            });
        });
    }
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
    let mut blank_cells = [[NoteCell(false) ; MEASURE_LENGTH] ; NOTE_RANGE];
    blank_cells[0][0].0 = true;
    blank_cells[2][0].0 = true;
    blank_cells[3][4].0 = true;
    blank_cells[7][6].0 = true;
    blank_cells[6][8].0 = true;

    let beats_per_second = BPM / 60.0;
    let seconds_per_frame = 1.0 / 60.0; // TODO: Figure out why  app.fps(); is completely broken
    let frames_per_beat = 1.0 / (beats_per_second * seconds_per_frame as f64);

    let mut audio = Audio::new();

    // Spawn a background thread where an audio device is opened with cpal
    audio.run();

    Model {
        _window,
        cells: blank_cells,
        active_beat: 0,
        cooldown: frames_per_beat,
        frames_since_tick: 0.0,
        audio,
    }
}


fn update(_app: &App, model: &mut Model, _update: Update) {

    if model.frames_since_tick < model.cooldown {
        model.frames_since_tick += 1.0;
        return;
    }

    let mut sample = usfx::Sample::default();
    sample.osc_type(usfx::OscillatorType::Sine);
    sample.env_attack(0.1);
    sample.env_decay(0.1);
    sample.env_sustain(0.5);
    sample.env_release(0.5);
    sample.dis_crunch(0.2);

    for i in 0..NOTE_RANGE {
        let c = model.cells[i];
        if c[model.active_beat].0 == true {
            sample.osc_frequency(CHROMATIC[i]);
            // Play a low sample with a square wave
            model.audio.play(sample);
        }
    }

    model.active_beat = (model.active_beat + 1) % 16;
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
            let note_color = compute_note_color(model.cells[i][j].0, j == model.active_beat);

            draw.rect()
                .w_h(CELL_DISPLAY_SIZE, CELL_DISPLAY_SIZE)
                .x_y(x_pos, y_pos)
                .color(note_color);
        }
    }
    draw.to_frame(_app, &_frame).unwrap();
}

fn compute_note_color(is_playing: bool, is_active_beat : bool) -> Rgb<u8>{
    match (is_playing, is_active_beat) {
        (false, false) => STEELBLUE,
        (false, true) => LIGHTSKYBLUE,
        (true, false) => BLACK,
        (true, true) => RED,
    }
}
