use proptest::prelude::*;
use pantheon_kernel::cell::CellState;
use pantheon_kernel::capability::{Action, ResourcePattern, CapabilityConstraints, CapabilityGrant};
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::lamport::LamportClock;
use pantheon_kernel::types::LamportTimestamp;

// ── Cell State Machine ─────────────────────────────────────────────────

#[test]
fn cell_state_transitions_are_deterministic() {
    let all_states = vec![
        CellState::Null, CellState::Created, CellState::Running,
        CellState::Suspended, CellState::Blocked, CellState::Terminated,
        CellState::Retired, CellState::Destroyed,
    ];
    for s1 in &all_states {
        for s2 in &all_states {
            let r1 = s1.can_transition_to(s2);
            let r2 = s1.can_transition_to(s2);
            assert_eq!(r1, r2);
        }
    }
}

#[test]
fn terminal_states_have_limited_transitions() {
    assert!(CellState::Terminated.can_transition_to(&CellState::Retired));
    assert!(CellState::Retired.can_transition_to(&CellState::Destroyed));
    assert!(CellState::Destroyed.can_transition_to(&CellState::Created));
    assert!(!CellState::Terminated.can_transition_to(&CellState::Running));
    assert!(!CellState::Retired.can_transition_to(&CellState::Running));
}

#[test]
fn only_created_is_reachable_from_null() {
    assert!(CellState::Null.can_transition_to(&CellState::Created));
    for state in &[
        CellState::Running, CellState::Suspended, CellState::Blocked,
        CellState::Terminated, CellState::Retired, CellState::Destroyed,
    ] {
        assert!(!CellState::Null.can_transition_to(state));
    }
}

#[test]
fn active_states_are_not_terminal() {
    let all = vec![
        CellState::Null, CellState::Created, CellState::Running,
        CellState::Suspended, CellState::Blocked, CellState::Terminated,
        CellState::Retired, CellState::Destroyed,
    ];
    for state in &all {
        assert!(!(state.is_active() && state.is_terminal()));
    }
}

// ── Lamport Clock ──────────────────────────────────────────────────────

#[test]
fn clock_tick_always_increases() {
    proptest!(ProptestConfig::with_cases(1000), |(initial in 0..10000u64)| {
        let clock = LamportClock::new(initial);
        let t1 = clock.tick();
        let t2 = clock.tick();
        prop_assert!(t2.value() > t1.value());
        prop_assert!(t1.value() > initial);
    });
}

#[test]
fn clock_observe_never_goes_backward() {
    proptest!(ProptestConfig::with_cases(1000), |(clock_initial in 0..10000u64, observed in 0..10000u64)| {
        let clock = LamportClock::new(clock_initial);
        let result = clock.observe(LamportTimestamp::new(observed));
        prop_assert!(result.value() > clock_initial);
        prop_assert_eq!(clock.peek(), result);
    });
}

// ── Resource Patterns ──────────────────────────────────────────────────

#[test]
fn resource_pattern_matches_self() {
    proptest!(ProptestConfig::with_cases(500), |(pattern in "[a-z]{0,20}")| {
        let rp = ResourcePattern::new(pattern.clone());
        prop_assert!(rp.matches(&pattern));
    });
}

#[test]
fn wildcard_matches_any_prefix() {
    proptest!(ProptestConfig::with_cases(500), |(prefix in "[a-z]{0,10}", suffix in "[a-z]{0,10}")| {
        let pattern = ResourcePattern::new(format!("{}*", prefix));
        let matched = pattern.matches(&format!("{}{}", prefix, suffix));
        prop_assert!(matched);
    });
}

#[test]
fn resource_subset_is_reflexive() {
    proptest!(ProptestConfig::with_cases(500), |(pattern in "[a-z]{0,20}")| {
        let rp = ResourcePattern::new(pattern.clone());
        prop_assert!(rp.is_subset_of(&rp));
    });
}

#[test]
fn non_wildcard_does_not_match_different() {
    proptest!(ProptestConfig::with_cases(500), |(pattern in "[a-z]{0,20}")| {
        let rp = ResourcePattern::new(pattern.clone());
        if !pattern.ends_with('*') {
            let different = format!("{}x", pattern);
            if different != pattern {
                prop_assert!(!rp.matches(&different));
            }
        }
    });
}

// ── Capability Attenuation ─────────────────────────────────────────────

#[test]
fn attenuate_narrows_resource() {
    proptest!(ProptestConfig::with_cases(500), |(resource in "[a-z]{1,5}://[a-z]{1,10}/[a-z]{1,10}")| {
        let issuer = CellId::random();
        let subject = CellId::random();
        let new_subject = CellId::random();

        let prefix = &resource[..resource.len().min(5)];
        let grant = CapabilityGrant::new(
            issuer,
            subject,
            format!("{}*", prefix),
            vec![Action::Read],
            CapabilityConstraints::unlimited(),
        );

        let attenuated = grant.attenuate(
            new_subject,
            Some(ResourcePattern::new(resource.clone())),
            None,
            None,
        );

        if let Ok(att) = attenuated {
            prop_assert_eq!(att.subject, new_subject);
            prop_assert_eq!(att.issuer, subject);
        }
    });
}

// ── Attenuation rejection ──────────────────────────────────────────────

#[test]
fn attenuate_rejects_broader_resource() {
    let issuer = CellId::random();
    let subject = CellId::random();
    let new_subject = CellId::random();

    let grant = CapabilityGrant::new(
        issuer,
        subject,
        "resource://specific/path",
        vec![Action::Read],
        CapabilityConstraints::unlimited(),
    );

    let result = grant.attenuate(
        new_subject,
        Some(ResourcePattern::new("resource://*")),
        None,
        None,
    );

    assert!(result.is_err());
}

#[test]
fn attenuate_narrows_actions() {
    let issuer = CellId::random();
    let subject = CellId::random();
    let new_subject = CellId::random();

    let grant = CapabilityGrant::new(
        issuer,
        subject,
        "resource://*",
        vec![Action::Read, Action::Write, Action::Execute],
        CapabilityConstraints::unlimited(),
    );

    let attenuated = grant.attenuate(
        new_subject,
        None,
        Some(vec![Action::Read, Action::Write]),
        None,
    );

    assert!(attenuated.is_ok());
    let att = attenuated.unwrap();
    assert_eq!(att.actions, vec![Action::Read, Action::Write]);
}
