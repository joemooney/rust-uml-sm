// use std::collections::HashMap;
// use std::fmt;
// use std::process;
// use std::io::{self, Write};
//use std::string::ToString;

/*
Simple State: a state without internal Vertices or Transitions.
Composite State: a state with at least one Region.
Submachine State: a state referencing to another StateMachine, conceptually, nested within the State.
Simple composite State: has exactly one Region
Orthogonal State: has multiple Regions (isOrthogonal = true).

Substate: a State enclosed within a Region of a composite State is called a substate of that composite State.
Direct Substate: a substate of a region but not nested in another substate;
Indirect substate: a nested substate of a region.

a StateMachine can have multiple Regions, each of which may contain States of its own, some of which maybe composites with their own multiple Regions, etc. Consequently,
Active State: For a region or composite state, the particular substate to which we have currently transitioned.
Active State Configuration: Recursively, the active states for all regions.

""
StateMachine execution is represented by transitions from one active state configuration to another in response to Eventoccurrences
that match the Triggers of the StateMachine.A State is said to be active if it is part of the active state configuration.A state configuration is said to be stable when:no further Transitions from that state configuration are enabled andall the entry Behaviors of that configuration, if present, have completed (but not necessarily the doActivityBehaviors of that configuration, which, if defined, may continue executing).After it has been created and completed its initial Transition, a StateMachine is always “in” some state configuration. However, because States can be hierarchical and because there can be Behaviors associated with both Transitions and States, “entering” a hierarchical state configuration involves a dynamic process that terminates only after a stable state configuration (as defined above) is reached. This creates some potential ambiguity as to precisely when a StateMachine is “in” a particular state within a state configuration. The rules for when a StateMachine is deemed to be “in” a State and when it is deemed to have “left” a State are described below in the sections “Entering a State” and “Exiting a State respectively.
A configuration is deemed stable even if there are deferred, completion, or any other types of Event occurrences pending in the event pool of that StateMachine
 State may have an associated entry Behavior. This Behavior, if defined, is executed whenever the State is entered through an externalTransition. In addition, a State may also have an associated exit Behavior, which, if defined, is executed whenever the State is exited.A State may also have an associated doActivity Behavior. This Behavior commences execution when the State is entered (but only after the State entry Behavior has completed) and executes concurrently with any other Behaviors that may be associated with the State, until:it completes (in which case a completion event is generated) orthe State is exited, in which case execution of the doActivity Behavior is aborted.The execution of a doActivity Behavior of a State is not affected by the firing of an internalTransition of that State
The concept of State history was introduced by David Harel in the original statechart formalism. It is a convenience concept associated with Regions of composite States whereby a Region keeps track of the state configuration it was in when it was last exited. This allows easy return to that same state configuration, if desired, the next time the Region becomes active (e.g., after returning from handling an interrupt), or if there is a local Transition that returns to its history. This is achieved simply by terminating a Transition on the desired type of history Pseudostate inside the Region.The advantage provided by this facility is that it eliminates the need for users to explicitly keep track of history in cases where this type of behavior is desired, which can result in significantly simpler state machine models.Two types of history Pseudostates are provided. Deep history (deepHistory) represents the full state configuration of the most recent visit to the containing Region. The effect is the same as if the Transition terminating on the deepHistoryPseudostate had, instead, terminated on the innermost State of the preserved state configuration, including execution of all entry Behaviors encountered along the way. Shallow history (shallowHistory) represents a return to only the topmost substate of the most recent state configuration, which is entered using the default entry rule.In cases where a Transition terminates on a history Pseudostate when the State has not been entered before (i.e., no priorhistory) or it had reached its FinalState, there is an option to force a transition to a specific substate, using the default history mechanism. This is a Transition that originates in the history Pseudostate and terminates on a specific Vertex (thedefault history state) of the Region containing the history Pseudostate. This Transition is only taken if execution leads tothe history Pseudostate and the State had never been active before. Otherwise, the appropriate history entry into the Region is executed (see above). If no default history Transition is defined, then standard default entry of the Region is performed as explained below.Deferred Events A State may specify a set of Event types that may be deferred in that State. This means that Event occurrences of those types will not be dispatched as long as that State remains active. Instead, these Event occurrences remain in the event pool until:a state configuration is reached where these Event types are no longer deferred or,if a deferred Event type is used explicitly in a Trigger of a Transition whose source is the deferring State (i.e., akind of override option).An Event may be deferred by a composite State or submachine States, in which case it remains deferred as long as the composite State remains in the active configuration.
The semantics of entering a State depend on the type of State and the manner in which it is entered. However, in all cases, the entry Behavior of the State is executed (if defined) upon entry, but only after any effect Behavior associated
with the incoming Transition is completed. Also, if a doActivity Behavior is defined for the State, this Behavior commences execution immediately after the entry Behavior is executed. It executes concurrently with any subsequent Behaviors associated with entering the State, such as the entry Behaviors of substates entered as part of the same compound transition.The above description fully covers the case of simple States. For composite States with a single Region the following alternatives exist:Default entry: This situation occurs when the composite State is the direct target of a Transition (graphically, this is indicated by an incoming Transition that terminates on the outside edge of the composite State). After executing the entry Behavior and forking a possible doActivity Behavior execution, if an initial Pseudostate is defined, State entry continues from that Vertex via its outgoing Transition (known as the default Transition of the State). If no initial Pseudostate is defined, there is no single approach defined. One alternative is to treat such a model as ill formed. A second alternative is to treat the composite State as a simple State, terminating the traversal on that State despite its internal parts.Explicit entry: If the incoming Transition or its continuations terminate on a directly contained substate of the composite State, then that substate becomes active and its entry Behavior is executed after the execution of the entry Behavior of the containing composite State. This rule applies recursively if the Transition terminates on an indirect (deeply nested) substate.Shallow history entry: If the incoming Transition terminates on a shallowHistory Pseudostate of a Region of the composite State, the active substate becomes the substate that was most recently active prior to this entry, unless:othe most recently active substate is the FinalState, oro this is the first entry into this State.oIn the latter two cases, if a default shallow history Transition is defined originating from the shallowHistory Pseudostate, it will be taken. Otherwise, default State entry is applied.Deep history entry: The rule for this case is the same as for shallow history except that the target Pseudostate is of type deepHistory and the rule is applied recursively to all levels in the active state configuration below this one.Entry point entry: If a Transition enters a composite State through an entryPointPseudostate, then the effectBehavior associated with the outgoing Transition originating from the entry point and penetrating into the State(but after the entry Behavior of the composite State has been executed).If the composite State is also an orthogonal State with multiple Regions, each of its Regions is also entered, either by default or explicitly. If the Transition terminates on the edge of the composite State (i.e., without entering the State), then all the Regions are entered using the default entry rule above. If the Transition explicitly enters one or more Regions (in case of a fork), these Regions are entered explicitly and the others by default.Regardless of how a State is entered, the StateMachine is deemed to be “in” that State even before any entry Behavior oreffect Behavior (if defined) of that State start executing.
When exiting a State, regardless of whether it is simple or composite, the final step involved in the exit, after all other Behaviors associated with the exit are completed, is the execution of the exit Behavior of that State. If the State has a doActivity Behavior that is still executing when the State is exited, that Behavior is aborted before the exit Behavior commences execution.When exiting from a composite State, exit commences with the innermost State in the active state configuration. This means that exit Behaviors are executed in sequence starting with the innermost active State. If the exit occurs through anexitPointPseudostate, then the exit Behavior of the State is executed after the effect Behavior of the Transition terminating on the exit point.
When exiting from an orthogonal State, each of its Regions is exited. After that, the exit Behavior of the State is executed.Regardless of how a State is exited, the StateMachine is deemed to have “left” that State only after the exit Behavior (if defined) of that State has completed execution.Encapsulated composite StatesIn some modeling situations, it is useful to encapsulate a composite State, by not allowing Transitions to penetrate directly into the State to terminate on one of its internal Vertices. (One common use case for this is when the internals ofa State in an abstract Classifier are intended to be specified differently in different subtype refinements of the abstract Classifier.) Despite the encapsulation, it is often necessary to bind the internal elements of the composite State with incoming and outgoing Transitions. This is done by means of entry and exit points, which are realized via the entryPointand exitPointPseudostates.Entry points represent termination points (sources) for incoming Transitions and origination points (targets) for Transitions that terminate on some internal Vertex of the composite State. In effect, the latter is a continuation of the external incoming Transition, with the proviso that the execution of the entry Behavior of the composite State (if defined) occurs between the effect Behavior of the incoming Transition and the effect Behavior of the outgoing Transition. If there is no outgoing Transition inside the composite State, then the incoming Transition simply performs adefault State entry.Exit points are the inverse of entry points. That is, Transitions originating from a Vertex within the composite State can terminate on the exit point. In a well-formed model, such a Transition should have a corresponding external Transition outgoing from the same exit point, representing a continuation of the terminating Transition. If the composite State has an exit Behavior defined, it is executed after any effect Behavior of the incoming inside Transition and before any effectBehavior of the outgoing external Transition.
Submachines are a means by which a single StateMachine specification can be reused multiple times. They are similar to encapsulated composite States in that they need to bind incoming and outgoing Transitions to their internal Vertices. However, whereas encapsulated composite States and their internals are contained within the StateMachine in which they are defined, submachines are, like programming language macros, distinct Behavior specifications, which may be defined in a different context than the one where they are used (invoked). Consequently, they require a more complex binding. This is achieved through the concept of submachine State (i.e., States with isSubmachineState = true), which represent references to corresponding submachine StateMachines. The concept of ConnectionPointReference is provided to support binding between the submachine State and the referenced StateMachine. A ConnectionPointReference represents a point on the submachine State at which a Transition either terminates or originates. That is, they serve as targets for incoming Transitions to submachine States, as well as sources for outgoing Transitions from submachine States. Each ConnectionPointReference is matched by a corresponding entry or exit point in the referenced submachine StateMachine. This provides the necessary binding mechanism between the submachine invocation and its specification.A submachine State implies a macro-like insertion of the specification of the corresponding submachine StateMachine. It is, therefore, semantically equivalent to a composite State. The Regions of the submachine StateMachine are the Regions of the composite State. The entry, exit, and effect Behaviors and internal Transitions are defined as contained in the submachine State.NOTE. Each submachine State represents a distinct instantiation of a submachine, even when two or more submachine States reference the same submachine.A submachine StateMachine can be entered via its default (initial) Pseudostate or via any of its entry points (i.e., it may imply entering a non-orthogonal or an orthogonal composite State with Regions). Entering via the initial Pseudostate hasthe same meaning as for ordinary composite States. An entry point is equivalent to a junctionPseudostate (forkin cases where the composite State is orthogonal): Entering via an entry point implies that the entry Behavior of the composite state is executed, followed by the Transition from the entry point to the target Vertex within the composite State. Any guards associated with these entry point Transitions must evaluate to true in order for the specification to be well formed.alloc

Similarly, a submachine Statemachine can be exited as a result of:reaching its FinalState,triggering of a group Transition originating from a submachine State, orvia any of its exit points.Exiting via a FinalState or by a group Transition has the same meaning as for ordinary composite States.
""
*/

extern crate rust_uml_sm;
use rust_uml_sm::StateMachineDef;
use rust_uml_sm::StateMachine;

#[derive(PartialEq, Clone, Debug)]
enum Emotion {
    Happy,
    Sad,
    Angry,
}

#[derive(StateMachine)]
struct Foo {
    emotion: Emotion,
}
impl Foo {
    fn new() -> Foo {
        Foo {
            emotion: Emotion::Happy,
        }
    }
    pub fn is_happy(&self) -> bool {
        if self.emotion == Emotion::Happy {
            println!("Yes! I'm happy");
            true
        } else {
            println!("No! I'm not happy");
            false
        }

    }
}


fn main() {
    let mut sm = StateMachineDef::new("sm1");
    println!("Created {:#?}", sm);
    let s1 = sm.add_state("s1").expect("Failed to add state");
    println!("Added S1 {:#?}", sm);
    let _s2 = sm.add_substate("s2", s1).expect("Failed to add state");
    println!("Added S2 {:#?}", sm);

    let foo = Foo::new_statemachine();
    foo.state.is_happy();

    /*
    let mut s = StateMachine::new("sm1");
    static S1: &State = &State::new("s1");
    static S2: &State = &State::new("s2");
    if let Err(e) = s.add_vertex(S1) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
    if let Err(e) = s.add_vertex(S2) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
    println!("{}", &s);
    */
}
