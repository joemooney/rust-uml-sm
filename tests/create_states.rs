extern crate rust_uml_sm;
use rust_uml_sm::StateMachine;

#[test]
fn test_create_simple_states() {
    let mut sm = StateMachine::new("sm1");
    let s1 = sm.add_state("s1").expect("Failed to add state");
    assert_eq!(s1, 2);
    let s2 = sm.add_state("s2").expect("Failed to add state");
    assert_eq!(s2, 3);
}

#[test]
fn test_create_nested_state() {
    let mut sm = StateMachine::new("sm1");
    let s1 = sm.add_state("s1").expect("Failed to add state");
    assert_eq!(s1, 2);
    let s2 = sm.add_substate("s2", s1).expect("Failed to add state");
    assert_eq!(s2, 4);
    let s3 = sm.add_substate("s3", s1).expect("Failed to add state");
    assert_eq!(s3, 5);
    let b = sm.is_contained_in(s2, s1);
    assert_eq!(b, true);
    let b = sm.is_contained_in(s1, s1);
    assert_eq!(b, false);
    let b = sm.is_contained_in(s1, s2);
    assert_eq!(b, false);
    let s4 = sm.add_substate("s4", s2).expect("Failed to add state");
    let b = sm.is_contained_in(s4, s2);
    assert_eq!(b, true);
    let b = sm.is_contained_in(s4, s1);
    assert_eq!(b, true);
    let b = sm.is_contained_in(s4, s3);
    assert_eq!(b, false);
}

#[test]
fn test_create_sm_regions() {
    let mut sm = StateMachine::new("sm1");
    let regions = vec![1];
    assert_eq!(sm.sm_regions(), regions);
    assert_eq!(sm.name(1).unwrap(), "region_1");
    sm.add_sm_region("region_x");
    assert_eq!(sm.name(1).unwrap(), "region_x");
    let region_y = sm.add_sm_region("region_y").expect("Failed to add region");
    assert_eq!(sm.name(region_y).unwrap(), "region_y");
    // sm.add_region(c: Container, region_dbid: DbId)
}

#[test]
fn test_create_region_with_states() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").expect("Failed to add region");
    println!("here1 {}", r1);
    let s1 = sm.add_substate("s1", r1).expect("Failed to add state");
    println!("here2 {}", s1);
    let s2 = sm.add_substate("s2", r1).expect("Failed to add state");
    let s3 = sm.add_substate("s3", s2).expect("Failed to add state");
    println!("here3");
    assert_eq!(sm.region(s1), Some(r1));
    assert_eq!(sm.region(s2), Some(r1));
    println!("{:#?}", sm);
    assert_eq!(sm.region(s3), Some(4));
    // sm.add_region(c: Container, region_dbid: DbId)
}

#[test]
fn test_names() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").expect("Failed to add region");
    let s1 = sm.add_substate("s1", r1).expect("Failed to add state");
    let s2 = sm.add_substate("s2", r1).expect("Failed to add state");
    let s3 = sm.add_substate("s3", s2).expect("Failed to add state");
    assert_eq!(sm.name(0).unwrap(), "sm1");
    assert_eq!(sm.name(r1).unwrap(), "r1");
    assert_eq!(sm.name(s1).unwrap(), "s1");
    assert_eq!(sm.name(s2).unwrap(), "s2");
    assert_eq!(sm.name(s3).unwrap(), "s3");
}

#[test]
fn test_fullnames() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").expect("Failed to add region");
    let s1 = sm.add_substate("s1", r1).expect("Failed to add state");
    let s2 = sm.add_substate("s2", r1).expect("Failed to add state");
    let s3 = sm.add_substate("s3", s2).expect("Failed to add state");
    assert_eq!(sm.fullname(0).unwrap(), "sm1");
    assert_eq!(sm.fullname(r1).unwrap(), "sm1::r1");
    assert_eq!(sm.fullname(s1).unwrap(), "sm1::r1::s1");
    assert_eq!(sm.fullname(s2).unwrap(), "sm1::r1::s2");
    assert_eq!(sm.fullname(s3).unwrap(), "sm1::r1::s2::region_1::s3");
}
