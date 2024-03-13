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

    pub fn reset(&mut self) {
        self.acc = Duration::ZERO;
    }

    pub fn progress(&self) -> f32 {
        self.acc.as_secs_f32() / self.threshold.as_secs_f32()
    }
}

#[derive(Default)]
pub struct Cooldown {
    acc: Duration,
    cooldown: Duration,
}

impl Cooldown {
    pub fn new(cooldown: Duration) -> Self {
        let mut ret = Self::default();
        ret.set_cooldown(cooldown);
        ret
    }

    pub fn set_cooldown(&mut self, cooldown: Duration) {
        self.cooldown = cooldown;
    }

    pub fn tick(&mut self, dt: Duration) {
        self.acc = self.acc.saturating_sub(dt);
    }

    pub fn is_cooling_down(&self) -> bool {
        !self.acc.is_zero()
    }

    pub fn reset(&mut self) {
        self.acc = Duration::ZERO;
    }

    pub fn cool_down(&mut self) {
        self.acc = self.cooldown;
    }

    pub fn progress(&self) -> f32 {
        1.0 - self.acc.as_secs_f32() / self.cooldown.as_secs_f32()
    }
}
