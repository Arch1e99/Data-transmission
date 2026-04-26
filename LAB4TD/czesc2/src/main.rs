use num_complex::Complex;
use plotters::prelude::*;
use rustfft::FftPlanner;
use std::f64::consts::PI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fm = 2.0;
    let fn_carrier = 50.0;
    let fs = 1000.0;
    let t_end = 1.0;
    let num_samples = (t_end * fs) as usize;

    let mut t = Vec::with_capacity(num_samples);
    let mut m = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let current_t = i as f64 / fs;
        t.push(current_t);
        m.push((2.0 * PI * fm * current_t).sin());
    }

    let ka_vals = [0.5, 5.0, 25.0];
    let kp_vals = [0.5, 2.0, 8.0];
    let kf_vals = [0.5, 2.0, 8.0];

    plot_spectra(
        "am_spectra.png",
        "widmo amplitudowe (AM)",
        "kA",
        &ka_vals,
        &t,
        &m,
        fs,
        |ka, t_val, m_val| (ka * m_val + 1.0) * (2.0 * PI * fn_carrier * t_val).cos(),
    )?;

    plot_spectra(
        "pm_spectra.png",
        "widmo amplitudowe (PM)",
        "kP",
        &kp_vals,
        &t,
        &m,
        fs,
        |kp, t_val, m_val| (2.0 * PI * fn_carrier * t_val + kp * m_val).cos(),
    )?;

    plot_spectra(
        "fm_spectra.png",
        "widmo amplitudowe (FM)",
        "kF",
        &kf_vals,
        &t,
        &m,
        fs,
        |kf, t_val, m_val| (2.0 * PI * fn_carrier * t_val + (kf / fm) * m_val).cos(),
    )?;

    Ok(())
}

fn plot_spectra<F>(
    filename: &str,
    title: &str,
    k_name: &str,
    k_vals: &[f64; 3],
    t: &[f64],
    m: &[f64],
    fs: f64,
    mod_func: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(f64, f64, f64) -> f64,
{
    let root = BitMapBackend::new(filename, (1000, 900)).into_drawing_area();
    root.fill(&WHITE)?;
    let root = root.titled(title, ("sans-serif", 35))?;
    let panels = root.split_evenly((3, 1));
    let num_samples = t.len();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(num_samples);

    for (i, &k) in k_vals.iter().enumerate() {
        let mut buffer: Vec<Complex<f64>> = t
            .iter()
            .zip(m.iter())
            .map(|(&t_val, &m_val)| Complex {
                re: mod_func(k, t_val, m_val),
                im: 0.0,
            })
            .collect();

        fft.process(&mut buffer);

        let mut freqs = Vec::new();
        let mut magnitudes = Vec::new();
        let mut max_mag = 0.0;

        for j in 0..(num_samples / 2) {
            let freq = j as f64 * fs / num_samples as f64;

            let mut mag = buffer[j].norm() / num_samples as f64;
            if j > 0 {
                mag *= 2.0;
            }

            if freq <= 120.0 {
                freqs.push(freq);
                magnitudes.push(mag);
                if mag > max_mag {
                    max_mag = mag;
                }
            }
        }

        max_mag = (max_mag * 1.2).max(0.1);

        let mut chart = ChartBuilder::on(&panels[i])
            .caption(format!("{} = {}", k_name, k), ("sans-serif", 20))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(0f64..120.0f64, 0f64..max_mag)?;

        chart
            .configure_mesh()
            .x_desc("czestotliwosc")
            .y_desc("amplituda")
            .draw()?;

        for (&f, &m) in freqs.iter().zip(magnitudes.iter()) {
            if m > 0.005 {
                chart.draw_series(std::iter::once(PathElement::new(
                    vec![(f, 0.0), (f, m)],
                    &BLACK,
                )))?;
                chart.draw_series(std::iter::once(Circle::new((f, m), 2, BLACK.filled())))?;
            }
        }
    }

    root.present()?;
    Ok(())
}
