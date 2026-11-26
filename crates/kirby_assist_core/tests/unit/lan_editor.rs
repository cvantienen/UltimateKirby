use super::*;

fn endpoint() -> Endpoint {
    Endpoint::parse("192.168.1.20", 4242).unwrap()
}

fn observation() -> Observation {
    Observation {
        observed_at: Duration::from_secs(1),
        training_room: Some(7),
        training_menu_open: true,
        route: Some(DirectRoute {
            interface: 1,
            local_address: Ipv4Addr::new(192, 168, 1, 10),
            prefix_length: 24,
            destination: endpoint().address(),
            uses_gateway: false,
            interface_is_lan: true,
        }),
    }
}

#[test]
fn rejects_non_numeric_non_private_and_zero_port_destinations() {
    for address in [
        "computer.local",
        "https://192.168.1.20",
        "192.168.1.20:4242",
        "127.0.0.1",
        "169.254.1.2",
        "8.8.8.8",
        "100.64.1.2",
        "0.0.0.0",
        "224.0.0.1",
        "255.255.255.255",
        "::1",
        "::ffff:192.168.1.20",
    ] {
        assert!(Endpoint::parse(address, 4242).is_err(), "{address}");
    }
    assert!(Endpoint::parse("192.168.1.20", 0).is_err());
}

#[test]
fn rejects_unverified_routed_non_lan_and_invalid_subnet_facts() {
    let base = observation();
    let route = base.route.unwrap();
    let mut invalid = vec![Observation {
        route: None,
        ..base
    }];
    for route in [
        DirectRoute {
            uses_gateway: true,
            ..route
        },
        DirectRoute {
            interface_is_lan: false,
            ..route
        },
        DirectRoute {
            destination: Ipv4Addr::new(192, 168, 1, 21),
            ..route
        },
        DirectRoute {
            local_address: Ipv4Addr::new(192, 168, 2, 10),
            ..route
        },
        DirectRoute {
            local_address: endpoint().address(),
            ..route
        },
        DirectRoute {
            local_address: Ipv4Addr::new(192, 168, 1, 0),
            ..route
        },
        DirectRoute {
            local_address: Ipv4Addr::new(192, 168, 1, 255),
            ..route
        },
        DirectRoute {
            prefix_length: 0,
            ..route
        },
        DirectRoute {
            prefix_length: 15,
            ..route
        },
        DirectRoute {
            prefix_length: 31,
            ..route
        },
        DirectRoute {
            prefix_length: 32,
            ..route
        },
        DirectRoute {
            prefix_length: 255,
            ..route
        },
    ] {
        invalid.push(Observation {
            route: Some(route),
            ..base
        });
    }
    for facts in invalid {
        assert!(Guard::default()
            .activate(endpoint(), facts, base.observed_at)
            .is_err());
    }
    for address in ["192.168.1.0", "192.168.1.255"] {
        let peer = Endpoint::parse(address, 4242).unwrap();
        let facts = Observation {
            route: Some(DirectRoute {
                destination: peer.address(),
                ..route
            }),
            ..base
        };
        assert!(Guard::default()
            .activate(peer, facts, base.observed_at)
            .is_err());
    }
}

#[test]
fn requires_explicit_menu_activation_and_edit_menu_presence() {
    let mut guard = Guard::default();
    let facts = observation();
    let closed = Observation {
        training_menu_open: false,
        ..facts
    };
    assert!(guard
        .activate(endpoint(), closed, facts.observed_at)
        .is_err());
    assert!(guard
        .activate(
            endpoint(),
            Observation {
                training_room: None,
                ..facts
            },
            facts.observed_at
        )
        .is_err());
    let ticket = guard
        .activate(endpoint(), facts, facts.observed_at)
        .unwrap();
    assert!(guard.allows_io(ticket, closed, facts.observed_at));
    assert!(!guard.allows_edit(ticket, closed, facts.observed_at));
    assert!(guard.allows_edit(ticket, facts, facts.observed_at));
}

#[test]
fn training_exit_and_route_changes_revoke_without_reconnecting() {
    let facts = observation();
    for changed in [
        Observation {
            training_room: None,
            ..facts
        },
        Observation {
            training_room: Some(8),
            ..facts
        },
        Observation {
            route: None,
            ..facts
        },
        Observation {
            route: Some(DirectRoute {
                interface: 2,
                ..facts.route.unwrap()
            }),
            ..facts
        },
        Observation {
            route: Some(DirectRoute {
                local_address: Ipv4Addr::new(192, 168, 1, 11),
                ..facts.route.unwrap()
            }),
            ..facts
        },
    ] {
        let mut guard = Guard::default();
        let ticket = guard
            .activate(endpoint(), facts, facts.observed_at)
            .unwrap();
        assert!(!guard.allows_io(ticket, changed, facts.observed_at));
        assert!(!guard.allows_io(ticket, facts, facts.observed_at));
        let new_ticket = guard
            .activate(endpoint(), facts, facts.observed_at)
            .unwrap();
        assert_ne!(ticket, new_ticket);
        assert!(guard.allows_io(new_ticket, facts, facts.observed_at));
    }
}

#[test]
fn stale_future_and_out_of_order_observations_revoke() {
    let facts = observation();
    for now in [
        facts.observed_at - Duration::from_millis(1),
        facts.observed_at + Duration::from_millis(100),
    ] {
        assert!(Guard::default().activate(endpoint(), facts, now).is_err());
        let mut guard = Guard::default();
        let ticket = guard
            .activate(endpoint(), facts, facts.observed_at)
            .unwrap();
        assert!(!guard.allows_io(ticket, facts, now));
        assert!(!guard.allows_io(ticket, facts, facts.observed_at));
    }
    let mut guard = Guard::default();
    let ticket = guard
        .activate(endpoint(), facts, facts.observed_at)
        .unwrap();
    let later = Observation {
        observed_at: facts.observed_at + Duration::from_millis(1),
        ..facts
    };
    assert!(guard.allows_io(ticket, later, later.observed_at));
    assert!(!guard.allows_io(ticket, facts, later.observed_at));
}

#[test]
fn disconnect_and_failed_activation_invalidate_existing_tickets() {
    let facts = observation();
    let mut guard = Guard::default();
    let first = guard
        .activate(endpoint(), facts, facts.observed_at)
        .unwrap();
    guard.revoke();
    assert!(!guard.allows_io(first, facts, facts.observed_at));
    let second = guard
        .activate(endpoint(), facts, facts.observed_at)
        .unwrap();
    assert_ne!(first, second);
    assert!(guard
        .activate(
            endpoint(),
            Observation {
                training_menu_open: false,
                ..facts
            },
            facts.observed_at
        )
        .is_err());
    assert!(!guard.allows_io(second, facts, facts.observed_at));
}

#[test]
fn admits_each_private_range_only_on_a_direct_subnet() {
    for (local, peer, prefix) in [
        ("10.1.2.10", "10.1.2.20", 24),
        ("172.16.2.10", "172.16.2.20", 24),
        ("192.168.1.10", "192.168.1.20", 24),
    ] {
        let peer = Endpoint::parse(peer, 4242).unwrap();
        let facts = Observation {
            route: Some(DirectRoute {
                local_address: local.parse().unwrap(),
                destination: peer.address(),
                prefix_length: prefix,
                ..observation().route.unwrap()
            }),
            ..observation()
        };
        let mut guard = Guard::default();
        let ticket = guard.activate(peer, facts, facts.observed_at).unwrap();
        assert!(guard.allows_edit(ticket, facts, facts.observed_at));
    }
}
