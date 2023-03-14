use dawlib::InstrumentDto;
use wavegen::{Waveform, wf, sawtooth, square, sine, Precision, SampleType};

pub mod streaming;

const DEFAULT_SAMPLE_RATE: f32 = 44100.0;

pub struct MusicBox {
    instruments: Vec<InstrumentDto>,
    playing_instruments: Vec<PlayedWaveform<f32, f32>>,
    current_sample: usize
}

impl Iterator for MusicBox {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.instruments.is_empty() && self.playing_instruments.is_empty() {
            return None;
        }

        self.update_state();
        
        let current_instrument_count = self.playing_instruments.len() as f32;

        let sample = self.playing_instruments.iter()
            .map(|instrument| instrument.nth(self.current_sample) / current_instrument_count)
            .sum::<f32>();

        self.current_sample += 1;

        Some(sample)
    }
}

struct PlayedWaveform<T: SampleType, P: Precision> {
    waveform: Waveform<T, P>,
    start_offset: usize,
    end: usize
}

impl <T: SampleType + std::ops::Mul<f32, Output = T>, P: Precision> PlayedWaveform<T, P> {
    const EDGE_SMOOTH: f32 = 400.0;
    pub fn new(waveform: Waveform<T, P>, start_offset: usize, end: usize) -> PlayedWaveform<T, P> {
        PlayedWaveform {
            waveform,
            start_offset,
            end
        }
    }

    pub fn nth(&self, n: usize) -> T {
        let local_offest = n - self.start_offset;
        let edge_smoother = 1.0f32.min(local_offest as f32 / Self::EDGE_SMOOTH).min((self.end - n) as f32 / Self::EDGE_SMOOTH);
        self.waveform.iter().nth(local_offest).unwrap() * edge_smoother
    }
}

impl MusicBox {
    pub fn new(instruments: Vec<InstrumentDto>) -> Self { 
        Self { 
            instruments, 
            playing_instruments: vec![], 
            current_sample: 0 
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
        let current_second = self.current_sample / DEFAULT_SAMPLE_RATE as usize;

        if self.current_sample % DEFAULT_SAMPLE_RATE as usize != 0 {
            return;
        }

        self.playing_instruments.retain(|playing| playing.end > self.current_sample);

        let mut new_instruments = self.instruments.iter()
            .filter_map(|instrument| {
                let notes = instrument.notes.get(&current_second)?;
                Some(match instrument.name.as_str() {
                    "sawtooth" => notes.iter()
                        .map(|note| wf!(f32, 44100., sawtooth!(note.frequency(), 0.1)))
                        .collect::<Vec<_>>(),
                    "sine" => notes.iter()
                        .map(|note| wf!(f32, 44100., sine!(note.frequency())))
                        .collect::<Vec<_>>(),
                    "square" => notes.iter()
                        .map(|note| wf!(f32, 44100., square!(note.frequency(), 0.1)))
                        .collect::<Vec<_>>(),
                    _ => todo!()
                })
            })
            .flatten()
            .map(|waveform| PlayedWaveform::new(waveform, self.current_sample, (current_second + 1) * DEFAULT_SAMPLE_RATE as usize))
            .collect::<Vec<_>>();

        self.playing_instruments.append(&mut new_instruments);

        self.instruments.iter_mut()
            .for_each(|instrument| {
                instrument.notes.retain(|second, _| *second > current_second)
            });

        self.instruments.retain(|instrument| !instrument.notes.is_empty());
    }
}
