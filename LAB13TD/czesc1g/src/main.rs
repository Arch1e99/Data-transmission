use plotters::prelude::*;
use rand::Rng;
use std::f64::consts::PI;
use std::fs;

#[derive(Clone, Copy)]
enum Modulation {
    ASK,
    PSK,
    FSK,
}

#[derive(Clone, Copy)]
enum Coding {
    Hamming74,
    Hamming1511,
}

#[derive(Clone, Copy)]
enum Configuration {
    NoiseThenAttenuation,
    AttenuationThenNoise,
}

struct SimParams {
    alpha: f64,
    beta: f64,
    modulation: Modulation,
    coding: Coding,
    config: Configuration,
    w: f64,
    t_c: f64,
    samples_per_bit: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("lab-13")?;

    let modulations = [Modulation::ASK, Modulation::PSK, Modulation::FSK];
    let codings = [Coding::Hamming74, Coding::Hamming1511];
    let configs = [
        Configuration::NoiseThenAttenuation,
        Configuration::AttenuationThenNoise,
    ];

    let alpha_steps = 10;
    let beta_steps = 10;
    let alpha_max = 3.0;
    let beta_max = 1.0;
    let bits_count = 1000;

    let mut count = 1;

    for &mod_type in &modulations {
        for &cod_type in &codings {
            for &conf_type in &configs {
                let mod_str = match mod_type {
                    Modulation::ASK => "ASK",
                    Modulation::PSK => "PSK",
                    Modulation::FSK => "FSK",
                };
                let cod_str = match cod_type {
                    Coding::Hamming74 => "Hamming74",
                    Coding::Hamming1511 => "Hamming1511",
                };
                let conf_str = match conf_type {
                    Configuration::NoiseThenAttenuation => "I+II",
                    Configuration::AttenuationThenNoise => "II+I",
                };

                let filename = format!("lab-13/wykres_{}_{}_{}.png", mod_str, cod_str, conf_str);
                let title = format!("BER: {} | {} | {}", mod_str, cod_str, conf_str);

                let mut data = vec![vec![0.0; beta_steps + 1]; alpha_steps + 1];

                for a in 0..=alpha_steps {
                    let alpha = (a as f64) * (alpha_max / alpha_steps as f64);

                    for b in 0..=beta_steps {
                        let beta = (b as f64) * (beta_max / beta_steps as f64);

                        let params = SimParams {
                            alpha,
                            beta,
                            modulation: mod_type,
                            coding: cod_type,
                            config: conf_type,
                            w: 2.0,
                            t_c: 1.0,
                            samples_per_bit: 40,
                        };

                        let ber = run_simulation(&params, bits_count);
                        data[a][b] = ber;
                    }
                }

                plot_3d_surface(
                    &filename,
                    &title,
                    &data,
                    alpha_max,
                    beta_max,
                    alpha_steps,
                    beta_steps,
                )?;
                println!("[{:02}/12] Wygenerowano: {}", count, filename);
                count += 1;
            }
        }
    }

    println!("Symulacja zakończona pomyślnie.");
    Ok(())
}

fn run_simulation(params: &SimParams, data_bit_count: usize) -> f64 {
    let mut rng = rand::thread_rng();
    let original_bits: Vec<u8> = (0..data_bit_count).map(|_| rng.gen_range(0..=1)).collect();

    let coded_bits = match params.coding {
        Coding::Hamming74 => apply_hamming(&original_bits, 3),
        Coding::Hamming1511 => apply_hamming(&original_bits, 4),
    };

    let total_bits = coded_bits.len();
    let t_b = params.t_c / total_bits as f64;
    let f_n = params.w / t_b;
    let f_n1 = (params.w + 1.0) / t_b;
    let f_n2 = (params.w + 2.0) / t_b;
    let dt = t_b / params.samples_per_bit as f64;

    // 1. Modulacja (tworzenie sygnału z użyciem nowych parametrów z SimParams)
    let mut signal = Vec::with_capacity(total_bits * params.samples_per_bit);
    for (i, &b) in coded_bits.iter().enumerate() {
        for s in 0..params.samples_per_bit {
            let t = (i * params.samples_per_bit + s) as f64 * dt;
            match params.modulation {
                Modulation::ASK => {
                    let a = if b == 0 { 1.0 } else { 0.5 };
                    signal.push(a * (2.0 * PI * f_n * t).sin());
                }
                Modulation::PSK => {
                    let phase = if b == 0 { 0.0 } else { PI };
                    signal.push((2.0 * PI * f_n * t + phase).sin());
                }
                Modulation::FSK => {
                    let f = if b == 0 { f_n1 } else { f_n2 };
                    signal.push((2.0 * PI * f * t).sin());
                }
            }
        }
    }

    // 2. Kanał transmisyjny
    match params.config {
        Configuration::NoiseThenAttenuation => {
            add_noise(&mut signal, params.alpha);
            apply_attenuation(&mut signal, params.beta, dt);
        }
        Configuration::AttenuationThenNoise => {
            apply_attenuation(&mut signal, params.beta, dt);
            add_noise(&mut signal, params.alpha);
        }
    }

    // 3. Demodulacja (Odbiornik Korelacyjny)
    let mut demodulated_bits = Vec::with_capacity(total_bits);
    for i in 0..total_bits {
        let mut sum_1 = 0.0;
        let mut sum_2 = 0.0;

        let t_mid = (i as f64 + 0.5) * t_b;
        let limit = 1.0 - params.beta;
        let g_mid = if params.beta >= 1.0 || t_mid > limit {
            0.0
        } else {
            (1.0 - t_mid / limit).powi(2)
        };

        for s in 0..params.samples_per_bit {
            let t = (i * params.samples_per_bit + s) as f64 * dt;
            let sample = signal[i * params.samples_per_bit + s];

            match params.modulation {
                Modulation::ASK => sum_1 += sample * (2.0 * PI * f_n * t).sin(),
                Modulation::PSK => sum_1 += sample * (2.0 * PI * f_n * t).sin(),
                Modulation::FSK => {
                    sum_1 += sample * (2.0 * PI * f_n1 * t).sin();
                    sum_2 += sample * (2.0 * PI * f_n2 * t).sin();
                }
            }
        }

        let sum_1_avg = sum_1 / params.samples_per_bit as f64;

        let bit = match params.modulation {
            Modulation::ASK => {
                if sum_1_avg > (0.375 * g_mid) {
                    0
                } else {
                    1
                }
            }
            Modulation::PSK => {
                if sum_1 > 0.0 {
                    0
                } else {
                    1
                }
            }
            Modulation::FSK => {
                if sum_1 > sum_2 {
                    0
                } else {
                    1
                }
            }
        };
        demodulated_bits.push(bit);
    }

    // 4. Dekodowanie
    let final_bits = match params.coding {
        Coding::Hamming74 => decode_hamming(&demodulated_bits, 3),
        Coding::Hamming1511 => decode_hamming(&demodulated_bits, 4),
    };

    calculate_ber(&original_bits, &final_bits)
}

fn add_noise(signal: &mut [f64], alpha: f64) {
    let mut rng = rand::thread_rng();
    for sample in signal.iter_mut() {
        let noise: f64 = rng.gen_range(-1.0..=1.0);
        *sample += alpha * noise;
    }
}

fn apply_attenuation(signal: &mut [f64], beta: f64, dt: f64) {
    let limit = 1.0 - beta;
    for (i, sample) in signal.iter_mut().enumerate() {
        let t = i as f64 * dt;
        if beta >= 1.0 || t > limit {
            *sample = 0.0;
        } else {
            let g_t = (1.0 - t / limit).powi(2);
            *sample *= g_t;
        }
    }
}

fn plot_3d_surface(
    filename: &str,
    title: &str,
    data: &Vec<Vec<f64>>,
    alpha_max: f64,
    beta_max: f64,
    alpha_steps: usize,
    beta_steps: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut max_ber = 0.0;
    for row in data {
        for &val in row {
            if val > max_ber {
                max_ber = val;
            }
        }
    }
    let z_max = if max_ber > 0.0 { max_ber } else { 0.1 };

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 25).into_font())
        .margin(20)
        .build_cartesian_3d(0.0..alpha_max, 0.0..z_max, 0.0..beta_max)?;

    chart.with_projection(|mut pb| {
        pb.yaw = 0.5;
        pb.pitch = 0.25;
        pb.scale = 0.85;
        pb.into_matrix()
    });

    chart
        .configure_axes()
        .x_formatter(&|x| format!("{:.1}", x))
        .y_formatter(&|y| format!("{:.3}", y))
        .z_formatter(&|z| format!("{:.1}", z))
        .draw()?;

    chart.draw_series(
        SurfaceSeries::xoz(
            (0..=alpha_steps).map(|a| (a as f64) * (alpha_max / alpha_steps as f64)),
            (0..=beta_steps).map(|b| (b as f64) * (beta_max / beta_steps as f64)),
            |x, z| {
                let a = ((x / alpha_max) * alpha_steps as f64).round() as usize;
                let b = ((z / beta_max) * beta_steps as f64).round() as usize;
                data[a.min(alpha_steps)][b.min(beta_steps)]
            },
        )
        .style_func(&|&v| {
            let intensity = (v / z_max).max(0.0).min(1.0);
            let (hue, lightness) = if intensity < 0.5 {
                (0.7 + (intensity * 2.0) * 0.3, 0.2 + (intensity * 2.0) * 0.3)
            } else {
                (
                    0.0 + ((intensity - 0.5) * 2.0) * 0.15,
                    0.5 + ((intensity - 0.5) * 2.0) * 0.1,
                )
            };
            HSLColor(hue, 1.0, lightness).into()
        }),
    )?;

    root.present()?;
    Ok(())
}

fn calculate_ber(original: &[u8], received: &[u8]) -> f64 {
    let mut errors = 0;
    let len = original.len().min(received.len());
    for i in 0..len {
        if original[i] != received[i] {
            errors += 1;
        }
    }
    errors as f64 / len as f64
}

fn apply_hamming(data: &[u8], m: u32) -> Vec<u8> {
    let n = (1 << m) - 1;
    let k = n - m as usize;
    let mut encoded = Vec::new();
    for chunk in data.chunks(k) {
        let mut block = vec![0u8; n];
        let mut data_idx = 0;
        for i in 1..=n {
            if (i & (i - 1)) != 0 && data_idx < chunk.len() {
                block[i - 1] = chunk[data_idx];
                data_idx += 1;
            }
        }
        for i in 1..=n {
            if block[i - 1] == 1 {
                for j in 0..m {
                    if (i & (1 << j)) != 0 {
                        block[(1 << j) - 1] ^= 1;
                    }
                }
            }
        }
        encoded.extend(block);
    }
    encoded
}

fn decode_hamming(received: &[u8], m: u32) -> Vec<u8> {
    let n = (1 << m) - 1;
    let mut decoded_data = Vec::new();
    for chunk in received.chunks(n) {
        if chunk.len() < n {
            break;
        }
        let mut block = chunk.to_vec();
        let mut syndrome = 0;
        for i in 1..=n {
            if block[i - 1] == 1 {
                syndrome ^= i;
            }
        }
        if syndrome > 0 && syndrome <= n {
            block[syndrome - 1] ^= 1;
        }
        for i in 1..=n {
            if (i & (i - 1)) != 0 {
                decoded_data.push(block[i - 1]);
            }
        }
    }
    decoded_data
}
