use super::*;

const INTERVALS: PollIntervals = PollIntervals {
    link: Duration::from_secs(10),
    battery: Duration::from_secs(60),
};

fn switched_on(now: Instant) -> Polling {
    let mut polling = Polling::new(INTERVALS);
    polling.set(true, now);

    polling
}

#[test]
fn default_intervals_are_the_agreed_ones() {
    assert_eq!(PollIntervals::default(), INTERVALS);
}

#[test]
fn nothing_is_due_while_off() {
    let polling = Polling::new(INTERVALS);

    let due = polling.due(Instant::now() + INTERVALS.battery);

    assert_eq!(
        due,
        Due {
            link: false,
            battery: false
        }
    );
}

#[test]
fn reschedule_advances_only_what_was_due() {
    let start = Instant::now();
    let mut polling = switched_on(start);
    let first_tick = start + INTERVALS.link;

    let first = polling.due(first_tick);
    polling.reschedule(first, first_tick);
    let second = polling.due(first_tick);

    assert_eq!(
        first,
        Due {
            link: true,
            battery: false
        }
    );
    assert_eq!(
        second,
        Due {
            link: false,
            battery: false
        }
    );
}

#[test]
fn reschedule_counts_from_the_end_of_the_work() {
    let start = Instant::now();
    let mut polling = switched_on(start);
    let tick = start + INTERVALS.link;
    let work_done = tick + 2 * INTERVALS.link;

    let due = polling.due(tick);
    polling.reschedule(due, work_done);

    assert!(!polling.due(work_done).link);
    assert!(polling.due(work_done + INTERVALS.link).link);
}

#[test]
fn switching_on_twice_keeps_the_schedule() {
    let start = Instant::now();
    let mut polling = switched_on(start);

    polling.set(true, start + INTERVALS.link);

    assert!(polling.due(start + INTERVALS.link).link);
}

#[test]
fn restart_counts_from_now_and_leaves_off_alone() {
    let start = Instant::now();
    let later = start + INTERVALS.battery;
    let mut on = switched_on(start);
    let mut off = Polling::new(INTERVALS);

    on.restart(later);
    off.restart(later);

    assert!(!on.due(later).battery);
    assert!(on.due(later + INTERVALS.battery).battery);
    assert!(!off.due(later + INTERVALS.battery).battery);
}

#[test]
fn identify_retry_waits_the_slow_interval() {
    let polling = Polling::new(INTERVALS);
    let now = Instant::now();

    let retry_at = polling.identify_retry_at(now);

    assert_eq!(retry_at, now + INTERVALS.battery);
}
