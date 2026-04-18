#[derive(Debug, Default)]

pub struct Stats{
    pub info: u64,
    pub warn: u64,
    pub error: u64,
    pub malformed: u64
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            info: 0,
            warn: 0,
            error: 0,
            malformed: 0,
        }
    }

    pub fn print_summary(&self){
        println!("INFO: {}", self.info);
        println!("WARN: {}", self.warn);
        println!("ERROR: {}", self.error);

        if self.malformed > 0 {
            println!("MALFORMED : {}", self.malformed);
        }
    }

    #[allow(dead_code)]
    pub fn total(&self) -> u64 {
        self.info + self.warn + self.error + self.malformed
    }
}