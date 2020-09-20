extern crate rust_uml_sm;
use rust_uml_sm::Effect;
use rust_uml_sm::Entry;
use rust_uml_sm::Exit;
use rust_uml_sm::Guard;
use rust_uml_sm::StateMachine;
use rust_uml_sm::Transition;

#[test]
fn test_create_simple_states() {
    let mut sm = StateMachine::new("sm1");
    let s1 = sm.add_state("s1").unwrap();
    assert_eq!(s1, 2);
    let s2 = sm.add_state("s2").unwrap();
    assert_eq!(s2, 3);
}

#[test]
fn test_create_nested_state() {
    let mut sm = StateMachine::new("sm1");
    let s1 = sm.add_state("s1").unwrap();
    assert_eq!(s1, 2);
    let s2 = sm.add_substate("s2", s1).unwrap();
    assert_eq!(s2, 4);
    let s3 = sm.add_substate("s3", s1).unwrap();
    assert_eq!(s3, 5);
    let b = sm.is_contained_in(s2, s1);
    assert_eq!(b, true);
    let b = sm.is_contained_in(s1, s1);
    assert_eq!(b, false);
    let b = sm.is_contained_in(s1, s2);
    assert_eq!(b, false);
    let s4 = sm.add_substate("s4", s2).unwrap();
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
    assert_eq!(sm.add_sm_region("region_x").unwrap(), 1);
    assert_eq!(sm.name(1).unwrap(), "region_x");
    let region_y = sm.add_sm_region("region_y").unwrap();
    assert_eq!(sm.name(region_y).unwrap(), "region_y");
    // sm.add_region(c: Container, region_dbid: DbId)
}

#[test]
fn test_create_region_with_states() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").unwrap();
    println!("here1 {}", r1);
    let s1 = sm.add_substate("s1", r1).unwrap();
    println!("here2 {}", s1);
    let s2 = sm.add_substate("s2", r1).unwrap();
    let s3 = sm.add_substate("s3", s2).unwrap();
    println!("here3");
    assert_eq!(sm.owning_region(s1).unwrap(), r1);
    assert_eq!(sm.owning_region(s2).unwrap(), r1);
    println!("{:#?}", sm);
    assert_eq!(sm.owning_region(s3).unwrap(), 4usize);
    // sm.add_region(c: Container, region_dbid: DbId)
}

#[test]
fn test_names() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").unwrap();
    let s1 = sm.add_substate("s1", r1).unwrap();
    let s2 = sm.add_substate("s2", r1).unwrap();
    let s3 = sm.add_substate("s3", s2).unwrap();
    assert_eq!(sm.name(0).unwrap(), "sm1");
    assert_eq!(sm.name(r1).unwrap(), "r1");
    assert_eq!(sm.name(s1).unwrap(), "s1");
    assert_eq!(sm.name(s2).unwrap(), "s2");
    assert_eq!(sm.name(s3).unwrap(), "s3");
}

#[test]
fn test_fullnames() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").unwrap();
    let s1 = sm.add_substate("s1", r1).unwrap();
    let s2 = sm.add_substate("s2", r1).unwrap();
    let s3 = sm.add_substate("s3", s2).unwrap();
    assert_eq!(sm.fullname(0).unwrap(), "sm1");
    assert_eq!(sm.fullname(r1).unwrap(), "sm1::r1");
    assert_eq!(sm.fullname(s1).unwrap(), "sm1::r1::s1");
    assert_eq!(sm.fullname(s2).unwrap(), "sm1::r1::s2");
    assert_eq!(sm.fullname(s3).unwrap(), "sm1::r1::s2::region_1::s3");
}

#[test]
fn test_query_regions() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").unwrap();
    let r2 = sm.add_sm_region("r2").unwrap();
    let s1 = sm.add_substate("s1", r1).unwrap();
    let s2 = sm.add_substate("s2", r1).unwrap();
    let s3 = sm.add_substate("s3", s2).unwrap();
    let _ = sm.add_substate("s4", s3).unwrap();
    println!("{:#?}", sm);
    assert_eq!(sm.sm_regions(), vec![r1, r2]);
    assert_eq!(sm.regions(s1).unwrap(), vec![]);
    assert_eq!(sm.regions(s2).unwrap(), vec![5]);
    assert_eq!(sm.regions(s3).unwrap(), vec![7]);
}

#[test]
fn test_ancestor() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").unwrap();
    let r2 = sm.add_sm_region("r2").unwrap();
    let s1 = sm.add_substate("s1", r1).unwrap();
    let s2 = sm.add_substate("s2", r1).unwrap();
    let s3 = sm.add_substate("s3", s2).unwrap();
    let s4 = sm.add_substate("s4", s3).unwrap();
    assert_eq!(sm.ancestor_of(r1, s1), true);
    assert_eq!(sm.ancestor_of(r2, s1), false);
    assert_eq!(sm.ancestor_of(0, r1), true);
    assert_eq!(sm.ancestor_of(r1, s3), true);
    assert_eq!(sm.ancestor_of(s1, s3), false);
    assert_eq!(sm.ancestor_of(s2, s3), true);
    assert_eq!(sm.ancestor_of(s3, s4), true);
    assert_eq!(sm.ancestor_of(s3, r1), false);
    assert_eq!(sm.ancestor_of(0, r1), true);
    assert_eq!(sm.ancestor_of(0, 0), true);
    assert_eq!(sm.ancestor_of(s3, s3), true);
}

#[test]
fn test_lca() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").unwrap();
    let _ = sm.add_sm_region("r2").unwrap();
    let s1 = sm.add_substate("s1", r1).unwrap();
    let s2 = sm.add_substate("s2", r1).unwrap();
    let s3 = sm.add_substate("s3", s2).unwrap();
    let s4 = sm.add_substate("s4", s3).unwrap();
    assert_eq!(sm.lca(r1, s1).unwrap(), r1);
    assert_eq!(sm.lca(s1, s1).unwrap(), r1);
    assert_eq!(sm.lca(s2, s1).unwrap(), r1);
    assert_eq!(sm.lca(s3, s1).unwrap(), r1);
    assert_eq!(sm.lca(s4, s3).unwrap(), sm.get_only_region(s2).unwrap());
}

#[test]
fn test_lca_state() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").unwrap();
    let _ = sm.add_sm_region("r2").unwrap();
    let s1 = sm.add_substate("s1", r1).unwrap();
    let s2 = sm.add_substate("s2", r1).unwrap();
    let s3 = sm.add_substate("s3", s2).unwrap();
    let s4 = sm.add_substate("s4", s3).unwrap();
    assert_eq!(sm.lca_state(r1, s1).unwrap(), sm.dbid);
    assert_eq!(sm.lca_state(s1, s1).unwrap(), sm.dbid);
    assert_eq!(sm.lca_state(s2, s1).unwrap(), sm.dbid);
    assert_eq!(sm.lca_state(s3, s1).unwrap(), sm.dbid);
    assert_eq!(sm.lca_state(s4, s3).unwrap(), s2);
}

#[test]
fn test_state_type() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").unwrap();
    let _ = sm.add_sm_region("r2").unwrap();
    let s1 = sm.add_substate("s1", r1).unwrap();
    let s2 = sm.add_substate("s2", r1).unwrap();
    let s3 = sm.add_substate("s3", s2).unwrap();
    let r3 = sm.add_region("r3", s3).unwrap();
    let r4 = sm.add_region("r4", s3).unwrap();
    let s4 = sm.add_substate("s4", r3).unwrap();
    let s5 = sm.add_substate("s5", r4).unwrap();
    sm.print(0).unwrap();
    sm.print(s1).unwrap();
    assert_eq!(sm.is_simple(s1).unwrap(), true);
    assert_eq!(sm.is_composite(s1).unwrap(), false);
    assert_eq!(sm.is_simple(s2).unwrap(), false);
    assert_eq!(sm.is_composite(s2).unwrap(), true);
    assert_eq!(sm.is_orthogonal(s2).unwrap(), false);
    assert_eq!(sm.is_simple(s3).unwrap(), false);
    assert_eq!(sm.is_composite(s3).unwrap(), true);
    assert_eq!(sm.is_orthogonal(s3).unwrap(), true);
    assert_eq!(sm.is_simple(s4).unwrap(), true);
    assert_eq!(sm.is_simple(s5).unwrap(), true);
}

fn false_guard() -> bool {
    println!("returning false...");
    false
}

fn true_guard() -> bool {
    println!("returning true...");
    true
}

fn print_enter() {
    println!("enter");
}

fn print_transition() {
    println!("transition");
}

#[test]
fn test_state_behaviors() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").unwrap();
    let _ = sm.add_sm_region("r2").unwrap();
    let s1 = sm.add_substate("s1", r1).unwrap();
    let s2 = sm.add_substate("s2", r1).unwrap();
    let s3 = sm.add_substate("s3", s2).unwrap();
    let r3 = sm.add_region("r3", s3).unwrap();
    let r4 = sm.add_region("r4", s3).unwrap();
    let s4 = sm.add_substate("s4", r3).unwrap();
    let s5 = sm.add_substate("s5", r4).unwrap();
    // sm.on_entry(s1)
}

// To see diagram Alt-D (Install plantuml extention, graphviz, java)
/*
@startuml

state sm1 {
    ' single quote is a comment
    's1: first line   'This would be a long description
    's1: second line  'of the state s1 if you uncomment
    's2: description

    [*] --> s1
    s1 --> s2
    state s2 {
        [*] --> s3
        s3 -> s4
        s4 --> [*]
        ---
        [*] -> s6
        s6 -> s7
        s7 -> [*]
    }

    s2 --> [*]
    --
    [*] --> s5
    s5 -> [*]
}

@enduml
*/

#[test]
fn test_transitions() {
    let mut sm = StateMachine::new("sm1");
    let r1 = sm.add_sm_region("r1").unwrap();
    let _ = sm.add_sm_region("r2").unwrap();
    let s1 = sm.add_substate("s1", r1).unwrap();
    let s2 = sm.add_substate("s2", r1).unwrap();
    let s3 = sm.add_substate("s3", s2).unwrap();
    let r3 = sm.add_region("r3", s3).unwrap();
    let r4 = sm.add_region("r4", s3).unwrap();
    let s4 = sm.add_substate("s4", r3).unwrap();
    let s5 = sm.add_substate("s5", r4).unwrap();
    sm.set_entry(s1, Entry::new(print_enter)).unwrap();
    sm.set_exit(
        s1,
        Exit::new(|| {
            println!("bye from s1");
        }),
    )
    .unwrap();
    let ev1 = Some(sm.add_event_type("ev1").unwrap());
    let guard_false = Guard::some(false_guard);
    let guard_true = Guard::some(true_guard);
    let trans_effect = Effect::some(print_transition);

    let t1 = sm
        .add_transition("t1", ev1, s1, s2, trans_effect, guard_false)
        .unwrap();
    let t2 = sm
        .add_transition("t2", ev1, s1, s2, trans_effect, guard_true)
        .unwrap();
    assert_eq!(sm.check_transition(t1).unwrap(), false);
    assert_eq!(sm.check_transition(t2).unwrap(), true);

    sm.set_initial(r1, s1, trans_effect).unwrap();

    // sm.on_entry(s1)
}
