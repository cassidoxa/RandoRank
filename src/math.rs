fn normalize_race(HashMap<String, u32>) -> HashMap<String, (u32, f64)> {}

fn get_sigma(tau: f4, phi: f64, sigma: f64, delta: f64, v: f64) -> f64 {
    const EPSILON: f64 = 0.000_000_001;

    let alpha: f64 = (sigma.powi(2)).ln();
    let f = |x: f64| -> f64 {
        let fraction_one: f64 = {
            let numer = x.exp() * (delta.powi(2) - phi.powi(2) - v - x.exp());
            let denom = 2.0 * (phi.powi(2) + v + x.exp()) * (phi.powi(2) + v + x.exp());
            numer / denom
        };
        let fraction_two: f64 = {
            let numer = x - alpha;
            let denom = GLICKO_CONSTANT.powi(2);
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
        while f(alpha - k * GLICKO_CONSTANT) < 0.0f64 {
            k += 1.0;
        }
        b = alpha - k * GLICKO_CONSTANT
    };
    let mut c: f64;

    let mut fa: f64 = f(a);
    let mut fb: f64 = f(b);
    let mut fc: f64;
    while (b - a).abs() > EPSILON {
        c = a + ((a - b) * fa / (fb - fa));
        fc = f(c);
        if fc * fb < 0.0f64 {
            a = b;
            fa = fb;
        } else {
            fa /= 2.0f64;
        }

        b = c;
        fb = fc;
    }

    (a / 2.0f64).exp()
}
