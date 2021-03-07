use cpal::traits::{EventLoopTrait, HostTrait};
use nannou::prelude::*;
use rand::Rng;
use std::f64::consts::PI;
use std::{
    sync::{Arc, Mutex},
    thread,
};

const MEASURE_LENGTH: usize = 16;
const NOTE_RANGE: usize = 12;

const CELL_DISPLAY_SIZE: f32 = 10.0;
const CELL_DISPLAY_PADDING: f32 = 3.0;

const SAMPLE_RATE: usize = 44_100;

const BPM: f64 = 185.0;

const CHROMATIC: [usize; NOTE_RANGE] = [440, 466, 494, 523, 554, 587, 622, 659, 698, 740, 783, 831];

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

        let _stream_id = event_loop
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
struct NoteCell(bool);

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();
    let mut blank_cells = [[NoteCell(false); MEASURE_LENGTH]; NOTE_RANGE];
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
    sample.env_attack(0.01);
    sample.env_decay(0.8);
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

    if model.active_beat == 0 {
        model.cells = advance_automata(model.cells);
    }

    model.frames_since_tick = 0.0;
}

fn view(_app: &App, model: &Model, _frame: Frame) {
    let draw = _app.draw();
    draw.background().color(PLUM);

    let total_size_x: f32 = (CELL_DISPLAY_SIZE + CELL_DISPLAY_PADDING) * MEASURE_LENGTH as f32;
    let total_size_y: f32 = (CELL_DISPLAY_SIZE + CELL_DISPLAY_PADDING) * NOTE_RANGE as f32;

    let centering_bias_x: f32 = total_size_x / 2.0;
    let centering_bias_y: f32 = total_size_y / 2.0;

    for i in 0..model.cells.len() {
        let row = model.cells[i];
        let y_pos = ((CELL_DISPLAY_SIZE + CELL_DISPLAY_PADDING) * i as f32)
            + 0.5 * CELL_DISPLAY_PADDING
            - centering_bias_y;

        for j in 0..row.len() {
            let x_pos = ((CELL_DISPLAY_SIZE + CELL_DISPLAY_PADDING) * j as f32)
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

fn compute_note_color(is_playing: bool, is_active_beat: bool) -> Rgb<u8> {
    match (is_playing, is_active_beat) {
        (false, false) => STEELBLUE,
        (false, true) => LIGHTSKYBLUE,
        (true, false) => BLACK,
        (true, true) => RED,
    }
}

fn advance_automata(
    cells: [[NoteCell; MEASURE_LENGTH]; NOTE_RANGE],
) -> [[NoteCell; MEASURE_LENGTH]; NOTE_RANGE] {
    let mut next_cells = cells;
    let mut rng = rand::thread_rng();

    for i in 0..NOTE_RANGE {
        for j in 0..MEASURE_LENGTH {
            let c_n = count_neighbors(cells, i, j);

            // 100 % prob at 2 neighbors with falloff to 0 as n-> n=0 and n>=4
            let base_probability = (0.0 - (c_n as f64 - 2.0).pow(2.0) + 4.0) / 4.0.max(0.0);

            // Rhythmatic bias to the 4th note, and a little bit of spice on the 2th notes
            let rhythmatic_scalar = nutrient_field(j as f64);

            let to_taste_bias = 0.9;

            let probability = (base_probability as f64 * to_taste_bias) * rhythmatic_scalar;

            let check: f64 = rng.gen();

            //println!("{}", count_active_notes_1_semitone_up_and_down(cells, i, j));
            if count_active_notes_1_semitone_up_and_down(cells, i, j) == 0
                && count_active_notes_3_semitones_up_and_down(cells, i, j) < 3
                && probability >= check
            {
                next_cells[i][j] = NoteCell(true);
            } else {
                next_cells[i][j] = NoteCell(false);
            }
        }
    }
    return next_cells;
}

fn count_neighbors(cells: [[NoteCell; MEASURE_LENGTH]; NOTE_RANGE], i: usize, j: usize) -> usize {
    let subset: [(usize, usize); 8] = get_moore_neighborhood(i, j);
    return subset.iter().fold(0, |acc, x| {
        return if cells[x.0][x.1].0 == true {
            acc + 1
        } else {
            acc + 0
        };
    });
}

fn get_moore_neighborhood(i: usize, j: usize) -> [(usize, usize); 8] {
    let wrap_cylinder_x = |int| int % MEASURE_LENGTH;
    let wrap_cylinder_y = |int| int % NOTE_RANGE;
    let j = j + MEASURE_LENGTH;
    let i = i + NOTE_RANGE;

    return [
        (wrap_cylinder_y(j + 1), wrap_cylinder_x(i)), // north
        (wrap_cylinder_y(j + 1), wrap_cylinder_x(i)), // south
        (wrap_cylinder_y(j), wrap_cylinder_x(i + 1)), // east
        (wrap_cylinder_y(j), wrap_cylinder_x(i - 1)), // weast
        (wrap_cylinder_y(j - 1), wrap_cylinder_x(i + 1)), // northeast
        (wrap_cylinder_y(j + 1), wrap_cylinder_x(i + 1)), // southeast
        (wrap_cylinder_y(j - 1), wrap_cylinder_x(i - 1)), // northweast
        (wrap_cylinder_y(j + 1), wrap_cylinder_x(i - 1)), // southweast
    ];
}

fn count_active_notes_3_semitones_up_and_down(
    cells: [[NoteCell; MEASURE_LENGTH]; NOTE_RANGE],
    i: usize,
    j: usize,
) -> usize {
    let wrap_cylinder_y = |int| int % NOTE_RANGE;
    let i = i + NOTE_RANGE;

    let mut count = 0;
    if cells[wrap_cylinder_y(i - 1)][j].0 == true {
        count += 1
    }
    if cells[wrap_cylinder_y(i - 2)][j].0 == true {
        count += 1
    }
    if cells[wrap_cylinder_y(i - 3)][j].0 == true {
        count += 1
    }
    if cells[wrap_cylinder_y(i + 1)][j].0 == true {
        count += 1
    }
    if cells[wrap_cylinder_y(i + 2)][j].0 == true {
        count += 1
    }
    if cells[wrap_cylinder_y(i + 3)][j].0 == true {
        count += 1
    }
    return count;
}

fn count_active_notes_1_semitone_up_and_down(
    cells: [[NoteCell; MEASURE_LENGTH]; NOTE_RANGE],
    i: usize,
    j: usize,
) -> usize {
    let wrap_cylinder_y = |int| int % NOTE_RANGE;
    let i = i + NOTE_RANGE;

    let mut count = 0;
    if cells[wrap_cylinder_y(i - 1)][j].0 == true {
        count += 1
    }
    if cells[wrap_cylinder_y(i + 1)][j].0 == true {
        count += 1
    }
    return count;
}

fn nutrient_field(x: f64) -> f64 {
    return ((PI * x).cos() + 1.0) / 4.0 + ((PI * 2.0 * x).cos() + 1.0) / 4.0;
}
