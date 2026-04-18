use std::collections::BTreeMap;

#[derive(Debug, Default)]

pub struct Stats {
    pub info: u64,
    pub warn: u64,
    pub error: u64,
    pub malformed: u64,
    pub malformed_by_reason: BTreeMap<&'static str, u64>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            info: 0,
            warn: 0,
            error: 0,
            malformed: 0,
            malformed_by_reason: BTreeMap::new(),
        }
    }

    pub fn record_malformed(&mut self, reason: &'static str) {
        self.malformed += 1;
        *self.malformed_by_reason.entry(reason).or_insert(0) += 1;
    }

    pub fn merge(&mut self, other: Stats) {
        self.info += other.info;
        self.warn += other.warn;
        self.error += other.error;
        self.malformed += other.malformed;

        for (reason, count) in other.malformed_by_reason {
            *self.malformed_by_reason.entry(reason).or_insert(0) += count;
        }
    }

    pub fn print_summary(&self) {
        println!("INFO: {}", self.info);
        println!("WARN: {}", self.warn);
        println!("ERROR: {}", self.error);

        if self.malformed > 0 {
            println!("MALFORMED: {}", self.malformed);
            println!("MALFORMED DETAILS:");
            for (reason, count) in &self.malformed_by_reason {
                println!("  - {}: {}", reason, count);
            }
        }
    }

    #[allow(dead_code)]
    pub fn total(&self) -> u64 {
        self.info + self.warn + self.error + self.malformed
    }
}

#[cfg(test)]
mod tests {
    use super::Stats;

    #[test]
    fn new_stats_start_at_zero() {
        let stats = Stats::new();
        assert_eq!(stats.info, 0);
        assert_eq!(stats.warn, 0);
        assert_eq!(stats.error, 0);
        assert_eq!(stats.malformed, 0);
        assert!(stats.malformed_by_reason.is_empty());
    }

    #[test]
    fn record_malformed_tracks_total_and_reason_count() {
        let mut stats = Stats::new();
        stats.record_malformed("unknown level");
        stats.record_malformed("unknown level");
        stats.record_malformed("too few fields");

        assert_eq!(stats.malformed, 3);
        assert_eq!(stats.malformed_by_reason.get("unknown level"), Some(&2));
        assert_eq!(stats.malformed_by_reason.get("too few fields"), Some(&1));
    }

    #[test]
    fn merge_combines_all_counters() {
        let mut a = Stats::new();
        a.info = 2;
        a.warn = 1;
        a.record_malformed("unknown level");

        let mut b = Stats::new();
        b.warn = 3;
        b.error = 4;
        b.record_malformed("unknown level");
        b.record_malformed("too few fields");

        a.merge(b);

        assert_eq!(a.info, 2);
        assert_eq!(a.warn, 4);
        assert_eq!(a.error, 4);
        assert_eq!(a.malformed, 3);
        assert_eq!(a.malformed_by_reason.get("unknown level"), Some(&2));
        assert_eq!(a.malformed_by_reason.get("too few fields"), Some(&1));
    }

    #[test]
    fn total_includes_malformed() {
        let mut stats = Stats::new();
        stats.info = 5;
        stats.warn = 1;
        stats.error = 2;
        stats.record_malformed("empty line");

        assert_eq!(stats.total(), 9);
    }
}
