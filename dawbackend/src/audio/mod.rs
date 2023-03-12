use dawlib::InstrumentDto;
use wavegen::{Waveform, wf, WaveformIterator, sawtooth, square, sine};

pub mod streaming;


pub struct MusicBox;

impl MusicBox {
    pub fn generate(instruments: &Vec<InstrumentDto>) -> Vec<Vec<f32>> {
        let mut sawtooth = vec![];
        let mut sine = vec![];
        let mut square = vec![];
        let end = instruments.iter().filter_map(|instrument| {
            instrument.notes.keys().max().copied()
        }).max();

        if end.is_none() {
            return vec![];
        }

        let end = end.unwrap();

        if let Some(sawtooth_instrument) = instruments.iter().find(|instrument| instrument.name == "sawtooth")  {
            for i in 0..=end {
                if let Some(notes) = sawtooth_instrument.notes.get(&i) {
                    let waveforms = notes.iter()
                    .map(|key| wf!(f32, 44100., sawtooth!(key.frequency())))
                    .collect::<Vec<Waveform<f32>>>();

                    let mut waveforms_iterators = waveforms.iter()
                    .map(|waveform| waveform.iter())
                    .collect::<Vec<WaveformIterator<f32, _>>>();

                    let divisor = waveforms.len() as f32;
                    let second = (0..44100).map(|_| {
                        waveforms_iterators.iter_mut()
                        .map(|waveform| waveform.next().unwrap() / divisor)
                        .sum()
                    }).collect::<Vec<f32>>();
 
                    sawtooth.push(Some(second));
                } else {
                    sawtooth.push(None);
                }
            }
        } else {
            for _ in 0..=end {
                sawtooth.push(None);
            }
        }

        if let Some(sawtooth_instrument) = instruments.iter().find(|instrument| instrument.name == "sine")  {
            for i in 0..=end {
                if let Some(notes) = sawtooth_instrument.notes.get(&i) {
                    let waveforms = notes.iter()
                    .map(|key| wf!(f32, 44100., sine!(key.frequency())))
                    .collect::<Vec<Waveform<f32>>>();

                    let mut waveforms_iterators = waveforms.iter()
                    .map(|waveform| waveform.iter())
                    .collect::<Vec<WaveformIterator<f32, _>>>();

                    let divisor = waveforms.len() as f32;
                    let second = (0..44100).map(|_| {
                        waveforms_iterators.iter_mut()
                        .map(|waveform| waveform.next().unwrap() / divisor)
                        .sum()
                    }).collect::<Vec<f32>>();
 
                    sine.push(Some(second));
                } else {
                    sine.push(None);
                }
            } 
        } else {
            for _ in 0..=end {
                sine.push(None);
            }
        }

        if let Some(sawtooth_instrument) = instruments.iter().find(|instrument| instrument.name == "square")  {
            for i in 0..=end {
                if let Some(notes) = sawtooth_instrument.notes.get(&i) {
                    let waveforms = notes.iter()
                    .map(|key| wf!(f32, 44100., square!(key.frequency())))
                    .collect::<Vec<Waveform<f32>>>();

                    let mut waveforms_iterators = waveforms.iter()
                    .map(|waveform| waveform.iter())
                    .collect::<Vec<WaveformIterator<f32, _>>>();

                    let divisor = waveforms.len() as f32;
                    let second = (0..44100).map(|_| {
                        waveforms_iterators.iter_mut()
                        .map(|waveform| waveform.next().unwrap() / divisor)
                        .sum()
                    }).collect::<Vec<f32>>();
 
                    square.push(Some(second));
                } else {
                    square.push(None);
                }
            }
        } else {
            for _ in 0..=end {
                square.push(None);
            }
        }
        
        let zipped_fragments = sawtooth.into_iter().zip(sine.into_iter()).zip(square.into_iter())
        .map(|((saw, sine), square)| {
            vec![saw, sine, square]
        }).collect::<Vec<_>>();

        zipped_fragments.into_iter().map(|fragment|{
            let mut fragment_chunks = fragment.into_iter()
            .flatten()
            .collect::<Vec<_>>();

            if fragment_chunks.is_empty() {
                vec![0.0f32; 88200]
            } else {
                let count = fragment_chunks.len() as f32;
                let mut result = fragment_chunks.pop().unwrap();
                for item in result.iter_mut() {
                    *item /= count;
                }
                for other_chunk in fragment_chunks {
                    for (index, value) in other_chunk.iter().enumerate() {
                        result[index] += *value / count;
                    }
                }

                for i in 0..400 {
                    result[i] *= i as f32 / 400.;
                    result[44100 - i - 1] *= i as f32 / 400.;
                }

                result.append(&mut result.clone());
                result
            }
        }).collect::<Vec<Vec<f32>>>()
    }
}
