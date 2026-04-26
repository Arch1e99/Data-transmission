// CZESC 1
// DO ZAD 1 (Tabela 1, pozycja 1); DO ZAD 2 (Tabela 2, pozycja 3)
use plotters::prelude::*;
use std::f64::consts::PI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("czesc 1...");

    let fs = 8000.0;
    let tc = 5.0;
    let n = (tc * fs) as usize;

    let f = 10.0;
    let phi = 2.0;

    let mut t_vec = Vec::with_capacity(n);
    let mut x_vec = Vec::with_capacity(n);
    let mut y_vec = Vec::with_capacity(n);
    let mut z_vec = Vec::with_capacity(n);
    let mut v_vec = Vec::with_capacity(n);

    for i in 0..n {
        let t = i as f64 / fs;
        t_vec.push(t);

        let x = (2.0 * PI * f * t + phi).cos() * (2.5 * t.powf(0.2) * PI).cos();
        x_vec.push(x);

        let y = (t.powi(3) - 1.0) + (4.0 * t.powi(2) * PI).cos() * t;
        y_vec.push(y);

        let z = x / ((y * (5.0 * t).cos() - x * y).abs() + 3.0);
        z_vec.push(z);

        let v = (x * 662.0) / ((x - y).abs() + 0.5);
        v_vec.push(v);
    }

    plot_signal("x_t.png", "Sygnal x(t)", &t_vec, &x_vec)?;
    plot_signal("y_t.png", "Sygnal y(t)", &t_vec, &y_vec)?;
    plot_signal("z_t.png", "Sygnal z(t)", &t_vec, &z_vec)?;
    plot_signal("v_t.png", "Sygnal v(t)", &t_vec, &v_vec)?;

    println!("done");
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

    let y_margin = (max_y - min_y) * 0.1;

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
