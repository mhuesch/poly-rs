use std::collections::{HashMap, HashSet};
use std::iter;

use super::{env::*, syntax::*, types::*};

#[derive(Clone, Debug)]
pub struct Constraint(Type, Type);

pub type Subst = HashMap<TV, Type>;

struct InferState(u64);

impl InferState {
    fn new() -> InferState {
        InferState(0)
    }

    // TODO maybe improve this.
    // it starts at 1, not 0.
    // also we could do more intelligible names a la sdiehl's iterator through
    // alphabetical names -
    // "a", "b", ... "z", "aa", "ab", ... "az", "ba", ...
    fn fresh(&mut self) -> Type {
        Type::TVar(self.fresh_tv())
    }

    fn fresh_tv(&mut self) -> TV {
        let cnt = match self {
            InferState(c) => {
                *c += 1;
                *c
            }
        };
        let s = format!("t{}", cnt);
        TV(s)
    }
}

impl Type {
    fn apply(self, subst: &Subst) -> Type {
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
            Type::TList(ty) => {
                let ty_ = ty.apply(subst);
                Type::TList(Box::new(ty_))
            }
            Type::TPair(t1, t2) => {
                let t1_ = t1.apply(subst);
                let t2_ = t2.apply(subst);
                Type::TPair(Box::new(t1_), Box::new(t2_))
            }
        }
    }

    fn ftv(self) -> HashSet<TV> {
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
            Type::TList(ty) => ty.ftv(),
            Type::TPair(t1, t2) => {
                let hs2 = t2.ftv();
                t1.ftv().union(&hs2).map(|x| x.clone()).collect()
            }
        }
    }
}

impl Scheme {
    fn apply(self, subst: &Subst) -> Scheme {
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
    fn ftv(self) -> HashSet<TV> {
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
    fn apply(self, subst: &Subst) -> Constraint {
        match self {
            Constraint(t1, t2) => {
                let t1_ = t1.apply(subst);
                let t2_ = t2.apply(subst);
                Constraint(t1_, t2_)
            }
        }
    }
    #[allow(dead_code)]
    fn ftv(self) -> HashSet<TV> {
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
    fn apply(self, subst: &Subst) -> Env {
        self.iter()
            .map(|(nm, sc)| (nm.clone(), sc.clone().apply(subst)))
            .collect()
    }
    fn ftv(self) -> HashSet<TV> {
        let mut hs = HashSet::new();
        for sc in self.values() {
            let sc_ftvs = sc.clone().ftv();
            hs = hs.union(&sc_ftvs).map(|x| x.clone()).collect();
        }
        hs
    }
}

#[derive(Debug)]
pub enum TypeError {
    UnificationFail(Type, Type),
    InfiniteType(TV, Type),
    UnboundVariable(Name),
    Ambigious(Vec<Constraint>),
    UnificationMismatch(Vec<Type>, Vec<Type>),
}

fn infer(env: Env, is: &mut InferState, expr: &Expr) -> Result<(Type, Vec<Constraint>), TypeError> {
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
            let (typ, csts) = infer(local_env, is, bd)?;
            let ret = Type::TArr(Box::new(tv), Box::new(typ));
            Ok((ret, csts))
        }
        Expr::App(e1, e2) => {
            let (t1, mut csts1) = infer(env.clone(), is, e1)?;
            let (t2, mut csts2) = infer(env, is, e2)?;
            let tv = is.fresh();
            let cst = Constraint(t1, Type::TArr(Box::new(t2), Box::new(tv.clone())));
            csts1.append(&mut csts2);
            csts1.push(cst);
            Ok((tv, csts1))
        }
        Expr::Let(nm, e, bd) => {
            let (t_e, mut csts_e) = infer(env.clone(), is, e)?;
            let subst = run_solve(csts_e.to_vec())?;
            let sc = generalize(env.clone().apply(&subst), t_e.apply(&subst));
            let local_env = {
                let mut le = env;
                le.replace(nm, sc);
                le.apply(&subst)
            };
            let (t_bd, mut csts_bd) = infer(local_env, is, bd)?;
            csts_e.append(&mut csts_bd);
            Ok((t_bd, csts_e))
        }
        Expr::List(xs) => {
            let tv_elem = is.fresh();
            let mut csts = Vec::new();
            for x in xs {
                let (t_x, mut csts_x) = infer(env.clone(), is, x)?;
                csts.append(&mut csts_x);
                csts.push(Constraint(t_x, tv_elem.clone()));
            }
            let tv_list = is.fresh();
            csts.push(Constraint(tv_list.clone(), Type::TList(Box::new(tv_elem))));
            Ok((tv_list, csts))
        }
        Expr::Fix(bd) => {
            let (t_bd, mut csts_bd) = infer(env.clone(), is, bd)?;
            let tv = is.fresh();
            let cst = Constraint(t_bd, Type::TArr(Box::new(tv.clone()), Box::new(tv.clone())));
            csts_bd.push(cst);
            Ok((tv, csts_bd))
        }
        Expr::Prim(op) => {
            // TODO figure out if this is borked. `poly` takes the approach of
            // wrapping primop application, whereas I allow primops to be first
            // class functions.
            Ok((infer_primop(is, op), Vec::new()))
        }
        Expr::If(tst, thn, els) => {
            let (t_tst, mut csts_tst) = infer(env.clone(), is, tst)?;
            let (t_thn, mut csts_thn) = infer(env.clone(), is, thn)?;
            let (t_els, mut csts_els) = infer(env, is, els)?;
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

pub fn infer_program(mut env: Env, prog: &Program) -> Result<(Scheme, Env), TypeError> {
    for Defn(name, expr) in prog.p_defns.iter() {
        let sc = infer_expr(env.clone(), &expr)?;
        env.extend(name.clone(), sc);
    }
    let sc = infer_expr(env.clone(), &prog.p_body)?;
    Ok((sc, env))
}

pub fn infer_expr(env: Env, expr: &Expr) -> Result<Scheme, TypeError> {
    let mut is = InferState::new();
    let (ty, csts) = infer(env, &mut is, expr)?;
    let subst = run_solve(csts)?;
    Ok(close_over(ty.apply(&subst)))
}

/// Return extra internal information, as compared to `infer_expr`.
pub fn constraints_expr(
    env: Env,
    expr: &Expr,
) -> Result<(Vec<Constraint>, Subst, Type, Scheme), TypeError> {
    let mut is = InferState::new();
    let (ty, csts) = infer(env, &mut is, expr)?;
    let subst = run_solve(csts.clone())?;
    let sc = close_over(ty.clone().apply(&subst));
    Ok((csts, subst, ty, sc))
}

fn close_over(ty: Type) -> Scheme {
    normalize(generalize(Env::new(), ty))
}

fn normalize(sc: Scheme) -> Scheme {
    let Scheme(_, body) = sc;
    let hm = {
        let mut vars: Vec<TV> = free_type_vars(body.clone()).collect();
        vars.dedup();
        let mut hm = HashMap::new();
        let mut is = InferState::new();
        for var in vars {
            hm.insert(var, is.fresh_tv());
        }
        hm
    };
    let foralls = hm.values().map(|x| x.clone()).collect();
    let ty = norm_type(&hm, body);
    Scheme(foralls, ty)
}

fn norm_type(hm: &HashMap<TV, TV>, ty: Type) -> Type {
    match ty {
        Type::TArr(a, b) => {
            let a_ = norm_type(hm, *a);
            let b_ = norm_type(hm, *b);
            Type::TArr(Box::new(a_), Box::new(b_))
        }
        Type::TCon(a) => Type::TCon(a),
        Type::TVar(a) => match hm.get(&a) {
            Some(x) => Type::TVar(x.clone()),
            None => panic!("norm_type: impossible: type var not in signature"),
        },
        Type::TList(a) => {
            let a_ = norm_type(hm, *a);
            Type::TList(Box::new(a_))
        }
        Type::TPair(a, b) => {
            let a_ = norm_type(hm, *a);
            let b_ = norm_type(hm, *b);
            Type::TPair(Box::new(a_), Box::new(b_))
        }
    }
}

fn free_type_vars(ty: Type) -> Box<dyn Iterator<Item = TV>> {
    match ty {
        Type::TVar(a) => Box::new(iter::once(a)),
        Type::TArr(a, b) => Box::new(free_type_vars(*a).chain(free_type_vars(*b))),
        Type::TCon(_) => Box::new(iter::empty()),
        Type::TList(a) => free_type_vars(*a),
        Type::TPair(a, b) => Box::new(free_type_vars(*a).chain(free_type_vars(*b))),
    }
}

// TODO can we take a reference to avoid cloning?
fn run_solve(csts: Vec<Constraint>) -> Result<Subst, TypeError> {
    solver(HashMap::new(), csts)
}

fn solver(subst: Subst, mut csts: Vec<Constraint>) -> Result<Subst, TypeError> {
    match csts.pop() {
        None => Ok(subst),
        Some(Constraint(t1, t2)) => {
            let subst_1 = unifies(t1, t2)?;
            let csts_subbed = csts.into_iter().map(|cst| cst.apply(&subst_1)).collect();
            solver(compose(subst_1, subst), csts_subbed)
        }
    }
}

fn unifies(t1: Type, t2: Type) -> Result<Subst, TypeError> {
    match (t1, t2) {
        (a, b) if a == b => Ok(HashMap::new()),
        (Type::TVar(v), t) => bind(v, t),
        (t, Type::TVar(v)) => bind(v, t),
        (Type::TArr(t1, t2), Type::TArr(t3, t4)) => unify_many(vec![*t1, *t2], vec![*t3, *t4]),
        (Type::TList(t1), Type::TList(t2)) => unifies(*t1, *t2),
        (Type::TPair(t1, t2), Type::TPair(t3, t4)) => unify_many(vec![*t1, *t2], vec![*t3, *t4]),
        (a, b) => Err(TypeError::UnificationFail(a, b)),
    }
}

fn unify_many(mut ts_1: Vec<Type>, mut ts_2: Vec<Type>) -> Result<Subst, TypeError> {
    if ts_1.is_empty() != ts_2.is_empty() {
        Err(TypeError::UnificationMismatch(ts_1, ts_2))
    } else {
        match (ts_1.pop(), ts_2.pop()) {
            (None, None) => Ok(HashMap::new()),
            (Some(t1), Some(t2)) => {
                let subst_1 = unifies(t1, t2)?;
                for t in ts_1.iter_mut() {
                    *t = t.clone().apply(&subst_1)
                }
                for t in ts_2.iter_mut() {
                    *t = t.clone().apply(&subst_1)
                }
                let subst_2 = unify_many(ts_1, ts_2)?;
                Ok(compose(subst_2, subst_1))
            }
            _ => panic!("unify_many: impossible: case handled above"),
        }
    }
}

fn bind(a: TV, t: Type) -> Result<Subst, TypeError> {
    if t.clone() == Type::TVar(a.clone()) {
        Ok(HashMap::new())
    } else if occurs_check(&a, t.clone()) {
        Err(TypeError::InfiniteType(a, t))
    } else {
        let mut hm = HashMap::new();
        hm.insert(a, t);
        Ok(hm)
    }
}

fn occurs_check(a: &TV, t: Type) -> bool {
    t.ftv().contains(&a)
}

fn compose(mut s1: Subst, mut s2: Subst) -> Subst {
    for (_, typ) in s2.iter_mut() {
        *typ = typ.clone().apply(&s1);
    }
    // INFO we want a union which is biased to `s2`. `extend` will overwrite
    // entries in `s1`.
    s1.extend(s2.into_iter());
    s1
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
                .zip(iter::repeat_with(|| is.fresh()))
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

fn infer_lit(lit: &Lit) -> Type {
    match lit {
        Lit::LInt(_) => type_int(),
        Lit::LBool(_) => type_bool(),
    }
}

fn infer_primop(is: &mut InferState, op: &PrimOp) -> Type {
    match op {
        PrimOp::Add => binop_arr(type_int(), type_int()),
        PrimOp::Mul => binop_arr(type_int(), type_int()),
        PrimOp::Sub => binop_arr(type_int(), type_int()),
        PrimOp::Eql => binop_arr(type_int(), type_bool()),
        PrimOp::Null => {
            let tv = is.fresh();
            type_arr(type_list(tv), type_bool())
        }
        PrimOp::Map => {
            let a = is.fresh();
            let b = is.fresh();
            let t_f = type_arr(a.clone(), b.clone());
            let t_ls = type_list(a);
            let t_ret = type_list(b);
            type_arr_multi(vec![t_f, t_ls], t_ret)
        }
        PrimOp::Foldl => {
            let a = is.fresh();
            let b = is.fresh();
            let t_f = type_arr_multi(vec![b.clone(), a.clone()], b.clone());
            let t_ls = type_list(a.clone());
            type_arr_multi(vec![t_f, b.clone(), t_ls], b)
        }
        PrimOp::Pair => {
            let a = is.fresh();
            let b = is.fresh();
            type_arr_multi(vec![a.clone(), b.clone()], type_pair(a, b))
        }
        PrimOp::Fst => {
            let a = is.fresh();
            let b = is.fresh();
            type_arr(type_pair(a.clone(), b), a)
        }
        PrimOp::Snd => {
            let a = is.fresh();
            let b = is.fresh();
            type_arr(type_pair(a, b.clone()), b)
        }
        PrimOp::Cons => {
            let a = is.fresh();
            let ls = type_list(a.clone());
            type_arr_multi(vec![a, ls.clone()], ls.clone())
        }
    }
}

fn binop_arr(arg: Type, ret: Type) -> Type {
    let inner = Type::TArr(Box::new(arg.clone()), Box::new(ret));
    Type::TArr(Box::new(arg), Box::new(inner))
}
