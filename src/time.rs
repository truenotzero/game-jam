use std::time::Duration;


pub struct Threshold {
    acc: Duration,
    threshold: Duration,
}

impl Threshold {
    pub fn new(threshold: Duration) -> Self {
        Self {
            acc: Duration::ZERO,
            threshold,
        }
    }

    pub fn tick(&mut self, dt: Duration) -> bool {
        self.acc += dt;
        if self.acc > self.threshold {
            self.acc -= self.threshold;
            true
        } else {
            false
        }
    }
}
