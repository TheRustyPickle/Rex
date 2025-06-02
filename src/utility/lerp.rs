use std::collections::HashMap;
use std::time::Instant;

struct LerpValue {
    start: f64,
    current: f64,
    target: f64,
    start_time: Instant,
    active: bool,
}

#[derive(Default)]
pub struct LerpState {
    values: HashMap<String, LerpValue>,
    duration_secs: f64,
    active_lerp_count: usize,
}

impl LerpState {
    #[must_use]
    pub fn new(duration_secs: f64) -> Self {
        Self {
            values: HashMap::new(),
            duration_secs,
            active_lerp_count: 0,
        }
    }

    pub fn lerp(&mut self, id: &str, new_target: f64) -> f64 {
        let now = Instant::now();

        let entry = self.values.entry(id.to_string()).or_insert_with(|| {
            self.active_lerp_count += 1;
            LerpValue {
                start: 0.0,
                current: 0.0,
                target: new_target,
                start_time: now,
                active: true,
            }
        });

        // Start a new lerp if the target has changed
        if (entry.target - new_target).abs() > f64::EPSILON {
            if !entry.active {
                entry.active = true;
                self.active_lerp_count += 1;
            }
            entry.start = entry.current;
            entry.target = new_target;
            entry.start_time = now;
        }

        let t = (now - entry.start_time).as_secs_f64() / self.duration_secs;
        let clamped_t = t.min(1.0);

        entry.current = entry.start + (entry.target - entry.start) * clamped_t;

        // Check if lerp has completed
        if clamped_t >= 1.0 && entry.active {
            entry.current = entry.target;
            entry.active = false;
            self.active_lerp_count = self.active_lerp_count.saturating_sub(1);
        }

        entry.current
    }

    #[must_use]
    pub fn has_active_lerps(&self) -> bool {
        self.active_lerp_count > 0
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.active_lerp_count = 0;
    }
}
