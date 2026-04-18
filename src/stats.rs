use std::collections::BTreeMap;

#[derive(Debug, Default)]

pub struct Stats{
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

    pub fn print_summary(&self){
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