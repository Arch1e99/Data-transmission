use plotters::prelude::*;
use rustfft::{num_complex::Complex, FftPlanner};
use std::f64::consts::PI;

fn string_to_bits(text: &str) -> Vec<u8> {
    let mut bits = Vec::new();
    for ch in text.chars() {
        let ascii_val = ch as u8;
        if (32..=127).contains(&ascii_val) {
            for i in (0..7).rev() {
                bits.push((ascii_val >> i) & 1);
            }
        }
    }
    bits
}

fn calculate_spectrum_db(signal: &[f64]) -> Vec<f64> {
    let n = signal.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);

    let mut buffer: Vec<Complex<f64>> = signal
        .iter()
        .map(|&val| Complex { re: val, im: 0.0 })
        .collect();

    fft.process(&mut buffer);

    let mut spectrum_db = Vec::with_capacity(n / 2);
    for val in buffer.iter().take(n / 2) {
        let magnitude = val.norm() / (n as f64);
        let db = if magnitude > 1e-10 {
            20.0 * magnitude.log10()
        } else {
            -200.0
        };
        spectrum_db.push(db);
    }
    spectrum_db
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = "Test";
    let bit_stream = string_to_bits(text);

    let w: f64 = 2.0;
    let tb: f64 = 0.1;
    let fn_freq = w / tb;

    let fn1 = (w + 1.0) / tb;
    let fn2 = (w + 2.0) / tb;

    let a1: f64 = 0.5;
    let a2: f64 = 1.0;

    let fs: f64 = 1000.0;
    let dt = 1.0 / fs;

    let mut t_vals = Vec::new();
    let mut za_vals = Vec::new();
    let mut zf_vals = Vec::new();
    let mut zp_vals = Vec::new();

    let mut current_t = 0.0;

    for &b in &bit_stream {
        let samples_per_bit = (tb * fs).round() as usize;

        for _ in 0..samples_per_bit {
            t_vals.push(current_t);

            let amp = if b == 0 { a1 } else { a2 };
            za_vals.push(amp * (2.0 * PI * fn_freq * current_t).sin());

            let freq = if b == 0 { fn1 } else { fn2 };
            zf_vals.push((2.0 * PI * freq * current_t).sin());

            let phase = if b == 0 { 0.0 } else { PI };
            zp_vals.push((2.0 * PI * fn_freq * current_t + phase).sin());

            current_t += dt;
        }
    }

    let ma_db = calculate_spectrum_db(&za_vals);
    let mf_db = calculate_spectrum_db(&zf_vals);
    let mp_db = calculate_spectrum_db(&zp_vals);

    let n = za_vals.len();
    let mut freqs = Vec::with_capacity(n / 2);
    for i in 0..(n / 2) {
        freqs.push((i as f64) * fs / (n as f64));
    }

    let root = BitMapBackend::new("widma_amplitudowe.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let areas = root.split_evenly((3, 1));

    let ma_series: Vec<(f64, f64)> = freqs.iter().cloned().zip(ma_db.iter().cloned()).collect();
    let mf_series: Vec<(f64, f64)> = freqs.iter().cloned().zip(mf_db.iter().cloned()).collect();
    let mp_series: Vec<(f64, f64)> = freqs.iter().cloned().zip(mp_db.iter().cloned()).collect();

    let max_freq = 100.0;

    let mut chart_ma = ChartBuilder::on(&areas[0])
        .margin(10)
        .caption("Widmo amplitudowe ASK [dB]", ("sans-serif", 20))
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..max_freq, -100.0f64..0.0f64)?;
    chart_ma
        .configure_mesh()
        .x_desc("Czestotliwosc [Hz]")
        .y_desc("Amplituda [dB]")
        .draw()?;
    chart_ma.draw_series(LineSeries::new(ma_series, &BLUE))?;

    let mut chart_mf = ChartBuilder::on(&areas[1])
        .margin(10)
        .caption("Widmo amplitudowe FSK [dB]", ("sans-serif", 20))
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..max_freq, -100.0f64..0.0f64)?;
    chart_mf
        .configure_mesh()
        .x_desc("Czestotliwosc [Hz]")
        .y_desc("Amplituda [dB]")
        .draw()?;
    chart_mf.draw_series(LineSeries::new(mf_series, &RED))?;

    let mut chart_mp = ChartBuilder::on(&areas[2])
        .margin(10)
        .caption("Widmo amplitudowe PSK [dB]", ("sans-serif", 20))
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..max_freq, -100.0f64..0.0f64)?;
    chart_mp
        .configure_mesh()
        .x_desc("Czestotliwosc [Hz]")
        .y_desc("Amplituda [dB]")
        .draw()?;
    chart_mp.draw_series(LineSeries::new(mp_series, &GREEN))?;

    root.present()?;
    Ok(())
}
