use std::collections::HashMap;
use std::fmt;
use std::process;
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

type Name = &'static str;
pub type StateMachineResult<T> = Result<T, StateMachineError>;

#[derive(Debug, Clone, Copy)]
enum VertexType {
    State,
    InitialState,
    FinalState,
}

#[derive(Debug, Copy, Clone)]
enum ElementType {
    Vertex(VertexType),
    Region,
    StateMachine,
}

#[derive(Debug, Copy, Clone)]
pub struct Element {
    dbid: usize, // index into arena db of elements
    idx: usize,  // index into arena db of element type
    element_type: ElementType,
}
impl Element {
    fn new(dbid: usize, idx: usize, element_type: ElementType) -> Element {
        Element {
            dbid,
            idx,
            element_type,
        }
    }
}

struct ElementName {
    name: Name,
    fullname: Name,
}

enum DbElement {
    Element(Element),
    DbId(DbId),
}

// Improve code readability, aliases for specific cases
// we will type check during construction
type RegionIdx = usize;
type RegionDbId = usize;
type DbId = usize;
type StateIdx = usize;
type VertexIdx = usize;
type TransitionId = usize;

#[derive(Debug)]
pub struct Db {
    name: Name,
    elements: Vec<Element>,
    state_machine: StateMachineDef,
    states: Vec<State>,
    parents: Vec<DbId>,
    names: Vec<Name>,
    fullnames: Vec<String>,
    vertices: Vec<VertexDef>,
    regions: Vec<Region>,
}

pub type StateMachine = Db;

pub enum ReportType {
    Full,
    States,
}

impl Db {
    pub fn new(name: Name) -> Self {
        let mut db = Db {
            name,
            elements: Vec::new(),
            states: Vec::new(),
            parents: Vec::new(),
            vertices: Vec::new(),
            names: Vec::new(),
            fullnames: Vec::new(),
            state_machine: StateMachineDef::new(name, 0),
            regions: Vec::new(),
        };
        let dbid = db.new_element(name, 0, 0, ElementType::StateMachine);
        db.add_region("region_1", dbid)
            .expect("internal_error:912399");
        db
    }

    fn new_element(
        &mut self,
        name: Name,
        parent: DbId,
        idx: usize,
        element_type: ElementType,
    ) -> DbId {
        let dbid = self.elements.len();
        let new_ele = Element::new(dbid, idx, element_type);
        self.elements.push(new_ele);
        self.parents.push(parent);
        self.names.push(name);
        if dbid == 0 {
            self.fullnames.push(name.into());
        } else {
            self.fullnames
                .push(self.fullnames[parent].clone() + "::" + name);
        }
        dbid
    }

    /// Used to rename the default region name
    fn rename(&mut self, dbid: DbId, name: Name) {
        self.names[dbid] = name;
        if dbid == 0 {
            self.fullnames[0] = name.into();
        } else {
            self.fullnames[dbid] = self.fullnames[self.parents[dbid]].clone() + "::" + name;
        }
    }

    pub fn fullname(&self, dbid: DbId) -> StateMachineResult<&String> {
        self.is_valid_dbid(dbid)?;
        Ok(&self.fullnames[dbid])
    }

    pub fn region(&self, dbid: DbId) -> Option<DbId> {
        if dbid >= self.elements.len() {
            return None;
        }
        match self.elements[dbid].element_type {
            ElementType::Vertex(_) => self.vertices[self.elements[dbid].idx].container,
            ElementType::Region => None,
            ElementType::StateMachine => None,
        }
    }

    pub fn is_valid_dbid(&self, dbid: DbId) -> StateMachineResult<()> {
        if dbid >= self.elements.len() {
            Err(StateMachineError::InvalidDbId(dbid))
        } else {
            Ok(())
        }
    }

    pub fn name(&self, dbid: DbId) -> StateMachineResult<Name> {
        self.is_valid_dbid(dbid)?;
        Ok(self.names[dbid])
        /*
        match self.elements[dbid].element_type {
            ElementType::Vertex(_) => Ok(self.vertices[self.elements[dbid].idx].name),
            ElementType::Region => Ok(self.regions[self.elements[dbid].idx].name),
            ElementType::StateMachine => Ok(self.name),
        }
        */
    }

    fn is_duplicate(&self, name: Name, vec: &Vec<DbId>) -> bool {
        vec.iter().any(|&i| match self.elements[i].element_type {
            ElementType::Vertex(_) => self.vertices[self.elements[i].idx].name == name,
            ElementType::Region => self.regions[self.elements[i].idx].name == name,
            _ => true,
        })
    }

    pub fn add_state(&mut self, name: Name) -> StateMachineResult<DbId> {
        self.add_substate(name, self.state_machine.dbid)
    }

    pub fn add_substate(&mut self, name: Name, parent: DbId) -> StateMachineResult<DbId> {
        let p_ele = self.get_element(parent)?;
        println!("{:#?}", self);
        println!("Adding Substate to: {:#?}", p_ele);
        let r_dbid = match p_ele.element_type {
            ElementType::Vertex(VertexType::State) => {
                let p_state_idx = self.get_state_by_ele(p_ele)?;
                println!("Adding Substate to: s_idx:{}", p_state_idx);
                match self.states[p_state_idx].get_only_region()? {
                    Some(r_dbid) => r_dbid,
                    None => self.add_region("region_1", p_ele.dbid)?,
                }
            }
            ElementType::Region => p_ele.dbid,
            ElementType::StateMachine => self
                .state_machine
                .get_only_region()?
                .expect("internal_error:239923"),
            _ => return Err(StateMachineError::CannotAddState(p_ele.dbid)),
        };
        let r_idx = self.get_region_by_dbid(r_dbid)?;
        if self.is_duplicate(name, &self.regions[r_idx].subvertex) {
            return Err(StateMachineError::StateAlreadyExists(name));
        }
        let v_idx = self.vertices.len();
        let s_idx = self.states.len();
        let dbid = self.new_element(name, r_dbid, v_idx, ElementType::Vertex(VertexType::State));
        self.states.push(State::new(dbid));
        self.vertices.push(VertexDef::new(
            name,
            dbid,
            s_idx,
            Some(r_dbid),
            VertexType::State,
        ));
        self.regions[r_idx].subvertex.push(dbid);
        println!("Vertices:{:#?}", self.vertices);
        Ok(dbid)
    }

    pub fn add_sm_region(&mut self, name: Name) -> StateMachineResult<RegionDbId> {
        let dbid = self.elements.len();
        // if we only have the statemachine and default region
        // both created upon StateMachine construction,
        // and then we call add_sm_region then presumably
        // we want to use that region name instead.
        if dbid == 2 && self.regions[0].name == "region_1" {
            println!("Updating default region");
            self.rename(1, name);
            return Ok(1);
        }
        self.add_region(name, self.state_machine.dbid)
        /*
        let c = Container::StateMachine(self.state_machine.dbid);
        let idx = self.regions.len();
        self.regions.push(Region::new(name, dbid, c));
        let new_ele = Element::new(dbid, idx, ElementType::Region);
        self.state_machine.add_region(dbid);
        self.elements.push(new_ele);
        Ok(dbid)
        */
    }

    pub fn sm_regions(&self) -> Vec<RegionDbId> {
        self.regions.iter().map(|r| r.dbid).collect()
    }

    pub fn report(&self, report: ReportType) {
        match report {
            ReportType::Full => {
                println!("not implemented");
            }
            ReportType::States => {
                println!("not implemented");
            }
        }
    }

    fn add_region(&mut self, name: Name, parent: DbId) -> StateMachineResult<DbId> {
        let dbid = self.elements.len();
        //let ele_type = self.get_element_type(parent).expect("Invalid parent");
        let p_ele = self.get_element(parent)?;
        let c = match p_ele.element_type {
            ElementType::Vertex(VertexType::State) => Container::State(p_ele.dbid),
            ElementType::StateMachine => Container::StateMachine(p_ele.dbid),
            _ => return Err(StateMachineError::InvalidState(parent)),
        };
        // TODO: need to check if a region of the same name already exists
        let idx = self.regions.len();
        self.regions.push(Region::new(name, dbid, c));
        let dbid = self.new_element(name, parent, idx, ElementType::Region);
        self.add_region_to_container(c, dbid);
        Ok(dbid)
    }

    /*
    fn add_region(&mut self, region_dbid: DbId, container: DbId) -> StateMachineResult<DbId> {
        let c_ele = self.get_element(container)?;
        match c_ele.element_type {
            ElementType::Vertex(VertexType::State) => {
                let s_idx = self.get_state_by_dbid(c_ele.dbid)?;
                self.states[s_idx].add_region(region_dbid)
            }
            ElementType::StateMachine => self.state_machine.add_region(region_dbid),
        }
    }
    */

    fn add_region_to_container(&mut self, c: Container, region_dbid: DbId) {
        match c {
            Container::State(s_dbid) => {
                let s_idx = self.get_state_by_dbid(s_dbid).expect("Invalid state");
                self.states[s_idx].add_region(region_dbid)
            }
            Container::StateMachine(_) => self.state_machine.add_region(region_dbid),
        };
    }

    fn get_element(&self, dbid: DbId) -> StateMachineResult<Element> {
        if dbid < self.elements.len() {
            return Ok(self.elements[dbid]);
        } else {
            return Err(StateMachineError::ElementNotFound(dbid));
        }
    }

    // fn get_region(&self, ele: DbElement) -> StateMachineResult<RegionIdx> {
    //     match ele {
    //         DbElement::DbId(dbid) => self.get_region_by_dbid(dbid),
    //         DbElement::Element(e) => self.get_region_by_ele(e),
    //     }
    // }

    fn get_region_by_dbid(&self, dbid: DbId) -> StateMachineResult<RegionIdx> {
        println!("get_region_by_dbid: {:?}", dbid);
        let ele = self.get_element(dbid)?;
        println!("got_region_by_dbid: {:?}", ele);
        self.get_region_by_ele(ele)
    }

    fn get_region_by_ele(&self, ele: Element) -> StateMachineResult<RegionIdx> {
        match ele.element_type {
            ElementType::Region => {
                let r_idx = ele.idx;
                println!("got_region_by_ele: r_idx:{:?}", r_idx);
                return Ok(r_idx);
            }
            _ => return Err(StateMachineError::InvalidRegion(ele.dbid)),
        }
    }

    // fn get_state(&self, ele: DbElement) -> StateMachineResult<StateIdx> {
    //     match ele {
    //         DbElement::DbId(dbid) => self.get_state_by_dbid(dbid),
    //         DbElement::Element(e) => self.get_state_by_ele(e),
    //     }
    // }

    fn get_state_by_dbid(&self, dbid: DbId) -> StateMachineResult<StateIdx> {
        let ele = self.get_element(dbid)?;
        self.get_state_by_ele(ele)
    }

    fn get_state_by_ele(&self, ele: Element) -> StateMachineResult<StateIdx> {
        match ele.element_type {
            ElementType::Vertex(VertexType::State) => {
                let s_idx = self.vertices[ele.idx].idx;
                return Ok(s_idx);
            }
            _ => return Err(StateMachineError::InvalidState(ele.dbid)),
        }
    }

    // fn get_vertex(&self, ele: DbElement) -> StateMachineResult<VertexIdx> {
    //     match ele {
    //         DbElement::DbId(dbid) => self.get_vertex_by_dbid(dbid),
    //         DbElement::Element(e) => self.get_vertex_by_ele(e),
    //     }
    // }

    fn get_vertex_by_dbid(&self, dbid: DbId) -> StateMachineResult<VertexIdx> {
        println!("get_vertex_by_dbid: dbid:{:?}", dbid);
        let ele = self.get_element(dbid)?;
        println!("got_vertex_by_dbid: ele:{:?}", ele);
        self.get_vertex_by_ele(ele)
    }

    fn get_vertex_by_ele(&self, ele: Element) -> StateMachineResult<VertexIdx> {
        match ele.element_type {
            ElementType::Vertex(_) => return Ok(ele.idx),
            _ => return Err(StateMachineError::InvalidVertex(ele.dbid)),
        }
    }

    pub fn is_contained_in(&self, child: DbId, parent: DbId) -> bool {
        // Return True if child is contained in parent
        // If child equals parent return false
        if child >= self.elements.len() {
            return false;
        }
        if parent >= self.elements.len() {
            return false;
        }
        if child == parent {
            return false; // must be child, not same element
        }
        let c_ele = self.elements[child];
        println!("Child Ele is {:#?}", c_ele);
        match c_ele.element_type {
            ElementType::Vertex(VertexType::State) => {
                let c_ver = &self.vertices[c_ele.idx];
                if c_ver.dbid == parent {
                    return true;
                }
                println!("Child State is {:#?}", c_ver);
                match c_ver.container {
                    Some(dbid) => {
                        if dbid == parent {
                            true
                        } else {
                            self.is_contained_in(dbid, parent)
                        }
                    }
                    None => false,
                }
            }
            ElementType::Region => {
                let c_reg = &self.regions[c_ele.idx];
                if c_reg.dbid == parent {
                    true
                } else {
                    println!("Child Region is {:#?}", c_reg);
                    match c_reg.container {
                        Container::State(dbid) => {
                            if dbid == parent {
                                true
                            } else {
                                self.is_contained_in(dbid, parent)
                            }
                        }
                        Container::StateMachine(dbid) => dbid == parent,
                    }
                }
            }
            _ => false,
        }
    }
}

/// StateMachineError enumerates all possible errors returned by this library.
#[derive(Debug)]
pub enum StateMachineError {
    /// Represents an empty source. For example, an empty text file being given
    /// as input to `count_words()`.
    VertexAlreadyAdded(Name),
    VertexAlreadyInDifferentRegion(Name),
    ElementNotFound(DbId),
    InvalidState(DbId),
    InvalidVertex(DbId),
    InvalidRegion(DbId),
    InvalidDbId(DbId),
    StateAlreadyExists(Name),
    RegionAlreadyExists(Name),
    NoRegionsInStateMachine(Name),
    NoRegionsInState(Name),
    MultipleRegionsInStateMachine(DbId),
    CannotAddState(DbId),

    /// Represents a failure to read from input.
    ReadError {
        source: std::io::Error,
    },

    /// Represents all other cases of `std::io::Error`.
    IOError(std::io::Error),
}

impl std::error::Error for StateMachineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            StateMachineError::StateAlreadyExists(_) => None,
            StateMachineError::ReadError { ref source } => Some(source),
            StateMachineError::IOError(_) => None,
            _ => None,
        }
    }
}

impl std::fmt::Display for StateMachineError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            StateMachineError::StateAlreadyExists(name) => {
                write!(f, "State {} already defined", name)
            }
            StateMachineError::ReadError { .. } => write!(f, "Read error"),
            StateMachineError::IOError(ref err) => err.fmt(f),
            _ => write!(f, "Unhandled error"),
        }
    }
}

impl From<std::io::Error> for StateMachineError {
    fn from(err: std::io::Error) -> StateMachineError {
        StateMachineError::IOError(err)
    }
}

/*
impl std::ops::Deref for State {
    type Target = Vertex;
    fn deref(&self) -> &Self::Target {
        &self.vertex
    }
}
*/

#[derive(Debug)]
struct VertexDef {
    name: Name,
    dbid: usize, // index into arena db elements
    idx: usize,  // index into arena db vec of corresponding element type
    vertex_type: VertexType,
    container: Option<RegionIdx>,
    incoming: Vec<TransitionId>,
    outgoing: Vec<TransitionId>,
}
impl VertexDef {
    fn new(
        name: Name,
        dbid: DbId,
        idx: usize,
        region: Option<RegionIdx>,
        vertex_type: VertexType,
    ) -> VertexDef {
        VertexDef {
            name,
            dbid,
            idx,
            vertex_type,
            container: region,
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }
}

trait Vertex: std::fmt::Debug {
    fn def<'db>(&self, db: &'db Db) -> StateMachineResult<VertexIdx>;

    fn name(&self, db: &Db) -> StateMachineResult<Name> {
        Ok(db.vertices[self.def(db)?].name)
    }
    fn container(&self, db: &Db) -> StateMachineResult<Option<RegionIdx>> {
        Ok(db.vertices[self.def(db)?].container)
    }
    fn incoming<'db>(&self, db: &'db Db) -> StateMachineResult<&'db Vec<TransitionId>> {
        Ok(&db.vertices[self.def(db)?].incoming)
    }
    fn outgoing<'db>(&self, db: &'db Db) -> StateMachineResult<&'db Vec<TransitionId>> {
        Ok(&db.vertices[self.def(db)?].outgoing)
    }
    /// Am I a child of the given State?
    /// Return bool is this vertex is a substate (direct
    /// child or indirect) of a given State.
    /// isContainedInState
    fn is_contained_in_state(&self, db: &Db, dbid: DbId) -> StateMachineResult<bool> {
        Ok(db.is_contained_in(db.vertices[self.def(db)?].dbid, dbid))
    }
    fn is_contained_in_region(&self, db: &Db, dbid: DbId) -> StateMachineResult<bool> {
        Ok(db.is_contained_in(db.vertices[self.def(db)?].dbid, dbid))
    }
}

// This is an instance of an event
#[derive(Debug, PartialEq)]
struct Event {
    name: Name,
}

// This is a type of event which a state machine may expect
#[derive(Debug, PartialEq)]
struct EventType {
    name: Name,
}
#[derive(Debug, PartialEq)]
struct SubmachineState {
    name: Name,
}

#[derive(Debug, PartialEq)]
struct Transition {
    name: Name,
}

enum Visibility {
    Public,
    Private,
    Protected,
    Package,
}

trait NamedElement {
    fn name(&self) -> Name;
    fn qualified_name(&self) -> Name;
    fn visiblity(&self) -> Visibility;
}

#[derive(Debug, PartialEq)]
enum StateType {
    Simple,
    Composite,
    Orthogonal,
    Submachine,
}

// This is a type of event which a state machine may expect
#[derive(Debug)]
struct StateMachineDef {
    name: Name,
    dbid: DbId,
    regions: Vec<DbId>,
    // States for which this StateMachine is their realization
    submachine_states: Vec<DbId>,
}

impl StateMachineDef {
    fn new(name: Name, dbid: DbId) -> StateMachineDef {
        StateMachineDef {
            name,
            dbid,
            submachine_states: Vec::new(),
            regions: Vec::new(),
        }
        //let region = Region::new("region_1", Container::StateMachine(&sm));
        //sm.regions.push(region);
        //sm
    }
    fn add_region(&mut self, region_dbid: DbId) {
        self.regions.push(region_dbid);
    }

    fn get_only_region(&self) -> StateMachineResult<Option<RegionIdx>> {
        match self.regions.len() {
            n if n == 1 => Ok(Some(self.regions[0])),
            n if n == 0 => Ok(None),
            _ => Err(StateMachineError::MultipleRegionsInStateMachine(self.dbid)),
        }
    }
    // StateMachine::new_vertex
    // Presumably there is only one region in the state machine
    // otherwise it would not make sense to add a vertex.

    /*
    fn new_vertex(&self, db: &Db, v: VertexIdx) -> StateMachineResult<()> {
        let r_dbid = self.get_only_region()?;
        db.new_vertex()
        match v.container() {
            Some(its_region) => {
                if its_region as *const _ == my_region as *const _ {
                    Err(StateMachineError::VertexAlreadyAdded(v.name()))
                } else {
                    Err(StateMachineError::VertexAlreadyInDifferentRegion(v.name()))
                }
            }
            None => my_region.add_vertex(v),
        }
    }
    */
}

#[derive(Debug, Copy, Clone)]
enum Container {
    State(DbId),
    StateMachine(DbId),
}

#[derive(Debug)]
struct Region {
    name: Name,
    dbid: usize,
    container: Container,
    subvertex: Vec<DbId>,
    transition: Vec<DbId>,
}

impl Region {
    fn new(name: Name, dbid: usize, container: Container) -> Region {
        Region {
            name,
            dbid,
            container,
            subvertex: Vec::new(),
            transition: Vec::new(),
        }
    }
    /// Return bool is this region is within a given state:
    /// either the given state owns the region directly or
    /// indirectly.
    /// This function walks from innermost to outermost.
    /// isContainedInState
    fn is_contained_in_state(&self, db: &Db, dbid: usize) -> StateMachineResult<bool> {
        match self.container {
            Container::State(s_dbid) => {
                if s_dbid == dbid {
                    Ok(true)
                } else {
                    // let v = s as &dyn Vertex;
                    let s_idx = db.get_state_by_dbid(s_dbid).expect("Invalid state");
                    db.states[s_idx].is_contained_in_state(db, dbid)
                }
            }
            Container::StateMachine(_) => Ok(false),
        }
    }
}

#[derive(Debug)]
struct State {
    dbid: DbId,
    state_type: StateType,
    regions: Vec<RegionIdx>,
    region: Option<RegionIdx>,
}

impl State {
    fn new(dbid: usize) -> State {
        State {
            dbid,
            state_type: StateType::Simple,
            regions: Vec::new(),
            region: None,
        }
    }
    fn isSimple(&self) -> bool {
        self.state_type == StateType::Simple
    }
    fn isComposite(&self) -> bool {
        self.state_type == StateType::Composite
    }
    fn isOrthogonal(&self) -> bool {
        self.state_type == StateType::Orthogonal
    }
    fn isSubmachineState(&self) -> bool {
        self.state_type == StateType::Submachine
    }

    fn add_region(&mut self, region_dbid: DbId) {
        self.regions.push(region_dbid);
    }

    fn get_only_region(&self) -> StateMachineResult<Option<RegionIdx>> {
        match self.regions.len() {
            n if n == 1 => Ok(Some(self.regions[0])),
            n if n == 0 => Ok(None),
            _ => Err(StateMachineError::MultipleRegionsInStateMachine(self.dbid)),
        }
    }

    /*
    fn new_state(&mut self, state: &'static State) -> StateMachineResult<()> {
        match self.state_type {
            StateType::Simple | StateType::Composite => {
                if self.state_type == StateType::Simple {
                    self.state_type = StateType::Composite;
                    match self.add_region("region_1") {
                        Err(e) => return Err(e),
                        _ => ()
                    }
                }
                self.regions[0].new_state(state)
            },
            StateType::Orthogonal => {
                Err(StateMachineError::StateAlreadyExists(state_name)),
            },
            StateType::Submachine => {
                Err(StateMachineError::StateAlreadyExists(state_name)),
            }
        }
    }
    */
}

impl Vertex for State {
    fn def<'db>(&self, db: &'db Db) -> StateMachineResult<VertexIdx> {
        db.get_vertex_by_dbid(self.dbid)
    }
}

impl fmt::Display for StateMachineDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "StateMachine: {}\nNumber of Regions: {}\n SubmachineState: {}",
            self.name,
            self.regions.len(),
            self.submachine_states.len(),
        )
    }
}
