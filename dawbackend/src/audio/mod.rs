use std::time::Duration;

use dawlib::InstrumentDto;
use wavegen::{sawtooth, sine, square, wf, Precision, SampleType, Waveform};

pub mod streaming;

const DEFAULT_SAMPLE_RATE: f32 = 44100.0;

pub struct MusicBox {
    instruments: Vec<InstrumentDto>,
    playing_instruments: Vec<Box<dyn SoundNode>>,
    samples_per_beat: usize,
    current_sample: usize,
}

impl Iterator for MusicBox {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.instruments.is_empty() && self.playing_instruments.is_empty() {
            return None;
        }

        self.update_state();

        let sample = self
            .playing_instruments
            .iter_mut()
            .filter_map(|instrument| instrument.next_sample())
            .sum::<f32>();

        self.current_sample += 1;

        Some(sample)
    }
}

struct PlayedWaveform<T: SampleType, P: Precision> {
    waveform: Waveform<T, P>,
    current_sample: usize,
    sample_count: usize,
}

impl<P: Precision> SoundNode for PlayedWaveform<f32, P> {
    fn next_sample(&mut self) -> Option<f32> {
        if self.current_sample >= self.sample_count {
            return None;
        }

        let edge_smoother = 1.0f32
            .min(self.current_sample as f32 / Self::EDGE_SMOOTH)
            .min((self.sample_count - self.current_sample) as f32 / Self::EDGE_SMOOTH);

        let result = self.waveform.iter().nth(self.current_sample).unwrap() * edge_smoother * 0.2;
        self.current_sample += 1;
        Some(result)
    }

    fn ended(&self) -> bool {
        self.current_sample >= self.sample_count
    }
}

struct Kick {
    sample_rate: usize,
    current_sample: usize,
}

impl Kick {
    fn new(sample_rate: usize) -> Self {
        Self {
            sample_rate,
            current_sample: 0,
        }
    }
}

trait SoundNode: Send {
    fn next_sample(&mut self) -> Option<f32>;
    fn ended(&self) -> bool;
}

struct CompoundSoundNode<T: SoundNode> {
    nodes: Vec<T>,
}

impl<T: SoundNode> SoundNode for CompoundSoundNode<T> {
    fn next_sample(&mut self) -> Option<f32> {
        self.nodes.iter_mut().map(|node| node.next_sample()).sum()
    }

    fn ended(&self) -> bool {
        self.nodes.iter().all(|node| node.ended())
    }
}

struct GainNode<T: SoundNode> {
    node: T,
    value: f32,
}

impl<T: SoundNode> GainNode<T> {
    fn new(node: T, value: f32) -> Self {
        Self { node, value }
    }
}

impl<T: SoundNode> SoundNode for GainNode<T> {
    fn next_sample(&mut self) -> Option<f32> {
        self.node.next_sample().map(|sample| sample * self.value)
    }

    fn ended(&self) -> bool {
        self.node.ended()
    }
}

struct SimpleDelayNode<T: SoundNode> {
    node: T,
    current_sample: usize,
    sample_delay: usize,
    buffer: Vec<f32>,
    remaining_samples: usize,
}

struct DelayReverb<T: SoundNode> {
    node: SimpleDelayNode<SimpleDelayNode<SimpleDelayNode<T>>>,
}

impl<T: SoundNode> DelayReverb<T> {
    fn new(node: T, sample_rate: usize) -> Self {
        Self {
            node: SimpleDelayNode::new(
                SimpleDelayNode::new(
                    SimpleDelayNode::new(node, sample_rate, Duration::from_secs_f32(0.1)),
                    sample_rate,
                    Duration::from_secs_f32(0.2),
                ),
                sample_rate,
                Duration::from_secs_f32(0.3)
            ),
        }
    }
}

impl<T: SoundNode> SoundNode for DelayReverb<T> {
    fn next_sample(&mut self) -> Option<f32> {
        self.node.next_sample()
    }

    fn ended(&self) -> bool {
        self.node.ended()
    }
}

impl<T: SoundNode> SimpleDelayNode<T> {
    fn new(node: T, sample_rate: usize, delay: Duration) -> Self {
        let sample_delay = (sample_rate as f32 * delay.as_secs_f32()) as usize;
        Self {
            node,
            current_sample: 0,
            sample_delay,
            buffer: vec![0.0; sample_delay],
            remaining_samples: 0,
        }
    }
}

impl<T: SoundNode> SoundNode for SimpleDelayNode<T> {
    fn next_sample(&mut self) -> Option<f32> {
        let result = self.node.next_sample();
        let buffer_position = self.current_sample % self.sample_delay;

        let result = if self.current_sample < self.sample_delay {
            if let Some(sample) = &result {
                self.buffer[buffer_position] = *sample;
            }
            self.remaining_samples += 1;
            result
        } else {
            let delayed_sample = self.buffer[buffer_position] * 0.4;
            if let Some(sample) = &result {
                self.buffer[buffer_position] = *sample;
                Some(sample + delayed_sample)
            } else if self.remaining_samples > 0 {
                self.remaining_samples -= 1;
                Some(delayed_sample)
            } else {
                None
            }
        };

        self.current_sample += 1;
        result
    }

    fn ended(&self) -> bool {
        self.node.ended() && self.remaining_samples == 0
    }
}

impl SoundNode for Kick {
    fn next_sample(&mut self) -> Option<f32> {
        if self.ended() {
            return None;
        }

        let time = self.current_sample as f32 / self.sample_rate as f32;

        let result = f32::sin((2000.0 * f32::exp(-15.0 * time)) * time);
        self.current_sample += 1;
        Some(result)
    }

    fn ended(&self) -> bool {
        self.current_sample >= self.sample_rate
    }
}

impl<T: SampleType + std::ops::Mul<f32, Output = T>, P: Precision> PlayedWaveform<T, P> {
    const EDGE_SMOOTH: f32 = 400.0;
    pub fn new(waveform: Waveform<T, P>, sample_count: usize) -> PlayedWaveform<T, P> {
        PlayedWaveform {
            waveform,
            current_sample: 0,
            sample_count,
        }
    }
}

impl MusicBox {
    pub fn new(tempo: usize, instruments: Vec<InstrumentDto>) -> Self {
        let modifier = tempo as f32 / 60.0;
        let samples_per_beat = (DEFAULT_SAMPLE_RATE / modifier) as usize;
        Self {
            instruments,
            playing_instruments: vec![],
            current_sample: 0,
            samples_per_beat,
        }
    }

    pub fn chunk(&mut self, size: usize) -> Result<Vec<f32>, Vec<f32>> {
        let mut output = Vec::with_capacity(size);

        for _ in 0..size {
            if let Some(sample) = self.next() {
                output.push(sample);
            } else {
                return Err(output);
            }
        }

        Ok(output)
    }

    fn update_state(&mut self) {
        let current_beat = self.current_sample / self.samples_per_beat;

        if self.current_sample % self.samples_per_beat != 0 {
            return;
        }

        self.playing_instruments.retain(|sound| !sound.ended());

        let mut new_instruments = self
            .instruments
            .iter()
            .filter_map(|instrument| {
                let gain = (instrument.gain + 30.0) / 30.0;
                let notes = instrument.notes.get(&current_beat)?;
                Some(match instrument.name.as_str() {
                    "sawtooth" => notes
                        .iter()
                        .map(|note| {
                            boxed(GainNode::new(
                                    PlayedWaveform::new(
                                        wf!(f32, 44100., sawtooth!(note.frequency())),
                                        self.samples_per_beat,
                                    ),
                                gain,
                            ))
                        })
                        .collect::<Vec<_>>(),
                    "sine" => notes
                        .iter()
                        .map(|note| {
                            boxed(GainNode::new(
                                DelayReverb::new(
                                PlayedWaveform::new(
                                    wf!(f32, 44100., sine!(note.frequency())),
                                    self.samples_per_beat,
                                ),
                                44100), gain,
                            ))
                        })
                        .collect::<Vec<_>>(),
                    "square" => notes
                        .iter()
                        .map(|note| {
                            boxed(GainNode::new(
                                DelayReverb::new(
                                PlayedWaveform::new(
                                    wf!(f32, 44100., square!(note.frequency())),
                                    self.samples_per_beat,
                                ),
                                44100), gain,
                            ))
                        })
                        .collect::<Vec<_>>(),
                    "kick" => notes
                        .iter()
                        .map(|_| boxed(GainNode::new(Kick::new(44100), gain)))
                        .collect::<Vec<_>>(),
                    _ => todo!(),
                })
            })
            .flatten()
            .collect::<Vec<_>>();

        self.playing_instruments.append(&mut new_instruments);

        self.instruments
            .iter_mut()
            .for_each(|instrument| instrument.notes.retain(|beat, _| *beat > current_beat));

        self.instruments
            .retain(|instrument| !instrument.notes.is_empty());
    }
}

fn boxed<T: SoundNode + 'static>(sound: T) -> Box<dyn SoundNode> {
    Box::new(sound)
}
