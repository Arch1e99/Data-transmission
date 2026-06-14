use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::f64::consts::PI;
use std::fs;

const A: f64 = 1.0;
const A1: f64 = 1.0;
const A2: f64 = 0.5;

const INFO_BITS: usize = 976;

const STEPS: usize = 10;
const ALPHA_STEP: f64 = 0.3;
const BETA_STEP: f64 = 0.1;
const SEED: u64 = 2026;
const NREP: usize = 40;

#[derive(Clone, Copy, PartialEq)]
enum Modulation {
    Ask,
    Psk,
    Fsk,
}

#[derive(Clone, Copy, PartialEq)]
enum Config {
    NoiseThenAtten,
    AttenThenNoise,
}

fn is_power_of_two(i: usize) -> bool {
    i != 0 && (i & (i - 1)) == 0
}

fn hamming_encode(data: &[u8], n: usize, k: usize) -> Vec<u8> {
    let r = n - k;
    let mut out = Vec::new();
    let mut idx = 0;
    while idx < data.len() {
        let mut block = vec![0u8; n];
        let mut di = idx;
        for i in 1..=n {
            if !is_power_of_two(i) {
                block[i - 1] = if di < data.len() { data[di] } else { 0 };
                di += 1;
            }
        }
        let mut s = 0usize;
        for i in 1..=n {
            if block[i - 1] == 1 {
                s ^= i;
            }
        }
        for j in 0..r {
            if (s >> j) & 1 == 1 {
                block[(1 << j) - 1] = 1;
            }
        }
        out.extend_from_slice(&block);
        idx += k;
    }
    out
}

fn hamming_decode(coded: &[u8], n: usize, _k: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let mut idx = 0;
    while idx + n <= coded.len() {
        let mut block: Vec<u8> = coded[idx..idx + n].to_vec();
        let mut s = 0usize;
        for i in 1..=n {
            if block[i - 1] == 1 {
                s ^= i;
            }
        }
        if s != 0 && s <= n {
            block[s - 1] ^= 1;
        }
        for i in 1..=n {
            if !is_power_of_two(i) {
                out.push(block[i - 1]);
            }
        }
        idx += n;
    }
    out
}

fn attenuation(tnorm: f64, beta: f64) -> f64 {
    if beta < 1.0 && tnorm < 1.0 - beta {
        let r = (1.0 - tnorm) / (1.0 - beta);
        r * r
    } else {
        0.0
    }
}

fn gauss(rng: &mut StdRng) -> f64 {
    let u1: f64 = rng.gen_range(1e-12..1.0);
    let u2: f64 = rng.gen_range(0.0..1.0);
    (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}

fn decide_bit(bit: u8, m: Modulation, cfg: Config, alpha: f64, g: f64, rng: &mut StdRng) -> u8 {
    let sigma = match cfg {
        Config::NoiseThenAtten => alpha * g,
        Config::AttenThenNoise => alpha,
    };

    match m {
        Modulation::Psk => {
            let s = if bit == 0 { A } else { -A };
            let d = s * g + sigma * gauss(rng);
            if d > 0.0 { 0 } else { 1 }
        }
        Modulation::Ask => {
            let level = if bit == 0 { A1 } else { A2 };
            let d = level * g + sigma * gauss(rng);
            let thr = 0.5 * (A1 + A2) * g;
            if d > thr { 0 } else { 1 }
        }
        Modulation::Fsk => {
            let sa = if bit == 1 { A } else { 0.0 };
            let sb = if bit == 0 { A } else { 0.0 };
            let da = sa * g + sigma * gauss(rng);
            let db = sb * g + sigma * gauss(rng);
            if da > db { 1 } else { 0 }
        }
    }
}

fn ber_count(a: &[u8], b: &[u8]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut e = 0usize;
    for i in 0..n {
        if a[i] != b[i] {
            e += 1;
        }
    }
    e as f64 / n as f64
}

fn write_csv(path: &str, data: &[Vec<f64>], alpha_max: f64, beta_max: f64) {
    let mut s = String::new();
    s.push_str(&format!("alpha_max,{:.6}\n", alpha_max));
    s.push_str(&format!("beta_max,{:.6}\n", beta_max));
    for row in data {
        let line: Vec<String> = row.iter().map(|v| format!("{:.6}", v)).collect();
        s.push_str(&line.join(","));
        s.push('\n');
    }
    fs::write(path, s).expect("zapis CSV");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = "lab-13/dane";
    fs::create_dir_all(data_dir)?;

    let mut info_rng = StdRng::seed_from_u64(SEED);
    let info: Vec<u8> = (0..INFO_BITS).map(|_| info_rng.gen_range(0..=1)).collect();

    let tx_74 = hamming_encode(&info, 7, 4);
    let tx_1511 = hamming_encode(&info, 15, 11);

    println!(
        "info={} bitow, tx(7,4)={} bitow, tx(15,11)={} bitow | alpha in [0,3], beta in [0,1]",
        info.len(),
        tx_74.len(),
        tx_1511.len()
    );

    let mods = [
        (Modulation::Ask, "ASK"),
        (Modulation::Psk, "PSK"),
        (Modulation::Fsk, "FSK"),
    ];
    let codes: [(&[u8], &str, usize, usize); 2] = [
        (&tx_74, "Hamming74", 7, 4),
        (&tx_1511, "Hamming1511", 15, 11),
    ];
    let cfgs = [
        (Config::NoiseThenAtten, "I+II"),
        (Config::AttenThenNoise, "II+I"),
    ];

    let alpha_max = STEPS as f64 * ALPHA_STEP; // 3.0
    let beta_max = STEPS as f64 * BETA_STEP; // 1.0

    let mut count = 0;
    for (m, ms) in mods {
        for (bits, cs, n, k) in codes {
            let nbits = bits.len();
            let denom = (nbits.saturating_sub(1)).max(1) as f64;

            for (cfg, cfg_str) in cfgs {
                let mut rng = StdRng::seed_from_u64(SEED);
                let mut data = vec![vec![0.0f64; STEPS + 1]; STEPS + 1];

                for ib in 0..=STEPS {
                    let beta = ib as f64 * BETA_STEP;
                    let g: Vec<f64> = (0..nbits)
                        .map(|nn| attenuation(nn as f64 / denom, beta))
                        .collect();

                    for ia in 0..=STEPS {
                        let alpha = ia as f64 * ALPHA_STEP;
                        let mut err_sum = 0.0f64;

                        for _ in 0..NREP {
                            let mut decided = Vec::with_capacity(nbits);
                            for (nn, &b) in bits.iter().enumerate() {
                                decided.push(decide_bit(b, m, cfg, alpha, g[nn], &mut rng));
                            }
                            let info_hat = hamming_decode(&decided, n, k);
                            err_sum += ber_count(&info_hat, &info);
                        }
                        data[ia][ib] = err_sum / NREP as f64;
                    }
                }

                let fname = format!("{}/ber_{}_{}_{}.csv", data_dir, ms, cs, cfg_str);
                write_csv(&fname, &data, alpha_max, beta_max);
                count += 1;
                println!("[{:2}/12] Zapisano {}", count, fname);
            }
        }
    }

    println!("Gotowe. {} zestawow danych w {}/", count, data_dir);
    Ok(())
}
