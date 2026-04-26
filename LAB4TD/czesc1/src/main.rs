use plotters::prelude::*;
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

    plot_signals(
        "am_signals.png",
        "modulacja amplitudy (AM)",
        "kA",
        &ka_vals,
        &t,
        &m,
        |ka, t_val, m_val| (ka * m_val + 1.0) * (2.0 * PI * fn_carrier * t_val).cos(),
    )?;

    plot_signals(
        "pm_signals.png",
        "modulacja fazy (PM)",
        "kP",
        &kp_vals,
        &t,
        &m,
        |kp, t_val, m_val| (2.0 * PI * fn_carrier * t_val + kp * m_val).cos(),
    )?;

    plot_signals(
        "fm_signals.png",
        "modulacja czestotliwosci (FM)",
        "kF",
        &kf_vals,
        &t,
        &m,
        |kf, t_val, m_val| (2.0 * PI * fn_carrier * t_val + (kf / fm) * m_val).cos(),
    )?;

    Ok(())
}

fn plot_signals<F>(
    filename: &str,
    title: &str,
    k_name: &str,
    k_vals: &[f64; 3],
    t: &[f64],
    m: &[f64],
    mod_func: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(f64, f64, f64) -> f64,
{
    let root = BitMapBackend::new(filename, (1000, 900)).into_drawing_area();
    root.fill(&WHITE)?;
    let root = root.titled(title, ("sans-serif", 35))?;

    let panels = root.split_evenly((3, 1));

    for (i, &k) in k_vals.iter().enumerate() {
        let mut z = Vec::with_capacity(t.len());
        let mut max_z = 2.0;

        for j in 0..t.len() {
            let z_val = mod_func(k, t[j], m[j]);
            z.push(z_val);
            if z_val.abs() > max_z {
                max_z = z_val.abs();
            }
        }

        max_z = (max_z * 1.2).ceil();

        let mut chart = ChartBuilder::on(&panels[i])
            .caption(format!("{} = {}", k_name, k), ("sans-serif", 20))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(0f64..1.0f64, -max_z..max_z)?;

        chart
            .configure_mesh()
            .x_desc("czas [s]")
            .y_desc("amplituda")
            .draw()?;

        chart
            .draw_series(LineSeries::new(
                t.iter().zip(z.iter()).map(|(&x, &y)| (x, y)),
                &BLACK.mix(0.4),
            ))?
            .label("z(t)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK.mix(0.4)));

        chart
            .draw_series(LineSeries::new(
                t.iter().zip(m.iter()).map(|(&x, &y)| (x, y)),
                &BLUE,
            ))?
            .label("m(t)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
    }

    root.present()?;
    Ok(())
}
