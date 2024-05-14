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
    Let {
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

    /// `let <Name> = <Term> in <Term>`
    pub fn let_(intro: NameIntro, binding: Term, body: Term) -> Term {
        Term::Let {
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
            Term::Let {
                intro: name,
                binding,
                body,
            } => {
                write!(f, "(let {} = {} in {})", name, binding, body)
            }
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
