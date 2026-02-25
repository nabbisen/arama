/// [0, zone_secs] の範囲に n 点を端点なしで均等配置する
///
/// 例: n=6, zone=120 → step=17.1 → [17.1, 34.3, 51.4, 68.6, 85.7, 102.9]
pub fn uniform_timestamps(n: usize, zone_secs: f64) -> Vec<f64> {
    if n == 0 || zone_secs <= 0.0 {
        return vec![];
    }
    let step = zone_secs / (n + 1) as f64;
    (1..=n).map(|i| step * i as f64).collect()
}

/// ソート済みリストから min_gap より近い後続点を除去する
pub fn deduplicate_by_gap(sorted: Vec<f64>, min_gap: f64) -> Vec<f64> {
    let mut result: Vec<f64> = Vec::new();
    for t in sorted {
        if result.last().map_or(true, |&last| t - last >= min_gap) {
            result.push(t);
        }
    }
    result
}
