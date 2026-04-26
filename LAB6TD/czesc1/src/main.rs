use plotters::prelude::*;
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

    let b_limit = 10;
    let bits_to_plot = &bit_stream[0..usize::min(b_limit, bit_stream.len())];

    let mut t_vals = Vec::new();
    let mut za_vals = Vec::new();
    let mut zf_vals = Vec::new();
    let mut zp_vals = Vec::new();

    let mut current_t = 0.0;

    for &b in bits_to_plot {
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

    let root = BitMapBackend::new("przebiegi_czasowe.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let areas = root.split_evenly((3, 1));

    let za_series: Vec<(f64, f64)> = t_vals
        .iter()
        .cloned()
        .zip(za_vals.iter().cloned())
        .collect();
    let zf_series: Vec<(f64, f64)> = t_vals
        .iter()
        .cloned()
        .zip(zf_vals.iter().cloned())
        .collect();
    let zp_series: Vec<(f64, f64)> = t_vals
        .iter()
        .cloned()
        .zip(zp_vals.iter().cloned())
        .collect();

    let mut chart_ask = ChartBuilder::on(&areas[0])
        .margin(10)
        .caption(
            "ASK - Kluczowanie z przesuwem amplitudy",
            ("sans-serif", 20),
        )
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..current_t, -1.2f64..1.2f64)?;
    chart_ask
        .configure_mesh()
        .x_desc("Czas [s]")
        .y_desc("Amplituda")
        .draw()?;
    chart_ask.draw_series(LineSeries::new(za_series, &BLUE))?;

    let mut chart_fsk = ChartBuilder::on(&areas[1])
        .margin(10)
        .caption(
            "FSK - Kluczowanie z przesuwem czestotliwosci",
            ("sans-serif", 20),
        )
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..current_t, -1.2f64..1.2f64)?;
    chart_fsk
        .configure_mesh()
        .x_desc("Czas [s]")
        .y_desc("Amplituda")
        .draw()?;
    chart_fsk.draw_series(LineSeries::new(zf_series, &RED))?;

    let mut chart_psk = ChartBuilder::on(&areas[2])
        .margin(10)
        .caption("PSK - Kluczowanie z przesuwem fazy", ("sans-serif", 20))
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..current_t, -1.2f64..1.2f64)?;
    chart_psk
        .configure_mesh()
        .x_desc("Czas [s]")
        .y_desc("Amplituda")
        .draw()?;
    chart_psk.draw_series(LineSeries::new(zp_series, &GREEN))?;

    root.present()?;
    Ok(())
}
