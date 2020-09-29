// use std::collections::HashMap;
use std::fmt;
// use std::process;
// use std::io::{self, Write};
// use std::string::ToString;

/// StateMachineError enumerates all possible errors returned by this library.
#[derive(Debug)]
pub enum StateMachineError {
    /// Represents an empty source. For example, an empty text file being given
    /// as input to `count_words()`.
    Duplicate(Name),
    VertexAlreadyAdded(Name),
    VertexAlreadyInDifferentRegion(Name),
    ElementNotFound(DbId),
    InvalidState(DbId),
    InvalidVertex(DbId),
    InvalidRegion(DbId),
    NoInitialState(DbId),
    InvalidDbId(DbId),
    NoCommonAncestor(DbId, DbId),
    StateAlreadyExists(Name),
    RegionAlreadyExists(Name),
    ContainsNoRegions(DbId),
    ContainsMultipleRegions(DbId),
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

type Name = &'static str;
pub type StateMachineResult<T> = Result<T, StateMachineError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VertexType {
    State,
    InitialState,
    FinalState,
}

/// Spec has lowercase for some of these Enums
#[derive(Debug, Clone, Copy)]
enum PseudostateKind {
    EntryPoint,
    ExitPoint,
    Initial,
    DeepHistory,
    ShallowHistory,
    Join,
    Fork,
    Junction,
    Terminate,
    Choice,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ElementType {
    Vertex(VertexType),
    Region,
    StateMachine,
    Transition,
    EventType,
}

#[derive(Debug, Copy, Clone)]
struct Element {
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

// Improve code readability, aliases for specific cases
// we will type check during construction
type RegionIdx = usize;
type RegionDbId = usize;
/// A DbId is an index into elements
type DbId = usize;
/// An Idx is an index into states|regions|vertices|triggers|...
/// For example, elements[dbid] -> idx1 -> vertices[idx1].idx2 -> states[idx2]
type Idx = usize;
type StateIdx = usize;
type StateDbId = usize;
type VertexIdx = usize;
type VertexDbId = usize;
type TriggerDbId = usize;
type TriggerIdx = usize;
type TransitionIdx = usize;

#[derive(Debug)]
/// elements:
/// parents:  Since this is the same size as elements,
///           we use the element dbid as an index to
///           look up its parent. No need for a HashMap.
pub struct Db {
    name: Name,
    elements: Vec<Element>,
    pub dbid: DbId,
    state_machine: StateMachineDef,
    states: Vec<State>,
    transitions: Vec<Transition>,
    event_types: Vec<EventType>,
    guards: Vec<Guard>,
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
    /// Create a new StateMachine.
    /// The constructed statemachine will have a single region
    /// automatically added and named "region_1". If you then
    /// call add_sm_region("region_name") before adding any
    /// vertices to the default region then the default region will be
    /// renamed.
    pub fn new(name: Name) -> Self {
        let mut db = Db {
            name,
            dbid: 0,
            elements: Vec::new(),
            states: Vec::new(),
            transitions: Vec::new(),
            event_types: Vec::new(),
            guards: Vec::new(),
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

    /// Given a dbid return the fully qualified path name from
    /// the statemachine and region recursively down to the element
    /// corresponding to the dbid.
    pub fn fullname(&self, dbid: DbId) -> StateMachineResult<&String> {
        self.is_valid_dbid(dbid)?;
        Ok(self._fullname(dbid))
    }

    /// Returns a full name, panics if the index is not valid
    /// invariant: state machine definition is complete
    fn _name(&self, dbid: DbId) -> Name {
        self.names[dbid]
    }

    /// Returns a full name, panics if the index is not valid
    /// invariant: state machine definition is complete
    fn _fullname(&self, dbid: DbId) -> &String {
        &self.fullnames[dbid]
    }

    /// Given the dbid of a vertex return the dbid of region in
    /// which it is directly contained.
    pub fn owning_region(&self, dbid: DbId) -> StateMachineResult<DbId> {
        self.is_valid_dbid(dbid)?;
        match self.elements[dbid].element_type {
            ElementType::Vertex(_) => Ok(self.vertices[self.elements[dbid].idx].container),
            ElementType::Region => Ok(dbid),
            ElementType::StateMachine => Err(StateMachineError::InvalidVertex(dbid)),
            ElementType::EventType => Err(StateMachineError::InvalidVertex(dbid)),
            ElementType::Transition => {
                Ok(self.vertices[self.transitions[self.elements[dbid].idx].source].container)
            }
        }
    }

    /// Return true if dbid is a State
    /// Mostly, It is easiest to consider a state machine as a state since
    /// There is no useful distinction between them for our purposes.
    pub fn is_state(&self, dbid: DbId) -> StateMachineResult<bool> {
        match self.element(dbid)?.element_type {
            ElementType::Vertex(VertexType::State) => Ok(true),
            // ElementType::StateMachine => Ok(true),
            _ => Ok(false),
        }
    }

    /// Return a pretty print string for an element
    pub fn to_string(&self, dbid: DbId) -> StateMachineResult<String> {
        let ele = self.element(dbid)?;
        match ele.element_type {
            ElementType::Vertex(_) => Ok(self.vertex_to_string(&self.vertices[ele.idx])),
            ElementType::StateMachine => Ok(format!("{:#?}", self.state_machine)),
            ElementType::Region => Ok(format!("{:#?}", self.regions[ele.idx])),
            ElementType::EventType => Ok(format!("{:#?}", self.event_types[ele.idx])),
            ElementType::Transition => Ok(format!("{:#?}", self.transitions[ele.idx])),
        }
    }

    fn vertex_to_string_body(&self, v: &VertexDef) -> String {
        format!(
            r#"    name: {},
    fullname: {},
    dbid: {},
    idx: {},
    vertex_type: {:#?},
    container: {},
    incoming: {},
    outgoing: {},"#,
            v.name,
            self._fullname(v.dbid),
            v.dbid,
            v.idx,
            v.vertex_type,
            self._fullname(v.container),
            v.incoming.len(),
            v.outgoing.len()
        )
    }

    fn vertex_to_string(&self, v: &VertexDef) -> String {
        match self.elements[v.dbid].element_type {
            ElementType::Vertex(VertexType::State) => self.state_to_string_body(v),
            _ => format!("Not implemented: {:#?}", v),
        }
    }

    fn state_to_string_body(&self, v: &VertexDef) -> String {
        let s = &self.states[v.idx];
        format!(
            r#"xState {{
{}
    state_type: {:#?},
    regions: {:#?},
    region: {:#?},
}}"#,
            self.vertex_to_string_body(v),
            s.state_type,
            s.regions,
            s.region
        )
    }

    /// Pretty print an element (region/state/...)
    pub fn print(&self, dbid: DbId) -> StateMachineResult<()> {
        println!("{}", self.to_string(dbid)?);
        Ok(())
    }

    /// Return true if the given state does not have any regions
    pub fn is_simple(&self, dbid: DbId) -> StateMachineResult<bool> {
        let s_idx = self.state(dbid)?;
        println!("{:#?}", self.states[s_idx]);
        Ok(self.states[s_idx].is_simple())
    }

    /// Return true if the given state has exactly on region
    pub fn is_composite(&self, dbid: DbId) -> StateMachineResult<bool> {
        let s_idx = self.state(dbid)?;
        Ok(self.states[s_idx].is_composite())
    }

    /// Return true if the given state has one or more on region
    pub fn is_orthogonal(&self, dbid: DbId) -> StateMachineResult<bool> {
        let s_idx = self.state(dbid)?;
        Ok(self.states[s_idx].is_orthogonal())
    }

    /// Return true if the given dbid is a valid element in the state machine
    pub fn is_valid_dbid(&self, dbid: DbId) -> StateMachineResult<()> {
        if dbid >= self.elements.len() {
            Err(StateMachineError::InvalidDbId(dbid))
        } else {
            Ok(())
        }
    }

    /// Return the short name for an element.
    pub fn name(&self, dbid: DbId) -> StateMachineResult<Name> {
        self.is_valid_dbid(dbid)?;
        Ok(self.names[dbid])
    }

    fn is_duplicate(&self, name: Name, vec: &Vec<DbId>) -> StateMachineResult<()> {
        match vec.iter().any(|&i| match self.elements[i].element_type {
            // ElementType::Transition => self.transitions[self.elements[i].idx].name == name,
            ElementType::Vertex(_) => self.vertices[self.elements[i].idx].name == name,
            ElementType::Region => self.regions[self.elements[i].idx].name == name,
            ElementType::Transition => self.transitions[self.elements[i].idx].name == name,
            _ => true,
        }) {
            true => Err(StateMachineError::Duplicate(name)),
            false => Ok(()),
        }
    }

    /// Add a transtion between two vertices to the state machine.
    /// The parent is defaulted to the owning region of the source vertex.
    /// The parent matters if the transition line connecting the vertices
    /// goes outside of the lca region - we would be saying the parent is
    /// the real lcx region in that case.
    pub fn add_event_type(&mut self, name: Name) -> StateMachineResult<DbId> {
        let e_idx = self.event_types.len();
        let parent = 0;
        let x = self.event_types.iter().map(|ev| ev.dbid).collect();
        self.is_duplicate(name, &x)?;
        let dbid = self.new_element(name, parent, e_idx, ElementType::Transition);
        self.event_types.push(EventType::new(name, dbid));
        Ok(dbid)
    }

    pub fn check_transition(&mut self, transition_dbid: TransitionIdx) -> StateMachineResult<bool> {
        Ok(self.transitions[self.transition(transition_dbid)?].check())
    }

    pub fn perform_entry(&mut self, state_dbid: StateIdx) -> StateMachineResult<()> {
        Ok(self.states[self.state(state_dbid)?].perform_entry())
    }

    pub fn perform_exit(&mut self, state_dbid: StateIdx) -> StateMachineResult<()> {
        Ok(self.states[self.state(state_dbid)?].perform_exit())
    }

    pub fn perform_do(&mut self, state_dbid: StateIdx) -> StateMachineResult<()> {
        Ok(self.states[self.state(state_dbid)?].perform_do())
    }

    /// Add a transtion between two vertices to the state machine.
    /// The parent is defaulted to the owning region of the source vertex.
    /// The parent matters if the transition line connecting the vertices
    /// goes outside of the lca region - we would be saying the parent is
    /// the real lcx region in that case.
    pub fn add_transition(
        &mut self,
        name: Name,
        trigger: Option<TriggerDbId>,
        source: VertexDbId,
        target: VertexDbId,
        effect: OptEffect,
        guard: OptGuard,
    ) -> StateMachineResult<DbId> {
        let t_idx = self.transitions.len();
        // let parent = self.vertices[self.vertex(source)?].container;

        // validation
        let parent = self.vertex_def(source)?.container;
        let source_idx = self.vertex(source)?;
        let outgoing = &self.vertices[source_idx].outgoing;
        self.is_duplicate(name, outgoing)?; // transitions have unique names
        let target_idx = self.vertex(target)?;
        let incoming = &self.vertices[target_idx].incoming;
        self.is_duplicate(name, incoming)?; // transitions have unique names

        // updates
        let dbid = self.new_element(name, parent, t_idx, ElementType::Transition);
        let outgoing = &mut self.vertices[source_idx].outgoing;
        outgoing.push(dbid);
        let incoming = &mut self.vertices[target_idx].incoming;
        incoming.push(dbid);
        let transition = Transition::new(name, dbid, trigger, source, target, effect, guard);
        self.transitions.push(transition);

        Ok(dbid)
    }

    fn vertex_def(&self, dbid: DbId) -> StateMachineResult<&VertexDef> {
        Ok(&self.vertices()[self.vertex(dbid)?])
    }

    fn vertices(&self) -> &Vec<VertexDef> {
        &self.vertices
    }

    pub fn print_active_states(&self) -> StateMachineResult<Vec<VertexDbId>> {
        let active_states = self._active_states(0)?;
        for dbid in &active_states {
            println!("[active_state] {}", self._fullname(*dbid))
        }
        Ok(active_states)
    }

    //pub fn regions(&mut self, dbid: DbId) -> StateMachineResult<Vec<VertexDbId>> {
    //}

    /// Return all incoming transitions whose target is directly
    /// within the given region, or the incoming transitions into
    /// the given vertex.
    pub fn transitions(&self, dbid: DbId) -> StateMachineResult<Vec<VertexDbId>> {
        let ele = self.element(dbid)?;
        match ele.element_type {
            ElementType::Region => Ok(self
                .transitions
                .iter()
                .filter(|t| self.parents[t.target] == dbid)
                .map(|t| t.dbid)
                .collect::<Vec<_>>()),
            ElementType::Vertex(VertexType::State) => Ok(self.vertices[ele.idx].incoming.clone()),
            ElementType::StateMachine => {
                Ok(self.transitions.iter().map(|t| t.dbid).collect::<Vec<_>>())
            }
            _ => Err(StateMachineError::InvalidVertex(dbid)),
        }
    }

    /// Return all transitions enabled for this vertex recursively.
    fn _active_states(&self, dbid: DbId) -> StateMachineResult<Vec<VertexDbId>> {
        println!(">active_state] {}", self._fullname(dbid));
        let ele = self.element(dbid)?;
        let mut active_states: Vec<TransitionIdx> = Vec::new();
        match ele.element_type {
            ElementType::Vertex(_) => {
                let v = &self.vertices[ele.idx];
                if ele.element_type == ElementType::Vertex(VertexType::State) {
                    let s = &self.states[v.idx];
                    if s.is_simple() {
                        println!("<active_state] {}", self._fullname(dbid));
                        active_states.push(dbid);
                    } else {
                        println!("|active_state] {}", self._fullname(dbid));
                        for r in &s.regions {
                            active_states.extend(self._active_states(*r)?);
                        }
                    }
                } else {
                    // Normally, this would not be possible, for pseudostates
                    println!("+active_state] {}", self._fullname(dbid));
                    active_states.push(dbid);
                }
            }
            ElementType::Region => {
                let r_idx = self.region(dbid)?;
                let r = &self.regions[r_idx];
                let active_state = r.active_state;
                if active_state != 0 {
                    println!("^active_state]: Region {} ", self._fullname(dbid));
                    active_states.extend(self._active_states(r.active_state)?);
                } else {
                    println!("error: Region {} has 0 active state", self._fullname(dbid));
                }
            }
            ElementType::StateMachine => {
                for dbid in &self.state_machine.regions {
                    active_states.extend(self._active_states(*dbid)?);
                }
            }
            _ => return Err(StateMachineError::InvalidVertex(dbid)),
        };
        Ok(active_states)
    }

    /// Return all transitions enabled for this vertex recursively.
    ///
    pub fn _plantuml(&self, dbid: DbId, indent: &String) -> StateMachineResult<String> {
        let ele = self.element(dbid)?;
        let new_indent = format!("{}    ", indent);
        match ele.element_type {
            ElementType::Region => {
                let r = self
                    .transitions(dbid)?
                    .iter()
                    .map(|t| {
                        let tx = &self.transitions[self.elements[*t].idx];
                        let mut src = self._name(tx.source);
                        src = if src == "initial" { "[*]" } else { src };
                        let tgt = self._name(tx.target);
                        format!("{}{} --> {}", indent, src, tgt)
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                // composite state transitions.
                let c = self
                    ._composite_states(dbid) // list of state dbids in region
                    .iter()
                    .map(|s_dbid| {
                        let cs = self
                            ._plantuml(*s_dbid, &new_indent)
                            .unwrap_or_else(|_| format!("Invalid_dbid:{}", s_dbid));
                        format!(
                            "{}state {} {{\n{}\n{}}}\n",
                            indent,
                            self._name(*s_dbid),
                            cs,
                            indent
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n");
                if c == "" {
                    Ok(r)
                } else if r == "" {
                    Ok(c)
                } else {
                    Ok(r + "\n" + &c)
                }
            }
            ElementType::Vertex(VertexType::State) => Ok(self.states[self.elements[dbid].idx]
                .regions
                .iter()
                .map(|r| {
                    self._plantuml(*r, &new_indent)
                        .unwrap_or_else(|_| format!("Invalid_dbid:{}", r))
                })
                .collect::<Vec<String>>()
                .join(&format!("{}-s-\n", indent))),
            ElementType::StateMachine => {
                let s = self
                    .state_machine
                    .regions
                    .iter()
                    .map(|r| {
                        self._plantuml(*r, &new_indent)
                            .unwrap_or_else(|_| format!("Invalid_dbid:{}", r))
                    })
                    .collect::<Vec<String>>()
                    .join(&format!("    {}---\n", indent));
                let o = format!(
                    "{}startuml\nstate {} {{\n{}}}\n@enduml\n",
                    '@', // prevent plantuml plugin from triggering
                    self._name(dbid),
                    s
                );
                Ok(o)
            }
            _ => Err(StateMachineError::InvalidVertex(dbid)),
        }
    }

    /// Return all transitions enabled for this vertex recursively.
    fn _transitions_enabled(&self, dbid: DbId) -> StateMachineResult<Vec<TransitionIdx>> {
        let ele = self.element(dbid)?;
        let mut enabled: Vec<TransitionIdx> = Vec::new();
        match ele.element_type {
            ElementType::Vertex(_) => {
                let v = &self.vertices[ele.idx];
                if ele.element_type == ElementType::Vertex(VertexType::State) {
                    let s = &self.states[v.idx];
                    for r in &s.regions {
                        enabled.extend(self._transitions_enabled(*r)?);
                    }
                }
                for t in &v.outgoing {
                    if self.transitions[*t].check() {
                        enabled.push(*t);
                    }
                }
            }
            ElementType::Region => {
                let r_idx = self.region(dbid)?;
                let r = &self.regions[r_idx];
                enabled.extend(self._transitions_enabled(r.active_state)?);
            }
            ElementType::StateMachine => {
                for dbid in &self.state_machine.regions {
                    enabled.extend(self._transitions_enabled(*dbid)?);
                }
            }
            _ => return Err(StateMachineError::InvalidVertex(dbid)),
        };
        Ok(enabled)
    }

    /// Add a transition from the InitialState to another
    /// vertex in the region. If there is no InitialState defined,
    /// then define one.
    pub fn initial_transition(
        &mut self,
        region: DbId,
        destination: DbId,
        effect: OptEffect,
    ) -> StateMachineResult<()> {
        let dbid = match self.initial_state(region) {
            Ok(Some(dbid)) => dbid,
            _ => self.add_vertex("initial", region, VertexType::InitialState)?,
        };
        self.add_transition("initial", None, dbid, destination, effect, OptGuard::None)?;
        Ok(())
    }

    fn initial_state(&self, region: DbId) -> StateMachineResult<Option<StateDbId>> {
        self._region(region)?.initial_state(self)
        /*
        let r_idx = self.region(region)?;
        for v in &self.regions[r_idx].subvertex {
            match self.elements[*v].element_type {
                ElementType::Vertex(VertexType::InitialState) => return Ok(Some(*v)),
                _ => (),
            }
        }
        Err(StateMachineError::NoInitialState(region))
        */
    }

    /// Add a state to the state machine.
    /// If you try to add a state when there is more than one region
    /// already defined for the state machine you will get an error.
    /// In which case you need to add the state to the desired
    /// region instead.
    pub fn set_entry(&mut self, state: DbId, entry: Entry) -> StateMachineResult<()> {
        let s_idx = self.state(state)?;
        self.states[s_idx].entry = Some(entry);
        Ok(())
    }

    /// Add a state to the state machine.
    /// If you try to add a state when there is more than one region
    /// already defined for the state machine you will get an error.
    /// In which case you need to add the state to the desired
    /// region instead.
    pub fn set_exit(&mut self, state: DbId, exit: Exit) -> StateMachineResult<()> {
        let s_idx = self.state(state)?;
        self.states[s_idx].exit = Some(exit);
        Ok(())
    }

    /// Add a state to the state machine.
    /// If you try to add a state when there is more than one region
    /// already defined for the state machine you will get an error.
    /// In which case you need to add the state to the desired
    /// region instead.
    pub fn add_state(&mut self, name: Name) -> StateMachineResult<DbId> {
        self.add_substate(name, self.state_machine.dbid)
    }

    /// Add a substate to an existing state or region.
    /// If you provide a state dbid then:
    ///   If the state does not have any regions, one will be created.
    ///   If the state has one region, that is one used.
    ///   If the state has multiple regions, that is an error.
    ///
    /// This is the same as add_vertex to identified region.
    ///
    /// If you try to add a state to a state with more than one region
    /// you will get an error - you need to add the state to the desired
    /// region instead.
    pub fn add_substate(&mut self, name: Name, parent: DbId) -> StateMachineResult<DbId> {
        let p_ele = self.element(parent)?;
        println!("{:#?}", self);
        println!("Adding Substate to: {:#?}", p_ele);
        let r_dbid = match p_ele.element_type {
            ElementType::Vertex(VertexType::State) => {
                let p_state_idx = self.get_state_by_ele(p_ele)?;
                println!("Adding Substate to: s_idx:{} ", p_state_idx);
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

        // if this is the first state added to the region
        // we cannot assume that there is an initial pseudostate
        // that is connected to it since initial_transition may be called
        // later to connect to a different state and we want to
        // protect against multiple calls to initial_transition

        // We reserve the name "initial" for the initial state
        // If you want to create a non-InitialState called "initial"
        // then use add_vertex(...) instead
        let vertex_type = if name == "initial" {
            VertexType::InitialState
        } else {
            VertexType::State
        };
        let dbid = self.add_vertex(name, r_dbid, vertex_type)?;
        Ok(dbid)
    }

    /// Create a new vertex. Initially the index to what the vertex represents
    /// is left unset (zero).
    /// name: name of the vertex, must be unique in the region
    /// region: dbid of the region containing the new vertex
    pub fn add_vertex(
        &mut self,
        name: Name,
        region: DbId,
        vertex_type: VertexType,
    ) -> StateMachineResult<VertexDbId> {
        let r_idx = self.region(region)?;
        self.is_duplicate(name, &self.regions[r_idx].subvertex)?;
        let v_idx = self.vertices.len();
        let dbid = self.new_element(name, region, v_idx, ElementType::Vertex(vertex_type));
        let idx = match vertex_type {
            VertexType::State => {
                let s_idx = self.states.len();
                self.states.push(State::new(dbid));
                s_idx
            }
            VertexType::InitialState => {
                self.regions[r_idx].initial_state = dbid;
                self.regions[r_idx].active_state = dbid;
                0 // we do not have a Vec of InitialState or PseudoStates
            }
            _ => 0,
        };
        self.vertices
            .push(VertexDef::new(name, dbid, idx, region, vertex_type));
        if self.regions[r_idx].subvertex.len() == 0 {
            self.regions[r_idx].initial_state = dbid;
            self.regions[r_idx].active_state = dbid;
        }
        self.regions[r_idx].subvertex.push(dbid);
        println!("Vertices:{:#?}", self.vertices);
        Ok(dbid)
    }

    /// Add a region to the state machine. If there was no region already
    /// added to the state machine and no vertices added, then it is
    /// assumed this replaces the name of the first region of the state machine.
    pub fn add_sm_region(&mut self, name: Name) -> StateMachineResult<RegionDbId> {
        let dbid = self.elements.len();
        // if we only have the statemachine and default region
        // both created upon StateMachine construction,
        // and then we call add_sm_region then presumably
        // we want to use that region name instead.
        if dbid == 2 && self.regions[0].name == "region_1" {
            println!(
                "Updating default region from {}",
                self.regions[self.elements[self.state_machine.regions[0]].idx].name
            );
            self.rename(1, name);
            self.regions[self.elements[self.state_machine.regions[0]].idx].name = name;
            println!(
                "Updating default region to {}/{}",
                self.regions[self.elements[self.state_machine.regions[0]].idx].name, name
            );
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

    /// Return the least common ancestor Region of s1 and s2
    /// If s1 is contained in s2 return the region of s1 and vice versa
    /// Otherwise
    pub fn lca(&self, s1: DbId, s2: DbId) -> StateMachineResult<RegionDbId> {
        // TODO: check if both are vertices
        self.is_valid_dbid(s1)?;
        if self.ancestor(s1, s2) {
            match self.elements[s2].element_type {
                ElementType::Vertex(_) => Ok(self.parents[s2]),
                ElementType::Transition => Ok(self.parents[s2]),
                ElementType::StateMachine => Err(StateMachineError::NoCommonAncestor(s1, s2)),
                ElementType::EventType => Err(StateMachineError::InvalidVertex(s2)),
                ElementType::Region => Ok(s2),
            }
        } else if self.ancestor(s2, s1) {
            match self.elements[s1].element_type {
                ElementType::Vertex(_) => Ok(self.parents[s1]),
                ElementType::StateMachine => Err(StateMachineError::NoCommonAncestor(s1, s2)),
                ElementType::EventType => Err(StateMachineError::InvalidVertex(s1)),
                ElementType::Region => Ok(s1),
                ElementType::Transition => Ok(self.parents[s1]),
            }
        } else {
            self.lca(self.parents[s1], self.parents[s2])
        }
    }

    /// Return the least common ancestor State of s1 and s2
    /// This is the owning state of the lca Region.
    pub fn lca_state(&self, s1: DbId, s2: DbId) -> StateMachineResult<DbId> {
        Ok(self.parents[self.lca(s1, s2)?])
    }

    /*
    fn parent_container_dbid(&self, child: DbId) -> StateMachineResult<DbId> {
        let idx = self.elements[child];
        match self.elements[child].element_type {
            ElementType::Vertex(_) => self.vertices[idx].container,
            ElementType::StateMachine => self.state_machine.regions[0],
            ElementType::Region => self.parents[child],
        }
    }
    */

    /// Return true if parent is an ancestor of child
    pub fn ancestor_of(&self, parent: DbId, child: DbId) -> bool {
        self.ancestor(child, parent)
    }

    /// Return true if child is contained within parent
    pub fn has_ancestor(&self, child: DbId, parent: DbId) -> bool {
        self.ancestor(child, parent)
    }

    /// Return true if child is contained within parent
    /// This is an alias for has_ancestor
    pub fn ancestor(&self, child: DbId, parent: DbId) -> bool {
        // TODO: check if both are vertices
        if child == parent {
            true
        } else {
            self.is_contained_in(child, parent)
        }
    }

    /// Return a list of dbids of regions in the state machine
    pub fn sm_regions(&self) -> Vec<RegionDbId> {
        self.state_machine.regions.clone()
    }

    /// Return a list of dbids of regions of a state
    pub fn regions(&self, dbid: DbId) -> StateMachineResult<Vec<RegionDbId>> {
        let ele = self.element(dbid)?;
        match ele.element_type {
            ElementType::Vertex(VertexType::State) => Ok(self.states[ele.idx].regions.clone()),
            ElementType::StateMachine => Ok(self.state_machine.regions.clone()),
            _ => Err(StateMachineError::InvalidState(dbid)),
        }
    }

    /// Return a list of dbids of othogonal states in a region
    /// panic if not called against a region
    pub fn _composite_states(&self, dbid: DbId) -> Vec<StateDbId> {
        self._states(dbid)
            .iter()
            .copied()
            .filter(|&dbid| self._is_composite(dbid))
            .collect()
    }

    /// Return a list of dbids of othogonal states in a region
    /// panic if not called against a region
    pub fn _orthonal_states(&self, dbid: DbId) -> Vec<StateDbId> {
        self._states(dbid)
            .iter()
            .copied()
            .filter(|&dbid| self._is_orthogonal(dbid))
            .collect()
    }

    /// Return a list of dbids of states in a region
    /// panic if not called against a region
    pub fn _states(&self, dbid: DbId) -> Vec<StateDbId> {
        let r_idx = self.elements[dbid].idx;
        self.regions[r_idx]
            .subvertex
            .iter()
            .copied()
            .filter(|&dbid| self._is_state(dbid))
            .collect()
    }

    /// Return a list of dbids of regions of a state
    pub fn states(&self, dbid: DbId) -> StateMachineResult<Vec<StateDbId>> {
        let ele = self.element(dbid)?;
        match ele.element_type {
            ElementType::Region => {
                let r_idx = self.region(dbid)?;
                Ok(self.regions[r_idx]
                    .subvertex
                    .iter()
                    .copied()
                    .filter(|dbid| self._is_state(*dbid))
                    .collect::<Vec<StateDbId>>())
            }
            _ => Err(StateMachineError::InvalidRegion(dbid)),
        }
    }

    #[inline]
    /// panic if not called against a vertex
    fn _is_state(&self, dbid: DbId) -> bool {
        self.elements[dbid].element_type == ElementType::Vertex(VertexType::State)
    }

    #[inline]
    /// panic if not called against a state
    fn _is_composite(&self, dbid: DbId) -> bool {
        self.states[self.vertices[self.elements[dbid].idx].idx].is_composite()
    }

    #[inline]
    /// panic if not called against a state
    fn _is_orthogonal(&self, dbid: DbId) -> bool {
        self.states[self.vertices[self.elements[dbid].idx].idx].is_orthogonal()
    }

    /// In the case where the state machine or state return the
    /// region dbid for the one region it contains or an error
    /// if there are is than one region defined.
    pub fn get_only_region(&self, dbid: DbId) -> StateMachineResult<RegionDbId> {
        let regions = self.regions(dbid)?;
        match regions.len() {
            0 => Err(StateMachineError::ContainsNoRegions(dbid)),
            1 => Ok(regions[0]),
            _ => Err(StateMachineError::ContainsMultipleRegions(dbid)),
        }
    }

    /// Print out one of a set of canned reports about the state machine
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

    /// Add a region to a State.
    /// Provding name of region, and the dbid of the owning state.
    pub fn add_region(&mut self, name: Name, parent: DbId) -> StateMachineResult<DbId> {
        let dbid = self.elements.len();
        //let ele_type = self.get_element_type(parent).expect("Invalid parent");
        let p_ele = self.element(parent)?;
        // let existing_regions = self.regions(parent)?;
        // let n = existing_regions.len();
        let c = match p_ele.element_type {
            ElementType::Vertex(VertexType::State) => {
                /*
                if n == 1 {
                    let r_dbid = existing_regions[0];
                    if self.elements[r_dbid].name == "region_1" {
                        self.rename(r_dbid, name);
                        self.regions[self.elements[r_dbid].idx].name = name;
                        return Ok(1);
                    }
                }
                */
                Container::State(p_ele.dbid)
            }
            ElementType::StateMachine => {
                /*
                if n == 1 {
                    let r_dbid = existing_regions[0];
                    if self.elements[r_dbid].name == "region_1" {
                        self.rename(r_dbid, name);
                        self.regions[self.elements[r_dbid].idx].name = name;
                        return Ok(1);
                    }
                }
                */
                Container::StateMachine(p_ele.dbid)
            }
            _ => return Err(StateMachineError::InvalidState(parent)),
        };
        // TODO: need to check if a region of the same name already exists
        let idx = self.regions.len();
        self.regions.push(Region::new(name, dbid, c));
        let dbid = self.new_element(name, parent, idx, ElementType::Region);
        self.add_region_to_container(c, dbid);
        println!("Added region {}:{}", name, dbid);
        Ok(dbid)
    }

    fn add_region_to_container(&mut self, c: Container, region_dbid: DbId) {
        match c {
            Container::State(s_dbid) => {
                let s_idx = self.state(s_dbid).expect("Invalid state");
                self.states[s_idx].add_region(region_dbid)
            }
            Container::StateMachine(_) => self.state_machine.add_region(region_dbid),
        };
    }

    fn element(&self, dbid: DbId) -> StateMachineResult<Element> {
        if dbid < self.elements.len() {
            return Ok(self.elements[dbid]);
        } else {
            return Err(StateMachineError::InvalidDbId(dbid));
        }
    }

    fn _region(&self, dbid: DbId) -> StateMachineResult<&Region> {
        println!("get_region_by_dbid: {:?}", dbid);
        let ele = self.element(dbid)?;
        println!("got_region_by_dbid: {:?}", ele);
        Ok(&self.regions[self.get_region_by_ele(ele)?])
    }

    fn region(&self, dbid: DbId) -> StateMachineResult<RegionIdx> {
        println!("get_region_by_dbid: {:?}", dbid);
        let ele = self.element(dbid)?;
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

    fn valid_transition(&self, dbid: DbId) -> StateMachineResult<()> {
        self.transition(dbid)?;
        Ok(())
    }

    fn transition(&self, dbid: DbId) -> StateMachineResult<TransitionIdx> {
        let ele = self.element(dbid)?;
        self.get_transition_by_ele(ele)
    }

    fn get_transition_by_ele(&self, ele: Element) -> StateMachineResult<VertexIdx> {
        match ele.element_type {
            ElementType::Transition => return Ok(ele.idx),
            _ => return Err(StateMachineError::InvalidVertex(ele.dbid)),
        }
    }

    /// Verify that a dbid references a State
    fn valid_state(&self, dbid: DbId) -> StateMachineResult<()> {
        self.state(dbid)?;
        Ok(())
    }

    /// Return the state index corresponding to a dbid
    fn state(&self, dbid: DbId) -> StateMachineResult<StateIdx> {
        let ele = self.element(dbid)?;
        self.get_state_by_ele(ele)
    }

    /// Return the state index corresponding to a dbid or panic!
    fn _state(&self, dbid: DbId) -> &State {
        &self.states[self.vertices[self.elements[dbid].idx].idx]
    }

    /// Return the state index corresponding to an element
    fn get_state_by_ele(&self, ele: Element) -> StateMachineResult<StateIdx> {
        match ele.element_type {
            ElementType::Vertex(VertexType::State) => {
                let s_idx = self.vertices[ele.idx].idx;
                return Ok(s_idx);
            }
            _ => return Err(StateMachineError::InvalidState(ele.dbid)),
        }
    }

    fn valid_vertex(&self, dbid: DbId) -> StateMachineResult<()> {
        self.vertex(dbid)?;
        Ok(())
    }

    /// Convert dbid to vertex index
    fn vertex(&self, dbid: DbId) -> StateMachineResult<VertexIdx> {
        println!("vertex: dbid:{:?}", dbid);
        let ele = self.element(dbid)?;
        println!("got_vertex_by_dbid: ele:{:?}", ele);
        self.get_vertex_by_ele(ele)
    }

    fn get_vertex_by_ele(&self, ele: Element) -> StateMachineResult<VertexIdx> {
        match ele.element_type {
            ElementType::Vertex(_) => return Ok(ele.idx),
            _ => return Err(StateMachineError::InvalidVertex(ele.dbid)),
        }
    }

    /// Return true if state/vertex/region etc. is contained in
    /// a different state/region.
    /// A child and parent dbid are provided.
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
        self._is_contained_in(child, parent)
    }

    fn _is_contained_in(&self, child: DbId, parent: DbId) -> bool {
        if child == 0 {
            println!("Not contained in {} {}", child, parent);
            false
        } else if self.parents[child] == parent {
            true
        } else {
            println!("no contained in {} {}", child, parent);
            self._is_contained_in(self.parents[child], parent)
        }
        /*
        let c_ele = self.elements[child];
        println!("Child Ele is {:#?}", c_ele);
        match c_ele.element_type {
            ElementType::Vertex(VertexType::State) => {
                let c_ver = &self.vertices[c_ele.idx];
                if c_ver.dbid == parent {
                    return true;
                }
                println!("Child State is {:#?}", c_ver);
                if c_ver.container == parent {
                    true
                } else {
                    self._is_contained_in(dbid, parent)
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
                                self._is_contained_in(dbid, parent)
                            }
                        }
                        Container::StateMachine(dbid) => dbid == parent,
                    }
                }
            }
            _ => false,
        }
        */
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
    container: RegionIdx, // Deviation: a vertex must be in a region.
    incoming: Vec<TransitionIdx>,
    outgoing: Vec<TransitionIdx>,
}
impl VertexDef {
    fn new(
        name: Name,
        dbid: DbId,
        idx: usize,
        region: RegionIdx, // Deviation: a vertex must be in a region.
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
    fn container(&self, db: &Db) -> StateMachineResult<RegionIdx> {
        Ok(db.vertices[self.def(db)?].container)
    }
    fn incoming<'db>(&self, db: &'db Db) -> StateMachineResult<&'db Vec<TransitionIdx>> {
        Ok(&db.vertices[self.def(db)?].incoming)
    }
    fn outgoing<'db>(&self, db: &'db Db) -> StateMachineResult<&'db Vec<TransitionIdx>> {
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
    dbid: DbId,
}
impl EventType {
    pub fn new(name: Name, dbid: DbId) -> EventType {
        EventType { name, dbid }
    }
}

#[derive(Debug, PartialEq)]
struct SubmachineState {
    name: Name,
}

pub type Entry = Behavior;
pub type Exit = Behavior;
pub type Effect = Behavior;
pub type OptEffect = OptBehavior;

//----------------------------------------------------------------
/// Behaviors are actions taken as a result of a transition.
/// Exit         - upon entry to a State
/// Entry        - upon exit from a State
/// Transition   - up performing a transition
/// DoActivity   - while in a state (aborted before Exit)
pub trait BehaviorFnTr: Fn() {
    fn name(&self) -> &str;
}
impl<F> BehaviorFnTr for F
where
    F: Fn(),
{
    fn name(&self) -> &str {
        "foo"
    }
}
impl Clone for Behavior {
    fn clone(&self) -> Self {
        Behavior {
            func: self.func,
            name: self.name.clone(),
        }
    }
}

#[derive(Copy)]
pub struct Guard {
    func: GuardFunc,
    name: &'static str,
}

impl Clone for Guard {
    fn clone(&self) -> Self {
        Guard {
            func: self.func,
            name: self.name.clone(),
        }
    }
}

type GuardFunc = fn() -> bool;

impl Guard {
    pub fn new(func: GuardFunc) -> Guard {
        Guard { func, name: "ggg" }
    }
    pub fn some(func: GuardFunc) -> OptGuard {
        OptGuard::Guard(Guard::new(func))
    }
}
impl fmt::Debug for Guard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Guard<{}>", self.name)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OptGuard {
    Guard(Guard),
    None,
}

#[derive(Debug, Copy, Clone)]
pub enum OptBehavior {
    Behavior(Behavior),
    None,
}

#[derive(Copy)]
pub struct Behavior {
    func: BehaviorFunc,
    name: &'static str,
}

type BehaviorFunc = fn() -> ();

impl Behavior {
    pub fn new(func: BehaviorFunc) -> Behavior {
        Behavior { func, name: "xxx" }
    }
    pub fn some(func: BehaviorFunc) -> OptBehavior {
        OptBehavior::Behavior(Behavior::new(func))
    }
}
impl fmt::Debug for Behavior {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Behavior<{}>", self.name)
    }
}

/*
impl Copy for Behavior {}
impl Clone for Behavior {
    fn clone(&self) -> Self {
        Behavior {
            func: self.func,
            name: self.name.clone(),
        }
    }
}
*/

type BehaviorFn = dyn BehaviorFnTr;
//type Behavior = Box<BehaviorFn>;
impl fmt::Debug for BehaviorFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Behavior<{}>", self.name())
    }
}

//----------------------------------------------------------------
/// Guard is used to determine is a transition can take
/// place. It is a function that returns true or false.
pub trait GuardFnTr: Fn() -> bool {
    fn name(&self) -> &str;
}
impl<F> GuardFnTr for F
where
    F: Fn() -> bool,
{
    fn name(&self) -> &str {
        "foo"
    }
}

// type Guard = Box<dyn Fn() -> bool>;
type GuardFn = dyn GuardFnTr<Output = bool>;
pub type OLDGuard = Box<GuardFn>;
impl fmt::Debug for GuardFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Guard<{}>", self.name())
    }
}
//----------------------------------------------------------------
// Debug helpers
//----------------------------------------------------------------
/// Extends a (possibly unsized) value with a Debug string.
// (This type is unsized when T is unsized)
pub struct Debuggable<T: ?Sized> {
    text: &'static str,
    value: T,
}

/// Produce a Debuggable<T> from an expression for T
macro_rules! dbg {
    ($($body:tt)+) => {
        Debuggable {
            text: stringify!($($body)+),
            value: $($body)+,
        }
    };
}

impl<T: ?Sized> fmt::Debug for Debuggable<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

// This makes Debuggable have most methods of the thing it wraps.
// It also lets you call it when T is a function.
impl<T: ?Sized> ::std::ops::Deref for Debuggable<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

//----------------------------------------------------------------
// Note: this type is unsized
pub type XGuardFn = Debuggable<dyn Fn() -> bool>;

/*
fn main() {
    let d: &PolFn = &dbg!(|x| {
        let _ = "random code so you can see how it's formatted";
        assert_eq!(3 * (1 + 2), 9);
        x
    });
    println!("{:?}", d);
}
*/

#[derive(Debug)]
pub struct Transition {
    name: Name,
    dbid: usize, // index into arena db elements
    trigger: Option<TriggerDbId>,
    source: VertexDbId,
    target: VertexDbId,
    effect: OptEffect,
    guard: OptGuard,
}

impl Transition {
    pub fn new(
        name: Name,
        dbid: usize,
        trigger: Option<TriggerDbId>,
        source: VertexDbId,
        target: VertexDbId,
        effect: OptEffect,
        guard: OptGuard,
    ) -> Transition {
        Transition {
            name,
            dbid,
            trigger,
            source,
            target,
            effect,
            guard,
        }
    }
    pub fn check(&self) -> bool {
        match &self.guard {
            OptGuard::None => true,
            OptGuard::Guard(guard) => (guard.func)(),
        }
    }
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

    /// For a StateMachine return a Region if there is only one region
    /// within that statemachine.
    fn get_only_region(&self) -> StateMachineResult<Option<RegionIdx>> {
        match self.regions.len() {
            n if n == 1 => Ok(Some(self.regions[0])),
            n if n == 0 => Ok(None),
            _ => Err(StateMachineError::ContainsMultipleRegions(self.dbid)),
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
/// during a transition the active_state of a region may be a vertex
/// but it must always end up as a State (or FinalState)
struct Region {
    name: Name,
    dbid: usize,
    container: Container,
    initial_state: DbId,
    active_state: DbId,
    subvertex: Vec<DbId>,
    transition: Vec<DbId>,
}

impl Region {
    fn new(name: Name, dbid: usize, container: Container) -> Region {
        Region {
            name,
            dbid,
            container,
            initial_state: 0,
            active_state: 0,
            subvertex: Vec::new(),
            transition: Vec::new(),
        }
    }
    fn initial_state(&self, sm: &StateMachine) -> StateMachineResult<Option<StateDbId>> {
        for v in &self.subvertex {
            match sm.elements[*v].element_type {
                ElementType::Vertex(VertexType::InitialState) => return Ok(Some(*v)),
                _ => (),
            }
        }
        Err(StateMachineError::NoInitialState(self.dbid))
    }
}

#[derive(Debug)]
/// Note: per spec this is a subclass of State but that it not neccessary
/// at the moment
/// No entry/exit/do/outgoing/regions
struct FinalState {
    dbid: DbId,
}

#[derive(Debug)]
struct State {
    dbid: DbId,
    state_type: StateType,
    regions: Vec<RegionIdx>,
    region: Option<RegionIdx>,
    entry: Option<Entry>,
    exit: Option<Exit>,
    do_while: Option<Behavior>,
}

impl State {
    fn new(dbid: usize) -> State {
        State {
            dbid,
            state_type: StateType::Simple,
            regions: Vec::new(),
            region: None,
            entry: None,
            exit: None,
            do_while: None,
        }
    }

    #[inline]
    fn is_simple(&self) -> bool {
        self.regions.len() == 0
    }

    #[inline]
    fn is_composite(&self) -> bool {
        self.regions.len() > 0
    }

    #[inline]
    fn is_orthogonal(&self) -> bool {
        self.regions.len() > 1
    }

    // fn is_submachine_state(&self) -> bool {
    // }

    fn add_region(&mut self, region_dbid: DbId) {
        self.regions.push(region_dbid);
    }

    /// For a State return a Region if there is only one region
    /// within that state
    fn get_only_region(&self) -> StateMachineResult<Option<RegionIdx>> {
        match self.regions.len() {
            n if n == 1 => Ok(Some(self.regions[0])),
            n if n == 0 => Ok(None),
            _ => Err(StateMachineError::ContainsMultipleRegions(self.dbid)),
        }
    }

    pub fn perform_entry(&self) {
        match &self.entry {
            Some(behavior) => (behavior.func)(),
            None => (),
        }
    }

    pub fn perform_exit(&self) {
        match &self.exit {
            Some(behavior) => (behavior.func)(),
            None => (),
        }
    }

    pub fn perform_do(&self) {
        match &self.do_while {
            Some(behavior) => (behavior.func)(),
            None => (),
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
        db.vertex(self.dbid)
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
