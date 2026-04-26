// CZESC 2
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
    let fs_1 = 1000.0;
    let f_1 = 50.0;
    let n_1 = 1000;
    let mut x_vals_1 = Vec::new();

    for i in 0..n_1 {
        let t = i as f64 / fs_1;
        x_vals_1.push((2.0 * PI * f_1 * t).sin());
    }

    let x_k_1 = dft(&x_vals_1);
    let mut f_k_1 = Vec::new();
    let mut m_db_1 = Vec::new();

    for k in 0..(n_1 / 2) {
        let freq = k as f64 * (fs_1 / n_1 as f64);
        f_k_1.push(freq);

        let m_k = (x_k_1[k].re.powi(2) + x_k_1[k].im.powi(2)).sqrt();
        let decibels = 10.0 * m_k.max(1e-12).log10();
        m_db_1.push(decibels);
    }

    draw_db_plot_single(&f_k_1, &m_db_1)?;

    let fs_2 = 2000.0;
    let f1 = 10.0;
    let f2 = fs_2 / 2.0 - f1; // 990
    let f3 = f1 / 2.0;        // 5
    let n_2 = 2000;
    let mut x_vals_2 = Vec::new();

    for i in 0..n_2 {
        let t = i as f64 / fs_2;
        let val = (2.0 * PI * f1 * t).sin() + (2.0 * PI * f2 * t).sin() + (2.0 * PI * f3 * t).sin();
        x_vals_2.push(val);
    }

    let x_k_2 = dft(&x_vals_2);
    let mut f_k_2 = Vec::new();
    let mut m_db_2 = Vec::new();

    for k in 0..(n_2 / 2) {
        let freq = k as f64 * (fs_2 / n_2 as f64);
        f_k_2.push(freq);

        let m_k = (x_k_2[k].re.powi(2) + x_k_2[k].im.powi(2)).sqrt();
        let decibels = 10.0 * m_k.max(1e-12).log10();
        m_db_2.push(decibels);
    }

    draw_db_plot_linear(&f_k_2, &m_db_2)?;
    draw_db_plot_log(&f_k_2, &m_db_2)?;
    Ok(())
}

fn draw_db_plot_single(f_k: &[f64], m_db: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("widmo_pojedyncze_db.png", (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("widmo M'(k) w [dB] (pojedynczy ton 50Hz)", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..500.0, -50.0f64..50.0f64)?;

    chart.configure_mesh().x_desc("czestotliwosc [Hz]").y_desc("amplituda [dB]").draw()?;

    let series = f_k.iter().zip(m_db.iter()).map(|(&f, &m)| (f, m));
    chart.draw_series(LineSeries::new(series, &BLUE))?;
    Ok(())
}

fn draw_db_plot_linear(f_k: &[f64], m_db: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("widmo_db_liniowa.png", (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("widmo M'(k) w [dB] (os X: liniowa)", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..1000.0, -50.0f64..50.0f64)?;

    chart.configure_mesh().x_desc("czestotliwosc [Hz]").y_desc("amplituda [dB]").draw()?;

    let series = f_k.iter().zip(m_db.iter()).map(|(&f, &m)| (f, m));
    chart.draw_series(LineSeries::new(series, &RED))?;
    Ok(())
}

fn draw_db_plot_log(f_k: &[f64], m_db: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("widmo_db_log.png", (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("widmo M'(k) w [dB] (os X: logarytmiczna)", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d((1.0f64..1000.0).log_scale(), -50.0f64..50.0f64)?;

    chart.configure_mesh().x_desc("czestotliwosc [Hz] (Log)").y_desc("amplituda [dB]").draw()?;

    let series = f_k
        .iter()
        .zip(m_db.iter())
        .filter(|&(f, _)| *f >= 1.0)
        .map(|(&f, &m)| (f, m));

    chart.draw_series(LineSeries::new(series, &GREEN))?;
    Ok(())
}
