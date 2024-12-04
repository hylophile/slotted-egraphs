// hashcons collisions etc. are all checked manually for now.
// there's no hashing tricks in this conceptual design.

struct ProvenContains {
    node: L,
    parent_proof: Equation, 
    child_proofs: Vec<Equation>,
}

struct Class {
    syn_node: L,
    active_nodes: Vec<ProvenContains>,
}

struct EGraph {
    suf: Suf,
    classes: Vec<Class>,
}

impl EGraph {
    fn add(&mut self, n: L) -> AppliedId {
        if let m*x = n for some x, i with classes[i].active_nodes.contains(x) {
            return m*i
        } else {
            let i = suf.add(n.slots());
            classes.insert(i, Class {
                syn_node: n, // really?
                active_nodes: vec![n],
            });
            while n ==_cong m*n for some m {
                suf.groups[i].extend(m);
            }
            return identity * i;
        }
    }

    fn union(&mut self, x: AppliedId, y: AppliedId) {
        suf.union(x, y);

        if y.id deprecated {
            moves nodes from y to x
        }
    }

    fn explain_equivalence(&mut self, x: AppliedId, y: AppliedId) -> Proof {
        todo!()
    }
}

struct Proof {
    // the last lemma is the goal
    Vec<Lemma>, // indexed by LemmaId
}

struct Lemma {
    lhs: Term,
    rhs: Term,
    by: ProofStep,
}

// or Applied<LemmaId>
struct AppliedLemma {
    lemma_id: usize,
    application: SlotMap,
}

enum ProofStep {
    Reflexivity,
    Symmetry(AppliedLemma),
    Transitivity(AppliedLemma, AppliedLemma),
    Congruence(Vec<AppliedLemma>),
    Explicit(/*justification*/ String),
}
