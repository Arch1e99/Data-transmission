// CZESC 2
use plotters::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex};
use std::f64::consts::PI;

const FS: f64 = 100.0;
const N: usize = 200;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f1 = 3.0;
    let f2 = 7.0;
    let alpha = 2.0;
    let beta = 3.0;

    let mut t = vec![0.0; N];
    let mut x = vec![0.0; N];
    let mut y = vec![0.0; N];
    let mut z = vec![0.0; N];

    // 1 & 2
    for i in 0..N {
        t[i] = i as f64 / FS;
        x[i] = 0.5 * (2.0 * PI * f1 * t[i]).sin();
        y[i] = (2.0 * PI * f2 * t[i]).sin() + 0.7 * (2.0 * PI * f1 * t[i]).sin();
        z[i] = alpha * x[i] + beta * y[i];
    }

    // 3
    let (m_x_complex, m_x) = compute_spectrum_with_complex(&x);
    let (m_y_complex, m_y) = compute_spectrum_with_complex(&y);
    let (_, m_z) = compute_spectrum_with_complex(&z);

    // 4
    let mut m_z_hat = vec![0.0; m_x.len()];
    for i in 0..m_x.len() {
        let combined_complex = Complex { re: alpha, im: 0.0 } * m_x_complex[i]
            + Complex { re: beta, im: 0.0 } * m_y_complex[i];
        m_z_hat[i] = combined_complex.norm() * 2.0 / N as f64;
    }

    plot_signal_and_spectrum("x_plot.png", "Sygnal x(t)", &t, &x, &m_x)?;
    plot_signal_and_spectrum("y_plot.png", "Sygnal y(t)", &t, &y, &m_y)?;
    plot_signal_and_spectrum("z_plot.png", "Sygnal z(t)", &t, &z, &m_z)?;
    plot_only_spectrum("mz_hat_plot.png", "Widmo estymowane Mz_hat", &m_z_hat)?;

    Ok(())
}

fn compute_spectrum_with_complex(signal: &[f64]) -> (Vec<Complex<f64>>, Vec<f64>) {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(signal.len());
    let mut buffer: Vec<Complex<f64>> = signal
        .iter()
        .map(|&val| Complex { re: val, im: 0.0 })
        .collect();

    fft.process(&mut buffer);

    let magnitudes: Vec<f64> = buffer
        .iter()
        .take(signal.len() / 2)
        .map(|c| c.norm() * 2.0 / signal.len() as f64)
        .collect();

    (buffer, magnitudes)
}

fn plot_signal_and_spectrum(
    filename: &str,
    title: &str,
    t: &[f64],
    signal: &[f64],
    spectrum: &[f64],
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let (upper, lower) = root.split_vertically(300);

    let min_y = signal.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y = signal.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut chart_time = ChartBuilder::on(&upper)
        .caption(format!("{} - Czas", title), ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..t.last().unwrap().clone(), (min_y - 0.5)..(max_y + 0.5))?;

    chart_time
        .configure_mesh()
        .x_desc("Czas [s]")
        .y_desc("Amplituda")
        .draw()?;
    chart_time.draw_series(LineSeries::new(
        t.iter().zip(signal.iter()).map(|(&x, &y)| (x, y)),
        &BLUE,
    ))?;

    let max_amp = spectrum.iter().cloned().fold(0. / 0., f64::max);
    let freqs: Vec<f64> = (0..spectrum.len())
        .map(|i| i as f64 * FS / N as f64)
        .collect();

    let mut chart_freq = ChartBuilder::on(&lower)
        .caption(format!("{} - Widmo", title), ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..FS / 2.0, 0.0..(max_amp * 1.2))?;

    chart_freq
        .configure_mesh()
        .x_desc("Czestotliwosc [Hz]")
        .y_desc("Amplituda")
        .draw()?;

    chart_freq.draw_series(spectrum.iter().enumerate().map(|(i, &amp)| {
        let f = freqs[i];
        PathElement::new(vec![(f, 0.0), (f, amp)], &BLACK)
    }))?;

    chart_freq.draw_series(spectrum.iter().enumerate().map(|(i, &amp)| {
        let f = freqs[i];
        Circle::new((f, amp), 3, BLACK.filled())
    }))?;

    root.present()?;
    Ok(())
}

fn plot_only_spectrum(
    filename: &str,
    title: &str,
    spectrum: &[f64],
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (800, 300)).into_drawing_area();
    root.fill(&WHITE)?;
    let max_amp = spectrum.iter().cloned().fold(0. / 0., f64::max);
    let freqs: Vec<f64> = (0..spectrum.len())
        .map(|i| i as f64 * FS / N as f64)
        .collect();

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..FS / 2.0, 0.0..(max_amp * 1.2))?;

    chart
        .configure_mesh()
        .x_desc("Czestotliwosc [Hz]")
        .y_desc("Amplituda")
        .draw()?;

    chart.draw_series(spectrum.iter().enumerate().map(|(i, &amp)| {
        let f = freqs[i];
        PathElement::new(vec![(f, 0.0), (f, amp)], &BLACK)
    }))?;

    chart.draw_series(spectrum.iter().enumerate().map(|(i, &amp)| {
        let f = freqs[i];
        Circle::new((f, amp), 3, BLACK.filled())
    }))?;

    root.present()?;
    Ok(())
}
