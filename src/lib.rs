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
    VertexAlreadyAdded(Name),
    VertexAlreadyInDifferentRegion(Name),
    ElementNotFound(DbId),
    InvalidState(DbId),
    InvalidVertex(DbId),
    InvalidRegion(DbId),
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
type DbId = usize;
type StateIdx = usize;
type VertexIdx = usize;
type TransitionId = usize;

#[derive(Debug)]
pub struct Db {
    name: Name,
    elements: Vec<Element>,
    pub dbid: DbId,
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
            dbid: 0,
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
        Ok(self._fullname(dbid))
    }

    fn _fullname(&self, dbid: DbId) -> &String {
        &self.fullnames[dbid]
    }

    pub fn region(&self, dbid: DbId) -> StateMachineResult<DbId> {
        self.is_valid_dbid(dbid)?;
        match self.elements[dbid].element_type {
            ElementType::Vertex(_) => Ok(self.vertices[self.elements[dbid].idx].container),
            ElementType::Region => Ok(dbid),
            ElementType::StateMachine => Err(StateMachineError::InvalidVertex(dbid)),
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

    pub fn to_string(&self, dbid: DbId) -> StateMachineResult<String> {
        let ele = self.element(dbid)?;
        match ele.element_type {
            ElementType::Vertex(_) => Ok(self.vertex_to_string(&self.vertices[ele.idx])),
            ElementType::StateMachine => Ok(format!("{:#?}", self.state_machine)),
            ElementType::Region => Ok(format!("{:#?}", self.regions[ele.idx])),
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

    pub fn print(&self, dbid: DbId) -> StateMachineResult<()> {
        println!("{}", self.to_string(dbid)?);
        Ok(())
    }

    pub fn is_simple(&self, dbid: DbId) -> StateMachineResult<bool> {
        let s_idx = self.get_state_by_dbid(dbid)?;
        println!("{:#?}", self.states[s_idx]);
        Ok(self.states[s_idx].is_simple())
    }

    pub fn is_composite(&self, dbid: DbId) -> StateMachineResult<bool> {
        let s_idx = self.get_state_by_dbid(dbid)?;
        Ok(self.states[s_idx].is_composite())
    }

    pub fn is_orthogonal(&self, dbid: DbId) -> StateMachineResult<bool> {
        let s_idx = self.get_state_by_dbid(dbid)?;
        Ok(self.states[s_idx].is_orthogonal())
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
        let r_idx = self.get_region_by_dbid(r_dbid)?;
        if self.is_duplicate(name, &self.regions[r_idx].subvertex) {
            return Err(StateMachineError::StateAlreadyExists(name));
        }
        let v_idx = self.vertices.len();
        let s_idx = self.states.len();
        let dbid = self.new_element(name, r_dbid, v_idx, ElementType::Vertex(VertexType::State));
        self.states.push(State::new(dbid));
        self.vertices
            .push(VertexDef::new(name, dbid, s_idx, r_dbid, VertexType::State));
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
                ElementType::StateMachine => Err(StateMachineError::NoCommonAncestor(s1, s2)),
                ElementType::Region => Ok(s2),
            }
        } else if self.ancestor(s2, s1) {
            match self.elements[s1].element_type {
                ElementType::Vertex(_) => Ok(self.parents[s1]),
                ElementType::StateMachine => Err(StateMachineError::NoCommonAncestor(s1, s2)),
                ElementType::Region => Ok(s1),
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

    pub fn ancestor_of(&self, parent: DbId, child: DbId) -> bool {
        self.ancestor(child, parent)
    }

    pub fn has_ancestor(&self, child: DbId, parent: DbId) -> bool {
        self.ancestor(child, parent)
    }

    pub fn ancestor(&self, child: DbId, parent: DbId) -> bool {
        // TODO: check if both are vertices
        if child == parent {
            true
        } else {
            self.is_contained_in(child, parent)
        }
    }

    pub fn sm_regions(&self) -> Vec<RegionDbId> {
        self.state_machine.regions.clone()
    }

    pub fn regions(&self, dbid: DbId) -> StateMachineResult<Vec<RegionDbId>> {
        let ele = self.element(dbid)?;
        match ele.element_type {
            ElementType::Vertex(VertexType::State) => Ok(self.states[ele.idx].regions.clone()),
            ElementType::StateMachine => Ok(self.state_machine.regions.clone()),
            _ => Err(StateMachineError::InvalidState(dbid)),
        }
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
                let s_idx = self.get_state_by_dbid(s_dbid).expect("Invalid state");
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

    fn get_region_by_dbid(&self, dbid: DbId) -> StateMachineResult<RegionIdx> {
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

    fn get_state_by_dbid(&self, dbid: DbId) -> StateMachineResult<StateIdx> {
        let ele = self.element(dbid)?;
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

    fn get_vertex_by_dbid(&self, dbid: DbId) -> StateMachineResult<VertexIdx> {
        println!("get_vertex_by_dbid: dbid:{:?}", dbid);
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
    incoming: Vec<TransitionId>,
    outgoing: Vec<TransitionId>,
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
    fn is_simple(&self) -> bool {
        self.regions.len() == 0
    }
    fn is_composite(&self) -> bool {
        self.regions.len() > 0
    }
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
