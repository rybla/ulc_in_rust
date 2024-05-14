use core::fmt;
use std::fmt::{Display, Formatter};

// ================================================================================
/// ## Term
// ================================================================================

#[derive(Clone, PartialEq, Debug)]
pub enum Term {
    Lam {
        intro: NameIntro,
        body: Box<Term>,
    },
    Neu {
        applicant: NameRef,
        arguments: Vec<Box<Term>>,
    },
    Def {
        intro: NameIntro,
        binding: Box<Term>,
        body: Box<Term>,
    },
}

impl Term {
    /// `λ<NameIntro> <Name>`
    pub fn lam(intro: NameIntro, body: Term) -> Term {
        Term::Lam {
            intro,
            body: Box::new(body),
        }
    }

    /// `<Name> <Term> ... <Term>`
    pub fn neu(applicant: NameRef, arguments: Vec<Term>) -> Term {
        Term::Neu {
            applicant,
            arguments: arguments.into_iter().map(Box::new).collect(),
        }
    }

    /// `<Name>`
    pub fn var(name: NameRef) -> Term {
        Term::Neu {
            applicant: name,
            arguments: vec![],
        }
    }

    /// `def <Name> = <Term> in <Term>`
    pub fn def(intro: NameIntro, binding: Term, body: Term) -> Term {
        Term::Def {
            intro,
            binding: Box::new(binding),
            body: Box::new(body),
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Term::Lam { intro: name, body } => write!(f, "λ{} {}", name, body),
            Term::Neu {
                applicant,
                arguments,
            } => {
                if arguments.is_empty() {
                    write!(f, "{}", applicant)
                } else {
                    write!(f, "(")?;
                    write!(f, "{}", applicant)?;
                    for argument in arguments.iter() {
                        write!(f, " {}", argument)?;
                    }
                    write!(f, ")")
                }
            }
            Term::Def {
                intro: name,
                binding,
                body,
            } => {
                write!(f, "(def {} = {} in {})", name, binding, body)
            }
        }
    }
}

// ================================================================================
/// ## TermBuilder
// ================================================================================

#[derive(Clone, PartialEq, Debug)]
pub enum TermBuilder {
    Lam {
        name: String,
        body: Box<TermBuilder>,
    },
    Neu {
        applicant: (String, Option<usize>),
        arguments: Vec<Box<TermBuilder>>,
    },
    Def {
        name: String,
        binding: Box<TermBuilder>,
        body: Box<TermBuilder>,
    },
}

pub mod term_builder {
    use super::TermBuilder;

    pub fn lam(name: &str, body: TermBuilder) -> TermBuilder {
        TermBuilder::Lam {
            name: name.to_string(),
            body: Box::new(body),
        }
    }

    pub fn neu(applicant: &str, arguments: Vec<TermBuilder>) -> TermBuilder {
        TermBuilder::Neu {
            applicant: (applicant.to_string(), None),
            arguments: arguments.into_iter().map(Box::new).collect(),
        }
    }

    pub fn neu_with_index(
        applicant: &str,
        index: usize,
        arguments: Vec<TermBuilder>,
    ) -> TermBuilder {
        TermBuilder::Neu {
            applicant: (applicant.to_string(), Some(index)),
            arguments: arguments.into_iter().map(Box::new).collect(),
        }
    }

    pub fn var(name: &str) -> TermBuilder {
        neu(name, vec![])
    }

    pub fn var_with_index(name: &str, index: usize) -> TermBuilder {
        neu_with_index(name, index, vec![])
    }

    pub fn def(name: &str, binding: TermBuilder, body: TermBuilder) -> TermBuilder {
        TermBuilder::Def {
            name: name.to_string(),
            binding: Box::new(binding),
            body: Box::new(body),
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::ulc::syntax::{NameIntro, NameRef, Term};

        use super::*;

        #[test]
        fn test_var() {
            assert_eq!(
                Term::from(lam("x", var("x"))),
                Term::lam(NameIntro::new("x"), Term::var(NameRef::new("x", 0)))
            );
            assert_eq!(
                Term::from(def("x", lam("y", var("y")), var("x"))),
                Term::def(
                    NameIntro::new("x"),
                    Term::lam(NameIntro::new("y"), Term::var(NameRef::new("y", 0))),
                    Term::var(NameRef::new("x", 0))
                )
            );
            assert_eq!(
                Term::from(def("x", lam("y", var("y")), neu("x", vec![var("x")]))),
                Term::def(
                    NameIntro::new("x"),
                    Term::lam(NameIntro::new("y"), Term::var(NameRef::new("y", 0))),
                    Term::neu(NameRef::new("x", 0), vec![Term::var(NameRef::new("x", 0))])
                )
            );
        }
    }
}

impl From<TermBuilder> for Term {
    fn from(term: TermBuilder) -> Self {
        from_term_builder_to_term(vec![], &term).unwrap()
    }
}

fn from_term_builder_to_term(ctx: Vec<String>, term: &TermBuilder) -> Result<Term, String> {
    match term {
        TermBuilder::Lam { name, body } => Ok(Term::lam(
            NameIntro::new(&name),
            from_term_builder_to_term(
                {
                    let mut ctx = ctx.clone();
                    ctx.insert(0, name.clone());
                    ctx
                },
                body,
            )?,
        )),
        TermBuilder::Neu {
            applicant,
            arguments,
        } => {
            let (name, index) = applicant;
            let applicant = match index {
                Some(index) => {
                    // check that the intro at that index actually does have the name
                    let name_actual = ctx
                        .get(*index)
                        .ok_or(format!("the name ref with label `{}` and index `{}` is invalid in the context `{:?}`",  name, index,  ctx).as_str())?;
                    if name != name_actual {
                        return Err(format!("the name ref with label `{}` and index `{}` is invalid in the context `{:?}`",  name, index,  ctx));
                    }
                    NameRef::new(name, *index)
                }
                None => {
                    // find the index of the name in the context
                    let index = ctx
                        .iter()
                        .position(|name_actual| name == name_actual)
                        .ok_or(
                            format!(
                                "the name ref with label `{}` is invalid in the context `{:?}`",
                                name, ctx
                            )
                            .as_str(),
                        )?;
                    NameRef::new(name, index)
                }
            };
            Ok(Term::neu(
                applicant,
                arguments
                    .iter()
                    .map(|arg| from_term_builder_to_term(ctx.clone(), arg))
                    .collect::<Result<Vec<Term>, String>>()?,
            ))
        }
        TermBuilder::Def {
            name,
            binding,
            body,
        } => {
            let intro = NameIntro::new(name);
            Ok(Term::def(
                intro.clone(),
                from_term_builder_to_term(ctx.clone(), binding)?,
                from_term_builder_to_term(
                    {
                        let mut ctx = ctx.clone();
                        ctx.insert(0, name.clone());
                        ctx
                    },
                    body,
                )?,
            ))
        }
    }
}

// ================================================================================
/// ## Val
// ================================================================================

#[derive(Clone, PartialEq, Debug)]
pub enum Val {
    Lam {
        intro: NameIntro,
        body: Box<Term>,
        closure: Box<Env>,
    },
}

impl Val {
    /// `λ<Env><Name> <Term>`
    pub fn lam(closure: Env, intro: NameIntro, body: Term) -> Val {
        Val::Lam {
            intro,
            body: Box::new(body),
            closure: Box::new(closure),
        }
    }
}

impl Display for Val {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Val::Lam {
                intro: name,
                body,
                closure,
            } => write!(f, "λ{}{} {}", closure, name, body),
        }
    }
}

// ================================================================================
/// ## NameIntro
// ================================================================================

#[derive(Clone, PartialEq, Debug)]
pub struct NameIntro {
    pub label: String,
}

impl NameIntro {
    pub fn new(label: &str) -> NameIntro {
        NameIntro {
            label: label.to_string(),
        }
    }
}

impl Display for NameIntro {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

// ================================================================================
/// ## NameRef
// ================================================================================

#[derive(Clone, PartialEq, Debug)]
pub struct NameRef {
    label: String,
    index: usize,
}

impl NameRef {
    pub fn new(label: &str, index: usize) -> NameRef {
        NameRef {
            label: label.to_string(),
            index,
        }
    }
}

impl Display for NameRef {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}#{}", self.label, self.index)
    }
}

// ================================================================================
/// ## Env
// ================================================================================

#[derive(Clone, PartialEq, Debug)]
pub struct Env {
    bindings: Vec<(NameIntro, Box<Val>)>,
}

impl From<Vec<(NameIntro, Val)>> for Env {
    fn from(bindings: Vec<(NameIntro, Val)>) -> Env {
        Env {
            bindings: bindings
                .into_iter()
                .map(|(name, val)| (name, Box::new(val)))
                .collect(),
        }
    }
}

impl Env {
    pub fn extend(&self, intro: NameIntro, val: Box<Val>) -> Env {
        let mut bindings = self.bindings.clone();
        bindings.insert(0, (intro, val));
        Env { bindings }
    }

    pub fn lookup(&self, x: &NameRef) -> Result<Box<Val>, String> {
        if let Some((y, v)) = self.bindings.get(x.index) {
            if y.label == x.label {
                Ok(v.clone())
            } else {
                Err(format!(
                    "environment's binding at index `{}` was expected to have the name `{}` but it actually has the name `{}`",
                    x.index, x.label, y.label
                ))
            }
        } else {
            Err(format!(
                "environment doesn't have binding at index `{}` of name `{}`",
                x.index, x.label
            ))
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(NameIntro, Box<Val>)> {
        self.bindings.iter()
    }
}

impl Default for Env {
    fn default() -> Env {
        Env { bindings: vec![] }
    }
}

impl Display for Env {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, (name, val)) in self.bindings.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{} = {}", name, val)?;
        }
        write!(f, "]")
    }
}
