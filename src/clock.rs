use std::{ops::AddAssign, time::Duration};

pub trait Clock {
    type Instant: AddAssign<Duration>;

    fn now(&mut self) -> Self::Instant;

    fn elapsed(&mut self, instant: &Self::Instant) -> Duration;
}

pub struct StdClock;

impl Clock for StdClock {
    type Instant = std::time::Instant;

    fn now(&mut self) -> Self::Instant {
        std::time::Instant::now()
    }

    fn elapsed(&mut self, instant: &Self::Instant) -> Duration {
        instant.elapsed()
    }
}

#[cfg(test)]
pub mod fixed_clock {
    use super::*;

    /// A clock that steps in the given increments.
    /// Once the given increments are exhausted then it always
    /// jumps 24 hours at a time.
    pub struct FixedClock {
        increments: Vec<Duration>,
    }

    impl FixedClock {
        pub fn new(mut increments: Vec<Duration>) -> Self {
            // Reverse the input so the `pop` will bring them
            // out in the right order.
            increments.reverse();

            Self { increments }
        }
    }

    impl Clock for FixedClock {
        type Instant = std::time::Instant;

        fn now(&mut self) -> Self::Instant {
            std::time::Instant::now()
        }

        fn elapsed(&mut self, _instant: &Self::Instant) -> Duration {
            self.increments.pop().unwrap_or(Duration::from_hours(24))
        }
    }
}
