use super::syntax::{Env, Term, Val};

pub fn interpret(env: &Env, term: &Term) -> Result<Val, String> {
    match term {
        Term::Lam { intro: name, body } => {
            Ok(Val::lam(env.clone(), name.clone(), body.as_ref().clone()))
        }
        Term::Neu {
            applicant,
            arguments,
        } => {
            let arguments = arguments
                .iter()
                .map(|arg| {
                    let val = interpret(env, arg)?;
                    Ok(Box::new(val))
                })
                .collect::<Result<Vec<Box<Val>>, String>>()?;
            let applicant = env.lookup(applicant)?;
            apply(&applicant, arguments)
        }
        Term::Def {
            intro: name,
            binding,
            body,
        } => {
            let binding = interpret(env, binding)?;
            let env = env.extend(name.clone(), Box::new(binding));
            interpret(&env, body)
        }
    }
}

fn apply(applicant: &Val, arguments: Vec<Box<Val>>) -> Result<Val, String> {
    let mut applicant = applicant.clone();
    for argument in &arguments {
        match applicant {
            Val::Lam {
                closure,
                intro: name,
                body,
            } => {
                let closure = closure.extend(name.clone(), argument.clone());
                applicant = interpret(&closure, &body.clone())?
            }
        }
    }
    Ok(applicant)
}

#[cfg(test)]
mod tests {
    use crate::ulc::{
        interpretation::interpret,
        syntax::{term_builder, Env, NameIntro, NameRef, Term, Val},
    };

    fn assert_interpret(env: Env, term: Term, expected_val: &Val) {
        let actual_val = interpret(&Env::default(), &term);
        assert_eq!(
            actual_val.as_ref(),
            Ok(expected_val),
            "\ninput:\n  {}\nactual:\n  {}\nexpected:\n  {}",
            term,
            match &actual_val {
                Ok(v) => format!("{}", v),
                Err(e) => e.clone(),
            },
            &expected_val,
        );
    }

    #[test]
    fn test1() {
        use term_builder::*;
        assert_interpret(
            Env::default(),
            // (def f = λx λy x#1 in (f λz z#0)))
            def(
                "f",
                lam("x", lam("y", var("x"))),
                neu("f", vec![lam("z", var("z"))]),
            )
            .into(),
            // λ[x = λ[f = λ[]x λy x#1]z z#0]y x#1
            &Val::lam(
                Env::from(vec![(
                    NameIntro::new("x"),
                    // λ[f = λ[]x λy x#1]z z#0
                    Val::lam(
                        Env::from(vec![(
                            NameIntro::new("f"),
                            // λy x#1
                            Val::lam(
                                Env::default(),
                                NameIntro::new("x"),
                                Term::lam(
                                    NameIntro::new("y"),
                                    Term::neu(NameRef::new("x", 1), vec![]),
                                ),
                            ),
                        )]),
                        NameIntro::new("z"),
                        Term::neu(NameRef::new("z", 0), vec![]),
                    ),
                )]),
                NameIntro::new("y"),
                // x#1
                Term::neu(NameRef::new("x", 1), vec![]),
            ),
        );
    }

    #[test]
    fn test2() {
        assert_interpret(
            Env::default(),
            Term::lam(NameIntro::new("x"), Term::neu(NameRef::new("x", 0), vec![])),
            &Val::lam(
                Env::default(),
                NameIntro::new("x"),
                Term::neu(NameRef::new("x", 0), vec![]),
            ),
        );
        assert_interpret(
            Env::default(),
            // (def f = λx λy x#1 in (f λz z#0)))
            Term::def(
                NameIntro::new("f"),
                // λx λy x#1
                Term::lam(
                    NameIntro::new("x"),
                    Term::lam(NameIntro::new("y"), Term::neu(NameRef::new("x", 1), vec![])),
                ),
                // (f λz z#0)
                Term::neu(
                    NameRef::new("f", 0),
                    vec![Term::lam(
                        NameIntro::new("z"),
                        Term::neu(NameRef::new("z", 0), vec![]),
                    )],
                ),
            ),
            // λ[x = λ[f = λ[]x λy x#1]z z#0]y x#1
            &Val::lam(
                Env::from(vec![(
                    NameIntro::new("x"),
                    // λ[f = λ[]x λy x#1]z z#0
                    Val::lam(
                        Env::from(vec![(
                            NameIntro::new("f"),
                            // λy x#1
                            Val::lam(
                                Env::default(),
                                NameIntro::new("x"),
                                Term::lam(
                                    NameIntro::new("y"),
                                    Term::neu(NameRef::new("x", 1), vec![]),
                                ),
                            ),
                        )]),
                        NameIntro::new("z"),
                        Term::neu(NameRef::new("z", 0), vec![]),
                    ),
                )]),
                NameIntro::new("y"),
                // x#1
                Term::neu(NameRef::new("x", 1), vec![]),
            ),
        );
    }
}
