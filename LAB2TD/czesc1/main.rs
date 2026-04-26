use num_complex::Complex64;
use plotters::prelude::*;
use std::f64::consts::PI;

fn dft(x: &[f64]) -> Vec<Complex64> {
    let n = x.len();
    let mut x_k = Vec::with_capacity(n);

    for k in 0..n {
        let mut sum = Complex64::new(0.0, 0.0);
        for n_idx in 0..n {
            let phi = -2.0 * PI * (k as f64) * (n_idx as f64) / (n as f64);
            let w = Complex64::new(phi.cos(), phi.sin());
            sum += Complex64::new(x[n_idx], 0.0) * w;
        }
        x_k.push(sum);
    }
    x_k
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fs = 1000.0;
    let f = 50.0;
    let a = 1.0;
    let n = 1000;

    let mut t_vals = Vec::new();
    let mut x_vals = Vec::new();

    for i in 0..n {
        let t = i as f64 / fs;
        t_vals.push(t);
        x_vals.push(a * (2.0 * PI * f * t).sin());
    }

    // a)
    let x_k = dft(&x_vals);

let mut f_k = Vec::new();
    let mut m_k = Vec::new();

    // b)
    for k in 0..(n / 2) {
        let freq = k as f64 * (fs / n as f64);
        f_k.push(freq);

        // c)
        let magnitude = (x_k[k].re.powi(2) + x_k[k].im.powi(2)).sqrt();
        m_k.push(magnitude);
    }

    draw_time_plot(&t_vals, &x_vals)?;
    draw_spectrum_plot(&f_k, &m_k)?;
    Ok(())
}

fn draw_time_plot(t: &[f64], x: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("sygnal_czas.png", (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            "przebieg w dziedzinie czasu x(t)",
            ("sans-serif", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..0.1, -1.5f64..1.5f64)?;

    chart
        .configure_mesh()
        .x_desc("czas [s]")
        .y_desc("amplituda")
        .draw()?;
    let series = t
        .iter()
        .zip(x.iter())
        .map(|(&t_val, &x_val)| (t_val, x_val));
    chart.draw_series(LineSeries::new(series, &BLUE))?;
    Ok(())
}

fn draw_spectrum_plot(f_k: &[f64], m_k: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("widmo_amplituda.png", (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;
    let max_m = m_k.iter().cloned().fold(f64::NAN, f64::max);
    let mut chart = ChartBuilder::on(&root)
        .caption("widmo amplitudowe M(k)", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..500.0, 0f64..(max_m * 1.1))?;

    chart
        .configure_mesh()
        .x_desc("czestotliwosc [Hz]")
        .y_desc("amplituda M(k)")
        .draw()?;

    let mut lines = Vec::new();
    for (&freq, &amp) in f_k.iter().zip(m_k.iter()) {
        lines.push(PathElement::new(vec![(freq, 0.0), (freq, amp)], &BLUE));
    }
    chart.draw_series(lines)?;
    Ok(())
}
