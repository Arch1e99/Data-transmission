/* CZESC 1
* wyznaczenie liczby prazkow w zakresie (0, fs/2):
* zgodnie z teoria, dla N probek, transformata Fouriera zwraca N prazkow
* zakres (0, fs/2) odpowiada pierwszej polowie wyniku FFT, czyli N/2 prazkow
* dla N = 200, liczba prazkow w zakresie (0, fs/2) wynosi 100
*/

use plotters::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex};
use std::f64::consts::PI;

const FS: f64 = 100.0;
const F: f64 = 2.0;
const H: usize = 10; // liczba skladowych harmonicznych
const N: usize = 200;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut t = vec![0.0; N];
    let mut x = vec![0.0; N];
    let mut y = vec![0.0; N];
    let mut z = vec![0.0; N];

    for i in 0..N {
        t[i] = i as f64 / FS;

        //x(t)
        let mut sum_x = 0.0;
        for k in 1..=H {
            let sign = if (k + 1) % 2 == 0 { 1.0 } else { -1.0 };
            sum_x += sign * (2.0 * PI * k as f64 * F * t[i]).sin() / (k as f64);
        }
        x[i] = (2.0 / PI) * sum_x;

        //y(t)
        let mut sum_y = 0.0;
        for k in 1..=H {
            let sign = if (k - 1) % 2 == 0 { 1.0 } else { -1.0 };
            let n = 2.0 * k as f64 - 1.0;
            sum_y += sign * (2.0 * PI * n * F * t[i]).sin() / (n * n);
        }
        y[i] = (8.0 / (PI * PI)) * sum_y;

        //z(t)
        let mut sum_z = 0.0;
        for k in 1..=H {
            let n = 2.0 * k as f64 - 1.0;
            sum_z += (2.0 * PI * n * F * t[i]).sin() / n;
        }
        z[i] = (4.0 / PI) * sum_z;
    }
    let m_x = compute_spectrum(&x);
    let m_y = compute_spectrum(&y);
    let m_z = compute_spectrum(&z);

    plot_signal_and_spectrum("x_plot.png", "Sygnal piloksztaltny x(t)", &t, &x, &m_x)?;
    plot_signal_and_spectrum("y_plot.png", "Sygnal trojkatny y(t)", &t, &y, &m_y)?;
    plot_signal_and_spectrum("z_plot.png", "Sygnal prostokatny z(t)", &t, &z, &m_z)?;

    Ok(())
}
fn compute_spectrum(signal: &[f64]) -> Vec<f64> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(signal.len());
    let mut buffer: Vec<Complex<f64>> = signal
        .iter()
        .map(|&val| Complex { re: val, im: 0.0 })
        .collect();

    fft.process(&mut buffer);

    buffer
        .iter()
        .take(signal.len() / 2)
        .map(|c| c.norm() * 2.0 / signal.len() as f64)
        .collect()
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

    let mut chart_time = ChartBuilder::on(&upper)
        .caption(format!("{} - Czas", title), ("sans-serif", 20))
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0..t.last().unwrap().clone(), -2.0f64..2.0f64)?;
    chart_time.configure_mesh().draw()?;
    chart_time.draw_series(LineSeries::new(
        t.iter().zip(signal.iter()).map(|(&x, &y)| (x, y)),
        &BLUE,
    ))?;

    let max_amp = spectrum.iter().cloned().fold(0. / 0., f64::max);
    let freqs: Vec<f64> = (0..spectrum.len())
        .map(|i| i as f64 * FS / N as f64)
        .collect();

    let mut chart_freq = ChartBuilder::on(&lower)
        .caption(format!("{} - Widmo (M)", title), ("sans-serif", 20))
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0..FS / 2.0, 0.0..(max_amp * 1.2))?;
    chart_freq.configure_mesh().draw()?;
    chart_freq.draw_series(spectrum.iter().enumerate().map(|(i, &amp)| {
        let f = freqs[i];
        PathElement::new(vec![(f, 0.0), (f, amp)], &RED)
    }))?;

    root.present()?;
    Ok(())
}
