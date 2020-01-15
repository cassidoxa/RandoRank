use std::{collections::HashMap, f64::NAN as NaN};

pub fn normalize_race(
    race: &HashMap<String, Option<u32>>,
    norm_factor: &f64,
) -> HashMap<String, (u32, f64)> {
    let mut times: Vec<u32> = race
        .values()
        .filter(|x| x.is_some())
        .map(|&x| x.unwrap())
        .collect();
    times.sort();

    let quartile_1: f64 = percentile_of(times.as_slice(), 25f64);
    let quartile_3: f64 = percentile_of(times.as_slice(), 75f64);
    let iqr = quartile_3 - quartile_1;
    let norm_min: f64 = quartile_1 + (iqr * norm_factor);
    let norm_max: f64 = times[0] as f64;

    let n = |x: f64| -> f64 {
        match (x - norm_min) / (norm_max - norm_min) {
            y if y > 0f64 => y,
            y if y <= 0f64 => 0f64,
            _ => 0f64,
        }
    };

    let mut normed_race: HashMap<String, (u32, f64)> = HashMap::with_capacity(race.len());
    for (key, value) in race.iter() {
        let new_value: u32 = value.unwrap_or(times[times.len() - 1] + 10);
        normed_race.insert(key.to_string(), (new_value, n(new_value as f64)));
    }
    println!("{:?}", normed_race);
    normed_race
}

fn get_sigma(tau: f64, phi: f64, sigma: f64, delta: f64, v: f64) -> f64 {
    const EPSILON: f64 = 0.000_000_001;

    let alpha: f64 = (sigma.powi(2)).ln();
    let f = |x: f64| -> f64 {
        let fraction_one: f64 = {
            let numer = x.exp() * (delta.powi(2) - phi.powi(2) - v - x.exp());
            let denom = 2f64 * (phi.powi(2) + v + x.exp()) * (phi.powi(2) + v + x.exp());
            numer / denom
        };
        let fraction_two: f64 = {
            let numer = x - alpha;
            let denom = tau.powi(2);
            numer / denom
        };
        fraction_one - fraction_two
    };
    let mut a: f64 = alpha;
    let mut b: f64;
    if delta.powi(2) > phi.powi(2) + v {
        b = (delta.powi(2) - phi.powi(2) - v).ln();
    } else {
        let mut k: f64 = 1.0;
        while f(alpha - k * tau) < 0f64 {
            k += 1f64;
        }
        b = alpha - k * tau
    };
    let mut c: f64;

    let mut fa: f64 = f(a);
    let mut fb: f64 = f(b);
    let mut fc: f64;
    while (b - a).abs() > EPSILON {
        c = a + ((a - b) * fa / (fb - fa));
        fc = f(c);
        if fc * fb < 0f64 {
            a = b;
            fa = fb;
        } else {
            fa /= 2f64;
        }

        b = c;
        fb = fc;
    }

    (a / 2f64).exp()
}

fn percentile_of(sorted_times: &[u32], pct: f64) -> f64 {
    if sorted_times.len() == 1 {
        return sorted_times[0] as f64;
    }
    let length = (sorted_times.len() - 1) as f64;
    let rank = (pct / 100f64) * length;
    let lrank = rank.floor();
    let d = rank - lrank;
    let i = lrank as usize;
    let lo = sorted_times[i] as f64;
    let hi = sorted_times[i + 1] as f64;
    lo + (hi - lo) * d
}
