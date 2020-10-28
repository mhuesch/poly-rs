use std::collections::{HashMap, HashSet};
use std::iter::repeat_with;

use super::{env::*, syntax::*, types::*};

pub struct Constraint(Type, Type);

pub type Subst = HashMap<TV, Type>;

pub type Unifier = (Subst, Vec<Constraint>);

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
            Type::TVar(ref a) => match subst.get(&a) {
                None => self,
                Some(x) => x.clone(),
            },
            Type::TArr(t1, t2) => {
                let t1_ = t1.apply(subst);
                let t2_ = t2.apply(subst);
                Type::TArr(Box::new(t1_), Box::new(t2_))
            }
        }
    }

    pub fn ftv(self) -> HashSet<TV> {
        match self {
            Type::TCon(_) => HashSet::new(),
            Type::TVar(a) => {
                let mut hs = HashSet::new();
                hs.insert(a);
                hs
            }
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
        match self {
            Scheme(xs, ty) => {
                let subst2 = {
                    let mut subst_ = subst.clone();
                    for x in &xs {
                        subst_.remove(&x);
                    }
                    subst_
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
            }
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

// INFO this is most of the reason I opted to introduce the `Env` type at all:
// it's not possible to define an `impl` on a foreign type. however it's not
// clear that this is really necessary. it's nice to have the `apply` / `ftv`
// API consistency, but wouldn't be too bad to just have a one-off function for
// `Env`.
impl Env {
    pub fn apply(self, subst: &Subst) -> Env {
        self.iter()
            .map(|(nm, sc)| (nm.clone(), sc.clone().apply(subst)))
            .collect()
    }
    pub fn ftv(self) -> HashSet<TV> {
        let mut hs = HashSet::new();
        for sc in self.values() {
            let sc_ftvs = sc.clone().ftv();
            hs = hs.union(&sc_ftvs).map(|x| x.clone()).collect();
        }
        hs
    }
}

pub enum TypeError {
    UnificationFail(Type, Type),
    InfiniteType(TV, Type),
    UnboundVariable(Name),
    Ambigious(Vec<Constraint>),
    UnificationMismatch(Vec<Type>, Vec<Type>),
}

pub fn infer(
    env: Env,
    is: &mut InferState,
    expr: Expr,
) -> Result<(Type, Vec<Constraint>), TypeError> {
    match expr {
        Expr::Lit(lit) => Ok((infer_lit(lit), Vec::new())),
        Expr::Var(nm) => {
            let ty = lookup_env(&env, is, &nm)?;
            Ok((ty, Vec::new()))
        }
        Expr::Lam(nm, bd) => {
            let tv = is.fresh();
            let sc = Scheme(Vec::new(), tv.clone());
            let local_env = {
                let mut le = env.clone();
                le.replace(nm, sc);
                le
            };
            let (typ, csts) = infer(local_env, is, *bd)?;
            let ret = Type::TArr(Box::new(tv), Box::new(typ));
            Ok((ret, csts))
        }
        Expr::App(e1, e2) => {
            let (t1, mut csts1) = infer(env.clone(), is, *e1)?;
            let (t2, mut csts2) = infer(env, is, *e2)?;
            let tv = is.fresh();
            let cst = Constraint(t1, Type::TArr(Box::new(t2), Box::new(tv.clone())));
            csts1.append(&mut csts2);
            csts1.push(cst);
            Ok((tv, csts1))
        }
        Expr::Let(nm, e, bd) => {
            let (t_e, mut csts_e) = infer(env.clone(), is, *e)?;
            let subst = run_solve(&csts_e)?;
            let sc = generalize(env.clone().apply(&subst), t_e.apply(&subst));
            let local_env = {
                let mut le = env;
                le.replace(nm, sc);
                le.apply(&subst)
            };
            let (t_bd, mut csts_bd) = infer(local_env, is, *bd)?;
            csts_e.append(&mut csts_bd);
            Ok((t_bd, csts_e))
        }
        Expr::Fix(bd) => {
            let (t_bd, mut csts_bd) = infer(env.clone(), is, *bd)?;
            let tv = is.fresh();
            let cst = Constraint(t_bd, Type::TArr(Box::new(tv.clone()), Box::new(tv.clone())));
            csts_bd.push(cst);
            Ok((tv, csts_bd))
        }
        Expr::Prim(op) => {
            // TODO figure out if this is borked. `poly` takes the approach of
            // wrapping primop application, whereas I allow primops to be first
            // class functions.
            Ok((infer_primop(op), Vec::new()))
        }
        Expr::If(tst, thn, els) => {
            let (t_tst, mut csts_tst) = infer(env.clone(), is, *tst)?;
            let (t_thn, mut csts_thn) = infer(env.clone(), is, *thn)?;
            let (t_els, mut csts_els) = infer(env, is, *els)?;
            let cst_1 = Constraint(t_tst, type_bool());
            let cst_2 = Constraint(t_thn.clone(), t_els);
            csts_tst.append(&mut csts_thn);
            csts_tst.append(&mut csts_els);
            csts_tst.push(cst_1);
            csts_tst.push(cst_2);
            Ok((t_thn, csts_tst))
        }
    }
}

fn generalize(env: Env, ty: Type) -> Scheme {
    todo!()
}

fn run_solve(csts: &Vec<Constraint>) -> Result<Subst, TypeError> {
    todo!()
}

fn lookup_env(env: &Env, is: &mut InferState, nm: &Name) -> Result<Type, TypeError> {
    match env.get(nm) {
        None => Err(TypeError::UnboundVariable(nm.clone())),
        Some(sc) => instantiate(is, sc),
    }
}

fn instantiate(is: &mut InferState, sc: &Scheme) -> Result<Type, TypeError> {
    match sc {
        Scheme(xs, ty) => {
            let subst: Subst = xs
                .clone()
                .into_iter()
                .zip(repeat_with(|| is.fresh()))
                .collect();
            Ok(ty.clone().apply(&subst))
        }
    }
}

fn generalize(env: Env, ty: Type) -> Scheme {
    let ty_ = ty.clone();
    let ty_ftv = ty.ftv();
    let env_ftv = env.ftv();
    let free_vars = ty_ftv.difference(&env_ftv).map(|x| x.clone());
    Scheme(free_vars.collect(), ty_)
}

pub fn infer_lit(lit: Lit) -> Type {
    match lit {
        Lit::LInt(_) => type_int(),
        Lit::LBool(_) => type_bool(),
    }
}

fn infer_primop(op: PrimOp) -> Type {
    match op {
        PrimOp::Add => binop_arr(type_int(), type_int()),
        PrimOp::Mul => binop_arr(type_int(), type_int()),
        PrimOp::Sub => binop_arr(type_int(), type_int()),
        PrimOp::Eql => binop_arr(type_int(), type_bool()),
    }
}

fn binop_arr(arg: Type, ret: Type) -> Type {
    let inner = Type::TArr(Box::new(arg.clone()), Box::new(ret));
    Type::TArr(Box::new(arg), Box::new(inner))
}
