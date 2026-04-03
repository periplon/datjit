use datjit_core::model::decorator::Distribution;
use rand::Rng;
use rand_distr::Distribution as RandDistribution;

/// Sample a single f64 from the given distribution, optionally clamped to a range.
pub fn sample_distribution(
    dist: &Distribution,
    range: Option<(f64, f64)>,
    rng: &mut impl Rng,
) -> f64 {
    let raw = match dist {
        Distribution::Uniform => {
            let (lo, hi) = range.unwrap_or((0.0, 1000.0));
            rng.gen_range(lo..=hi)
        }
        Distribution::Normal { mu, sigma } => {
            let d = rand_distr::Normal::new(*mu, *sigma).unwrap();
            d.sample(rng)
        }
        Distribution::LogNormal { mu, sigma } => {
            let d = rand_distr::LogNormal::new(*mu, *sigma).unwrap();
            d.sample(rng)
        }
        Distribution::Exponential { lambda } => {
            let d = rand_distr::Exp::new(*lambda).unwrap();
            d.sample(rng)
        }
        Distribution::Geometric { p } => {
            let d = rand_distr::Geometric::new(*p).unwrap();
            d.sample(rng) as f64
        }
        Distribution::Zipf { s } => {
            let d = rand_distr::Zipf::new(1000u64, *s).unwrap();
            d.sample(rng)
        }
        Distribution::Bimodal { peaks } => {
            let (p1, p2) = peaks;
            let spread = ((p2 - p1).abs() / 6.0).max(0.1);
            if rng.gen_bool(0.5) {
                let d = rand_distr::Normal::new(*p1, spread).unwrap();
                d.sample(rng)
            } else {
                let d = rand_distr::Normal::new(*p2, spread).unwrap();
                d.sample(rng)
            }
        }
        Distribution::Categorical(probs) => {
            let total: f64 = probs.iter().sum();
            let mut roll = rng.gen_range(0.0..total);
            for (i, prob) in probs.iter().enumerate() {
                roll -= prob;
                if roll <= 0.0 {
                    return i as f64;
                }
            }
            (probs.len() - 1) as f64
        }
        Distribution::Weighted(entries) => {
            let total: f64 = entries.iter().map(|(_, w)| w).sum();
            let mut roll = rng.gen_range(0.0..total);
            for (i, (_, w)) in entries.iter().enumerate() {
                roll -= w;
                if roll <= 0.0 {
                    return i as f64;
                }
            }
            (entries.len() - 1) as f64
        }
    };

    // Clamp to range for continuous distributions (Categorical/Weighted already returned)
    match dist {
        Distribution::Categorical(_) | Distribution::Weighted(_) | Distribution::Uniform => raw,
        _ => {
            if let Some((lo, hi)) = range {
                raw.clamp(lo, hi)
            } else {
                raw
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_uniform_default_range() {
        let mut r = rng();
        for _ in 0..100 {
            let v = sample_distribution(&Distribution::Uniform, None, &mut r);
            assert!((0.0..=1000.0).contains(&v));
        }
    }

    #[test]
    fn test_uniform_custom_range() {
        let mut r = rng();
        for _ in 0..100 {
            let v = sample_distribution(&Distribution::Uniform, Some((10.0, 20.0)), &mut r);
            assert!((10.0..=20.0).contains(&v));
        }
    }

    #[test]
    fn test_normal_mean_stddev() {
        let mut r = rng();
        let n = 10_000;
        let mu = 100.0;
        let sigma = 15.0;
        let samples: Vec<f64> = (0..n)
            .map(|_| sample_distribution(&Distribution::Normal { mu, sigma }, None, &mut r))
            .collect();

        let mean = samples.iter().sum::<f64>() / n as f64;
        let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n as f64 - 1.0);
        let stddev = variance.sqrt();

        assert!((mean - mu).abs() < 1.5, "mean {mean} too far from {mu}");
        assert!(
            (stddev - sigma).abs() < 2.0,
            "stddev {stddev} too far from {sigma}"
        );
    }

    #[test]
    fn test_lognormal_mean() {
        let mut r = rng();
        let n = 10_000;
        let mu = 2.0;
        let sigma = 0.5;
        let samples: Vec<f64> = (0..n)
            .map(|_| sample_distribution(&Distribution::LogNormal { mu, sigma }, None, &mut r))
            .collect();

        let mean = samples.iter().sum::<f64>() / n as f64;
        // Expected mean of LogNormal = exp(mu + sigma^2/2)
        let expected_mean = (mu + sigma * sigma / 2.0).exp();
        assert!(
            (mean - expected_mean).abs() < expected_mean * 0.1,
            "mean {mean} too far from expected {expected_mean}"
        );

        // All values should be positive
        assert!(samples.iter().all(|v| *v > 0.0));
    }

    #[test]
    fn test_exponential() {
        let mut r = rng();
        let lambda = 0.5;
        let samples: Vec<f64> = (0..10_000)
            .map(|_| sample_distribution(&Distribution::Exponential { lambda }, None, &mut r))
            .collect();
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        // Expected mean = 1/lambda = 2.0
        assert!(
            (mean - 2.0).abs() < 0.2,
            "exponential mean {mean} not close to 2.0"
        );
    }

    #[test]
    fn test_geometric() {
        let mut r = rng();
        let v = sample_distribution(&Distribution::Geometric { p: 0.5 }, None, &mut r);
        assert!(v >= 0.0);
    }

    #[test]
    fn test_zipf() {
        let mut r = rng();
        for _ in 0..100 {
            let v = sample_distribution(&Distribution::Zipf { s: 1.5 }, None, &mut r);
            assert!(v >= 1.0);
        }
    }

    #[test]
    fn test_bimodal() {
        let mut r = rng();
        let samples: Vec<f64> = (0..10_000)
            .map(|_| {
                sample_distribution(
                    &Distribution::Bimodal {
                        peaks: (20.0, 80.0),
                    },
                    None,
                    &mut r,
                )
            })
            .collect();
        let near_20 = samples.iter().filter(|v| (**v - 20.0).abs() < 15.0).count();
        let near_80 = samples.iter().filter(|v| (**v - 80.0).abs() < 15.0).count();
        // Both peaks should attract a significant portion
        assert!(near_20 > 2000, "too few near peak 20: {near_20}");
        assert!(near_80 > 2000, "too few near peak 80: {near_80}");
    }

    #[test]
    fn test_categorical_proportions() {
        let mut r = rng();
        let probs = vec![50.0, 30.0, 20.0];
        let n = 10_000;
        let mut counts = vec![0usize; 3];
        for _ in 0..n {
            let idx = sample_distribution(&Distribution::Categorical(probs.clone()), None, &mut r)
                as usize;
            counts[idx] += 1;
        }
        let p0 = counts[0] as f64 / n as f64;
        let p1 = counts[1] as f64 / n as f64;
        let p2 = counts[2] as f64 / n as f64;
        assert!((p0 - 0.50).abs() < 0.05, "p0={p0}");
        assert!((p1 - 0.30).abs() < 0.05, "p1={p1}");
        assert!((p2 - 0.20).abs() < 0.05, "p2={p2}");
    }

    #[test]
    fn test_weighted() {
        let mut r = rng();
        let entries = vec![("high".to_string(), 70.0), ("low".to_string(), 30.0)];
        let n = 10_000;
        let mut counts = vec![0usize; 2];
        for _ in 0..n {
            let idx = sample_distribution(&Distribution::Weighted(entries.clone()), None, &mut r)
                as usize;
            counts[idx] += 1;
        }
        let p0 = counts[0] as f64 / n as f64;
        assert!((p0 - 0.70).abs() < 0.05, "p0={p0}");
    }

    #[test]
    fn test_normal_with_range_clamp() {
        let mut r = rng();
        for _ in 0..1000 {
            let v = sample_distribution(
                &Distribution::Normal {
                    mu: 50.0,
                    sigma: 100.0,
                },
                Some((0.0, 100.0)),
                &mut r,
            );
            assert!((0.0..=100.0).contains(&v), "value {v} out of range");
        }
    }
}
