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

    let mut f = File::create("wyniki_czesc2.txt")?;
    writeln!(f, "wyniki szerokosci pasma - lab 05 (czesc 2)\n")?;

    writeln!(f, "modulacja amplitudy (am)")?;
    for &ka in &ka_v {
        let s: Vec<f64> = t
            .iter()
            .zip(m.iter())
            .map(|(&tv, &mv)| (ka * mv + 1.0) * (2.0 * PI * fn_c * tv).cos())
            .collect();
        process_p2("am", "ka", ka, &s, fs, fn_c, &mut f)?;
    }

    writeln!(f, "\nmodulacja fazy (pm)")?;
    for &kp in &kp_v {
        let s: Vec<f64> = t
            .iter()
            .zip(m.iter())
            .map(|(&tv, &mv)| (2.0 * PI * fn_c * tv + kp * mv).cos())
            .collect();
        process_p2("pm", "kp", kp, &s, fs, fn_c, &mut f)?;
    }

    writeln!(f, "\nmodulacja czestotliwosci (fm)")?;
    for &kf in &kf_v {
        let s: Vec<f64> = t
            .iter()
            .zip(m.iter())
            .map(|(&tv, &mv)| (2.0 * PI * fn_c * tv + (kf / fm) * mv).cos())
            .collect();
        process_p2("fm", "kf", kf, &s, fs, fn_c, &mut f)?;
    }

    Ok(())
}

fn process_p2(
    mt: &str,
    pn: &str,
    pv: f64,
    sig: &[f64],
    fs: f64,
    fn_c: f64,
    out: &mut File,
) -> Result<(), Box<dyn std::error::Error>> {
    let n = sig.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    let mut buf: Vec<Complex<f64>> = sig.iter().map(|&v| Complex { re: v, im: 0.0 }).collect();
    fft.process(&mut buf);

    let mut freqs = Vec::new();
    let mut mags = Vec::new();
    let mut tot_e = 0.0;
    let mut max_m = 0.0;

    for j in 0..(n / 2) {
        let mut mag = buf[j].norm() / n as f64;
        if j > 0 {
            mag *= 2.0;
        }
        freqs.push(j as f64 * fs / n as f64);
        mags.push(mag);
        tot_e += mag.powi(2);
        if mag > max_m {
            max_m = mag;
        }
    }

    let mut alpha = 0.0;
    let mut cur_r = 0.0;
    for ai in 1..=400 {
        let a = ai as f64 * 0.5;
        let ia = ((fn_c - a) * n as f64 / fs).round() as i32;
        let ib = ((fn_c + a) * n as f64 / fs).round() as i32;
        let ia = ia.max(0) as usize;
        let ib = ib.min((mags.len() - 1) as i32) as usize;
        let mut win_e = 0.0;
        for k in ia..=ib {
            win_e += mags[k].powi(2);
        }
        cur_r = (win_e / tot_e) * 100.0;
        if cur_r > 80.0 {
            alpha = a;
            break;
        }
    }

    writeln!(
        out,
        "[{}] {} = {:>4} | alfa = {:>4.1} hz | r_alfa = {:>6.2}% | pasmo b = {:>4.1} hz",
        mt,
        pn,
        pv,
        alpha,
        cur_r,
        2.0 * alpha
    )?;

    let plik_wyjsciowy = format!("{}_{}_{}_e80.png", mt, pn, pv);
    let root = BitMapBackend::new(&plik_wyjsciowy, (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} {} = {} (linear scale)", mt, pn, pv),
            ("sans-serif", 20),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0.0..120.0, 0.0..(max_m * 1.1))?;
    chart
        .configure_mesh()
        .x_desc("czestotliwosc [hz]")
        .y_desc("amplituda")
        .draw()?;
    chart.draw_series(std::iter::once(Rectangle::new(
        [(fn_c - alpha, 0.0), (fn_c + alpha, max_m * 1.1)],
        BLUE.mix(0.1).filled(),
    )))?;
    for (&f, &m) in freqs.iter().zip(mags.iter()) {
        if m > 0.001 {
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(f, 0.0), (f, m)],
                &BLACK,
            )))?;
        }
    }
    root.present()?;

    Ok(())
}
