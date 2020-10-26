use std::collections::HashMap;
use std::collections::HashSet;

use super::types::{
    TV,
    Type,
    Scheme,
};

// TODO: unclear to me how useful this newtype is. I had first thought that
// we could define impl methods on it to only allow addition of constraints -
// logger style. however once we have a mutable reference to `Constraints`,
// anything is possible, so we gain little guarantee on safety. so now I'm
// inclined to revert to a type synonym.
pub struct Constraints(Vec<Constraint>);

pub struct Constraint(Type, Type);

pub struct Subst(HashMap<TV, Type>);

pub type Unifier = (Subst, Constraints);

pub struct InferState(u64);

impl InferState {
    pub fn new() -> InferState {
        InferState(0)
    }

    // TODO maybe improve this.
    // it starts at 1, not 0.
    // also we could do more intelligible names a la sdiehl's iterator through
    // alphabetical names -
    // "a", "b", ... "z", "aa", "ab", ... "az", "ba", ...
    pub fn fresh(&mut self) -> Type {
        let cnt = match self {
            InferState(c) => {
                *c += 1;
                *c
            }
        };
        let s = format!("t{}", cnt);
        Type::TVar(TV(s))
    }
}

impl Type {
    pub fn apply(self, subst: &Subst) -> Type {
        match self {
            Type::TCon(a) => Type::TCon(a),
            Type::TVar(ref a) => {
                let Subst(hm) = subst;
                match hm.get(&a) {
                    None => self,
                    Some(x) => x.clone(),
                }
            },
            Type::TArr(t1, t2) => {
                let t1_ = t1.apply(subst);
                let t2_ = t2.apply(subst);
                Type::TArr(Box::new(t1_), Box::new(t2_))
            },
        }
    }

    pub fn ftv(self) -> HashSet<TV> {
        match self {
            Type::TCon(_) => HashSet::new(),
            Type::TVar(a) => {
                let mut hs = HashSet::new();
                hs.insert(a);
                hs
            },
            Type::TArr(t1, t2) => {
                let hs2 = t2.ftv();
                // TODO figure out if this is performing unnecessary copying
                // I think we could just iterate through hs2 and insert values
                // into `t1.ftv()`
                t1.ftv().union(&hs2).map(|x| x.clone()).collect()
            }
        }
    }
}

impl Scheme {
    pub fn apply(self, subst: &Subst) -> Scheme {
        let Subst(hm) = subst;
        match self {
            Scheme(xs, ty) => {
                let subst2 = {
                    let mut hm_ = hm.clone();
                    for x in &xs {
                        hm_.remove(&x);
                    }
                    Subst(hm_)
                };
                Scheme(xs, ty.apply(&subst2))
            }
        }
    }
    pub fn ftv(self) -> HashSet<TV> {
        match self {
            Scheme(xs, ty) => {
                let mut hs = ty.ftv();
                for x in xs {
                    hs.remove(&x);
                }
                hs
            },
        }
    }
}

impl Constraint {
    pub fn apply(self, subst: &Subst) -> Constraint {
        match self {
            Constraint(t1, t2) => {
                let t1_ = t1.apply(subst);
                let t2_ = t2.apply(subst);
                Constraint(t1_, t2_)
            }
        }
    }
    pub fn ftv(self) -> HashSet<TV> {
        match self {
            Constraint(t1, t2) => {
                let hs2 = t2.ftv();
                // TODO see note on Type::ftv about excess copying
                t1.ftv().union(&hs2).map(|x| x.clone()).collect()
            }
        }
    }
}
