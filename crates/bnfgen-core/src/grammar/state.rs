use crate::grammar::alt::AltId;
use rand::Rng;
use std::collections::HashMap;

pub struct State<R: Rng> {
    rng: R,
    /// tracking the number of times an alternative has been selected
    /// Notes: only those with invoke limits are tracked
    pub(crate) tracking: HashMap<AltId, usize>,
    /// total number of attempts (i.e., the sum of counts for all tracked alternatives)
    attempts: usize,
}

impl<R: Rng> State<R> {
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            tracking: HashMap::new(),
            attempts: 0,
        }
    }

    pub fn rng(&mut self) -> &mut R {
        &mut self.rng
    }

    pub fn track(&mut self, id: AltId) {
        let count = self.tracking.entry(id).or_insert(0);
        *count += 1;
    }

    pub fn count(&self, id: AltId) -> usize {
        *self.tracking.get(&id).unwrap_or(&0)
    }

    /// returns the total number of attempts
    pub fn total_attempts(&self) -> usize {
        self.attempts
    }

    pub fn make_attempt(&mut self) {
        self.attempts += 1;
    }
}
