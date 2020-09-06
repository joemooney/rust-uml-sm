struct StateMachine {}
struct State {
    v: VertexDef,
}

struct VertexDef {
    container: Option<&'static Region>,
}

trait Vertex {
    fn def(&self) -> &VertexDef;
    fn is_contained_in_state(&'static self, s: &'static State) -> bool {
        let def = &self.def();
        match def.container {
            None => false,
            Some(container) => false, // container.is_contained_in_state(s),
        }
    }
}

impl State {}

impl Vertex for State {
    fn def(&self) -> &VertexDef {
        return &self.v;
    }
}

enum Container {
    State(&'static State),
    StateMachine(&'static StateMachine),
}

struct Region {
    container: Container,
}

impl Region {
    fn is_contained_in_state(&self, s: &'static State) -> bool {
        match self.container {
            Container::State(my) => {
                if my as *const _ == s as *const _ {
                    true
                } else {
                    // let v = s as &dyn Vertex;
                    my.is_contained_in_state(s) // <-- I want to call Vertex default trait impl
                }
            }
            // Container::StateMachine(m) => m.is_contained_in_state(s),
            Container::StateMachine(m) => false, // TODO
        }
    }
}

fn main() {
    static SM: StateMachine = StateMachine {};
    static R: Region = Region {
        container: Container::StateMachine(&SM),
    };
    static S1: State = State {
        v: VertexDef {
            container: Some(&R),
        },
    };
    static S2: State = State {
        v: VertexDef {
            container: Some(&R),
        },
    };
    static S2: State = State {
        v: VertexDef {
            container: Some(&R),
        },
    };
    //let v = s1 as &dyn Vertex;
    if S1.is_contained_in_state(&S2) {
        println!("yes");
    } else {
        println!("no");
    }
    if S1.is_contained_in_state(&S1) {
        println!("yes");
    } else {
        println!("no");
    }
}
