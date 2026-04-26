// CZESC 2
// DO ZAD 1 (Tabela 3, pozycja 1); DO ZAD 2 (Tabela 4, pozycja 1)
use plotters::prelude::*;
use std::f64::consts::PI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ZAD 1
    let fs_u: f64 = 8000.0;
    let tc_u: f64 = 1.0;
    let n_u = (tc_u * fs_u).round() as usize;

    let mut t_u = Vec::with_capacity(n_u);
    let mut u_vec = Vec::with_capacity(n_u);

    for i in 0..n_u {
        let t = i as f64 / fs_u;
        t_u.push(t);

        let u = if t >= 0.0 && t < 0.1 {
            (6.0 * t).sin() * (5.0 * PI * t).cos()
        } else if t >= 0.1 && t < 0.4 {
            -1.1 * t - (41.0 * t.powi(2)).cos()
        } else if t >= 0.4 && t < 0.72 {
            t * (20.0 * t.powi(4)).sin()
        } else if t >= 0.72 && t < 1.0 {
            3.3 * (t - 0.72) * (27.0 * t + 1.3).cos()
        } else {
            0.0
        };
        u_vec.push(u);
    }
    plot_signal("u_t.png", "Sygnal u(t)", &t_u, &u_vec)?;

    // ZAD 2
    let fs_b: f64 = 22050.0;
    let tc_b: f64 = 1.0;
    let n_b = (tc_b * fs_b).round() as usize;

    let mut t_b = Vec::with_capacity(n_b);
    let mut b1_vec = Vec::with_capacity(n_b);
    let mut b2_vec = Vec::with_capacity(n_b);
    let mut b3_vec = Vec::with_capacity(n_b);

    let h_vals = [5, 20, 50];

    for i in 0..n_b {
        let t = i as f64 / fs_b;
        t_b.push(t);

        let calc_bk = |h_max: i32| -> f64 {
            let mut sum = 0.0;
            for h in 1..=h_max {
                let h_f64 = h as f64;
                sum += ((-1.0_f64).powi(h) / h_f64) * (h_f64 * PI * 2.0 * t).sin();
            }
            (2.0 / PI) * sum
        };

        b1_vec.push(calc_bk(h_vals[0]));
        b2_vec.push(calc_bk(h_vals[1]));
        b3_vec.push(calc_bk(h_vals[2]));
    }

    plot_signal("b1_t.png", "Sygnal b1(t) [H=5]", &t_b, &b1_vec)?;
    plot_signal("b2_t.png", "Sygnal b2(t) [H=20]", &t_b, &b2_vec)?;
    plot_signal("b3_t.png", "Sygnal b3(t) [H=50]", &t_b, &b3_vec)?;

    Ok(())
}

fn plot_signal(
    filename: &str,
    caption: &str,
    t: &[f64],
    y: &[f64],
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_y = y.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y = y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let max_t = t.last().copied().unwrap_or(1.0);

    let y_margin = if (max_y - min_y).abs() < f64::EPSILON {
        1.0
    } else {
        (max_y - min_y) * 0.1
    };

    let mut chart = ChartBuilder::on(&root)
        .caption(caption, ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..max_t, (min_y - y_margin)..(max_y + y_margin))?;

    chart
        .configure_mesh()
        .x_desc("Czas [s]")
        .y_desc("Amplituda")
        .draw()?;

    chart.draw_series(LineSeries::new(
        t.iter().zip(y.iter()).map(|(&x, &y)| (x, y)),
        &BLUE,
    ))?;

    root.present()?;
    Ok(())
}
