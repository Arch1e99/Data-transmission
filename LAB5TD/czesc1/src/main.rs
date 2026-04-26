use num_complex::Complex;
use plotters::prelude::*;
use rustfft::FftPlanner;
use std::f64::consts::PI;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fm = 2.0;
    let fn_c = 50.0;
    let fs = 1000.0;
    let t_end = 1.0;
    let n = (t_end * fs) as usize;

    let mut t = Vec::with_capacity(n);
    let mut m = Vec::with_capacity(n);
    for i in 0..n {
        let ct = i as f64 / fs;
        t.push(ct);
        m.push((2.0 * PI * fm * ct).sin());
    }

    let ka_v = [0.5, 5.0, 25.0];
    let kp_v = [0.5, 2.0, 8.0];
    let kf_v = [0.5, 2.0, 8.0];

    let mut f = File::create("wyniki.txt")?;
    writeln!(f, "wyniki szerokosci pasma - lab 05 (czesc 1)\n")?;

    writeln!(f, "modulacja amplitudy (am)")?;
    for &ka in &ka_v {
        let s: Vec<f64> = t
            .iter()
            .zip(m.iter())
            .map(|(&tv, &mv)| (ka * mv + 1.0) * (2.0 * PI * fn_c * tv).cos())
            .collect();
        process_p1("am", "ka", ka, &s, fs, &mut f)?;
    }

    writeln!(f, "\nmodulacja fazy (pm)")?;
    for &kp in &kp_v {
        let s: Vec<f64> = t
            .iter()
            .zip(m.iter())
            .map(|(&tv, &mv)| (2.0 * PI * fn_c * tv + kp * mv).cos())
            .collect();
        process_p1("pm", "kp", kp, &s, fs, &mut f)?;
    }

    writeln!(f, "\nmodulacja czestotliwosci (fm)")?;
    for &kf in &kf_v {
        let s: Vec<f64> = t
            .iter()
            .zip(m.iter())
            .map(|(&tv, &mv)| (2.0 * PI * fn_c * tv + (kf / fm) * mv).cos())
            .collect();
        process_p1("fm", "kf", kf, &s, fs, &mut f)?;
    }

    Ok(())
}

fn process_p1(
    mt: &str,
    pn: &str,
    pv: f64,
    sig: &[f64],
    fs: f64,
    out: &mut File,
) -> Result<(), Box<dyn std::error::Error>> {
    let n = sig.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    let mut buf: Vec<Complex<f64>> = sig.iter().map(|&v| Complex { re: v, im: 0.0 }).collect();
    fft.process(&mut buf);

    let mut freqs = Vec::new();
    let mut db_vals = Vec::new();
    let mut max_db = f64::MIN;

    for j in 0..(n / 2) {
        let freq = j as f64 * fs / n as f64;
        let mut mag = buf[j].norm() / n as f64;
        if j > 0 {
            mag *= 2.0;
        }
        let db = if mag > 1e-12 {
            20.0 * mag.log10()
        } else {
            -200.0
        };
        freqs.push(freq);
        db_vals.push(db);
        if db > max_db {
            max_db = db;
        }
    }

    let calc_b = |drop: f64| {
        let th = max_db - drop;
        let (mut fmin, mut fmax) = (fs, 0.0);
        for (i, &v) in db_vals.iter().enumerate() {
            if v >= th {
                if freqs[i] < fmin {
                    fmin = freqs[i];
                }
                if freqs[i] > fmax {
                    fmax = freqs[i];
                }
            }
        }
        fmax - fmin
    };

    let b3 = calc_b(3.0);
    let b6 = calc_b(6.0);
    let b10 = calc_b(10.0);

    writeln!(
        out,
        "[{}] {} = {:>4} | b_3db = {:>4} hz | b_6db = {:>4} hz | b_10db = {:>4} hz",
        mt, pn, pv, b3, b6, b10
    )?;
    let plik_wyjsciowy = format!("{}_{}_{}.png", mt, pn, pv);
    let root = BitMapBackend::new(&plik_wyjsciowy, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} {} = {} (db scale)", mt, pn, pv),
            ("sans-serif", 20),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0.0..120.0, (max_db - 40.0)..(max_db + 5.0))?;
    chart
        .configure_mesh()
        .x_desc("czestotliwosc [hz]")
        .y_desc("amplituda [db]")
        .draw()?;
    for (&f, &db) in freqs.iter().zip(db_vals.iter()) {
        if db > max_db - 40.0 {
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(f, max_db - 40.0), (f, db)],
                &BLACK,
            )))?;
        }
    }
    chart.draw_series(std::iter::once(PathElement::new(
        vec![(0.0, max_db - 10.0), (120.0, max_db - 10.0)],
        &RED.mix(0.5),
    )))?;
    root.present()?;

    Ok(())
}
