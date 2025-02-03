use std::time::{Duration, Instant};

pub fn instant_from_millis(millis: u64) -> Instant {
    let duration = Duration::from_millis(millis);
    Instant::now() - duration
}

pub fn is_after(what: Instant, than: Instant) -> bool {
    what > than && what - than > Duration::from_millis(5)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::time::is_after;

    #[test]
    fn test_is_after() {
        assert!(is_after(Instant::now(), Instant::now() - Duration::from_secs(1)));
        assert!(!is_after(Instant::now(), Instant::now() + Duration::from_secs(1)));

        let now = Instant::now();

        assert!(!is_after(now, now));
    }
}
