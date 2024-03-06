use std::time::Duration;

#[derive(Default)]
pub struct Threshold {
    acc: Duration,
    threshold: Duration,
}

impl Threshold {
    pub fn new(threshold: Duration) -> Self {
        let mut ret = Self::default();
        ret.set_threshold(threshold);

        ret
    }

    pub fn set_threshold(&mut self, threshold: Duration) {
        self.threshold = threshold;
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
