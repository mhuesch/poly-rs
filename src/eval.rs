use pretty::RcDoc;
use std::collections::HashMap;

use super::syntax::{Expr, Lit, Name, PrimOp};
use crate::{app, lam};

#[derive(Clone)]
pub enum Value {
    VInt(i64),
    VBool(bool),
    VClosure(Name, Box<Expr>, TermEnv),
}

type TermEnv = HashMap<Name, Value>;

impl Value {
    pub fn ppr(&self) -> RcDoc<()> {
        match self {
            VInt(n) => RcDoc::as_string(n),
            VBool(true) => RcDoc::text("true"),
            VBool(false) => RcDoc::text("false"),
            VClosure(_, _, _) => RcDoc::text("<<closure>>"),
        }
    }
}

struct EvalState(u64);

impl EvalState {
    fn new() -> EvalState {
        EvalState(0)
    }

    fn fresh(&mut self) -> Name {
        let cnt = match self {
            EvalState(c) => {
                *c += 1;
                *c
            }
        };
        let s = format!("_{}", cnt);
        Name(s)
    }
}

pub fn eval(expr: &Expr) -> Value {
    let env = HashMap::new();
    let mut es = EvalState::new();
    eval_(&env, &mut es, expr)
}

use Value::*;
fn eval_(env: &TermEnv, es: &mut EvalState, expr: &Expr) -> Value {
    match find_prim_app(&expr, false) {
        // in this case we directly interpret the PrimOp.
        Some((op, args)) => {
            let args_v: Vec<Value> = args
                .iter()
                .map(|arg| eval_(env, es, &arg.clone()))
                .collect();
            match op {
                PrimOp::Add => match (&args_v[0], &args_v[1]) {
                    (VInt(a_), VInt(b_)) => VInt(a_ + b_),
                    _ => panic!("+: bad types"),
                },
                PrimOp::Sub => match (&args_v[0], &args_v[1]) {
                    (VInt(a_), VInt(b_)) => VInt(a_ - b_),
                    _ => panic!("-: bad types"),
                },
                PrimOp::Mul => match (&args_v[0], &args_v[1]) {
                    (VInt(a_), VInt(b_)) => VInt(a_ * b_),
                    _ => panic!("*: bad types"),
                },
                PrimOp::Eql => match (&args_v[0], &args_v[1]) {
                    (VInt(a_), VInt(b_)) => VBool(a_ == b_),
                    _ => panic!("==: bad types"),
                },
            }
        }

        // we do not find a direct PrimOp application, so we interpret normally.
        None => match expr {
            Expr::Lit(Lit::LInt(x)) => VInt(*x),
            Expr::Lit(Lit::LBool(x)) => VBool(*x),

            Expr::Var(x) => match env.get(&x) {
                None => panic!("impossible: free variable: {:?}", x),
                Some(v) => v.clone(),
            },

            Expr::Lam(nm, bd) => VClosure(nm.clone(), bd.clone(), env.clone()),

            Expr::Let(x, e, bd) => {
                let e_v = eval_(env, es, e);
                let mut new_env = env.clone();
                new_env.insert(x.clone(), e_v);
                eval_(&new_env, es, bd)
            }

            Expr::If(tst, thn, els) => match eval_(env, es, tst) {
                VBool(true) => eval_(env, es, thn),
                VBool(false) => eval_(env, es, els),
                _ => panic!("impossible: non-bool in test position of if"),
            },

            // this represents a PrimOp that is not in application position.
            // since it is then being used as an argument (or being bound), we
            // must package it into a closure so it can be used "lifted".
            //
            // note that we assume these are arity-2 PrimOps - an assumption
            // which likely will not hold in the future.
            Expr::Prim(op) => {
                let nm1 = es.fresh();
                let nm2 = es.fresh();
                let bd = app!(
                    app!(Expr::Prim(op.clone()), Expr::Var(nm1.clone())),
                    Expr::Var(nm2.clone())
                );
                let inner = lam!(nm2, bd);
                VClosure(nm1, Box::new(inner), HashMap::new())
            }

            Expr::App(fun, arg) => match eval_(env, es, fun) {
                VClosure(nm, bd, clo) => {
                    let arg_v = eval_(env, es, arg);
                    let mut new_env = clo.clone();
                    new_env.insert(nm, arg_v);
                    eval_(&new_env, es, &bd)
                }
                _ => panic!("impossible: non-closure in function position of app"),
            },

            Expr::Fix(e) => eval_(
                env,
                es,
                &Expr::App(e.clone(), Box::new(Expr::Fix(e.clone()))),
            ),
        },
    }
}

/// look for a "direct application" of a PrimOp. this means that it is nested
/// within some number of `App`s, with no intervening other constructors. we
/// want to find this because we can directly interpret these, rather than
/// packaging them into closures.
///
/// we must take the `in_app` boolean so that we can differentiate "bare"
/// PrimOps, which are not inside an `App`, from enclosed ones.
fn find_prim_app(expr: &Expr, in_app: bool) -> Option<(PrimOp, Vec<Expr>)> {
    match expr {
        Expr::App(fun, arg) => {
            let (op, mut args) = find_prim_app(fun, true)?;
            args.push(*arg.clone());
            Some((op, args))
        }
        Expr::Prim(op) if in_app => Some((op.clone(), Vec::new())),
        _ => None,
    }
}
