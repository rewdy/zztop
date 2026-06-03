use crate::datafile::Entry;
use std::path::Path;

const HOUR: u64 = 3_600;
const DAY: u64 = 86_400;
const WEEK: u64 = 604_800;

pub fn frecency(rank: f64, age_seconds: u64) -> f64 {
    if age_seconds < HOUR {
        rank * 4.0
    } else if age_seconds < DAY {
        rank * 2.0
    } else if age_seconds < WEEK {
        rank * 0.5
    } else {
        rank * 0.25
    }
}

pub fn top_n(entries: &[Entry], now: u64, n: usize) -> Vec<&Entry> {
    let mut scored: Vec<(f64, &Entry)> = entries
        .iter()
        .map(|e| {
            let age = now.saturating_sub(e.time);
            (frecency(e.rank, age), e)
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    scored
        .into_iter()
        .filter(|(_, e)| Path::new(&e.path).exists())
        .map(|(_, e)| e)
        .take(n)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn entry(path: &str, rank: f64, time: u64) -> Entry {
        Entry {
            path: PathBuf::from(path),
            rank,
            time,
        }
    }

    #[test]
    fn frecency_threshold_boost_within_hour() {
        // 30 min ago, rank 1.0 -> score 4.0
        // 2 days ago, rank 4.0 -> score 1.0 (4 * 0.25 because past WEEK boundary? no, 2 days < WEEK)
        // 2 days = 172_800 seconds, still < 604_800 (a week), so * 0.5 -> 2.0
        let recent = frecency(1.0, 30 * 60);
        let old = frecency(4.0, 2 * DAY);
        assert!(recent > old, "expected recent={recent} > old={old}");
    }

    #[test]
    fn frecency_thresholds_match_zsh_z() {
        assert_eq!(frecency(10.0, 0), 40.0);
        assert_eq!(frecency(10.0, HOUR - 1), 40.0);
        assert_eq!(frecency(10.0, HOUR), 20.0);
        assert_eq!(frecency(10.0, DAY - 1), 20.0);
        assert_eq!(frecency(10.0, DAY), 5.0);
        assert_eq!(frecency(10.0, WEEK - 1), 5.0);
        assert_eq!(frecency(10.0, WEEK), 2.5);
        assert_eq!(frecency(10.0, WEEK * 1000), 2.5);
    }

    #[test]
    fn top_n_returns_at_most_n() {
        let dir = tempfile::tempdir().unwrap();
        let p1 = dir.path().join("a");
        let p2 = dir.path().join("b");
        let p3 = dir.path().join("c");
        for p in [&p1, &p2, &p3] {
            std::fs::create_dir(p).unwrap();
        }
        let entries = vec![
            entry(p1.to_str().unwrap(), 1.0, 0),
            entry(p2.to_str().unwrap(), 2.0, 0),
            entry(p3.to_str().unwrap(), 3.0, 0),
        ];
        let now = 100;
        let top = top_n(&entries, now, 2);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn top_n_returns_fewer_when_input_smaller() {
        let dir = tempfile::tempdir().unwrap();
        let p1 = dir.path().join("a");
        std::fs::create_dir(&p1).unwrap();
        let entries = vec![entry(p1.to_str().unwrap(), 1.0, 0)];
        let top = top_n(&entries, 100, 5);
        assert_eq!(top.len(), 1);
    }

    #[test]
    fn top_n_filters_non_existent_paths() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        std::fs::create_dir(&real).unwrap();
        let entries = vec![
            entry("/definitely/does/not/exist/zzz", 100.0, 0),
            entry(real.to_str().unwrap(), 1.0, 0),
        ];
        let top = top_n(&entries, 100, 5);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].path, real);
    }

    #[test]
    fn top_n_filters_after_sort_so_high_ranked_missing_does_not_crowd_out_real() {
        // Sanity: a high-ranked missing path should not push a lower-ranked real path off the list.
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        std::fs::create_dir(&real).unwrap();
        let entries = vec![
            entry("/missing/very/high/rank", 1000.0, 100),
            entry(real.to_str().unwrap(), 1.0, 100),
        ];
        let top = top_n(&entries, 200, 1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].path, real);
    }

    #[test]
    fn top_n_orders_by_frecency_score() {
        let dir = tempfile::tempdir().unwrap();
        let recent_low = dir.path().join("recent");
        let old_high = dir.path().join("old");
        std::fs::create_dir(&recent_low).unwrap();
        std::fs::create_dir(&old_high).unwrap();
        // 'now' = 1000. recent visited at 950 (50s ago) with rank 1, score = 4.0
        // old visited at 0 (1000s ago, > 1 hour? no, 1000s < 3600s) with rank 5, score = 20
        // Need bigger gap: put old at age > week with rank 5.
        // Use now = WEEK * 2 = 1_209_600. recent at WEEK*2 - 100 (100s ago, rank 1, score 4).
        // old at 0 (age = WEEK*2, > WEEK, score = rank * 0.25 = 5 * 0.25 = 1.25).
        // recent (4.0) > old (1.25).
        let now = WEEK * 2;
        let entries = vec![
            entry(recent_low.to_str().unwrap(), 1.0, now - 100),
            entry(old_high.to_str().unwrap(), 5.0, 0),
        ];
        let top = top_n(&entries, now, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].path, recent_low, "recent low-rank should outrank stale high-rank");
        assert_eq!(top[1].path, old_high);
    }

    #[test]
    fn top_n_handles_future_timestamps_via_saturating_sub() {
        // If a clock skew put time > now, age would underflow. We use saturating_sub.
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a");
        std::fs::create_dir(&p).unwrap();
        let entries = vec![entry(p.to_str().unwrap(), 1.0, 9_999_999_999)];
        let top = top_n(&entries, 100, 5);
        assert_eq!(top.len(), 1);
    }
}
