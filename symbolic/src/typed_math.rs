//! Compilation-local typed MathIR candidates. These values do not authenticate
//! source declarations, admit a structure, or represent checked evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermType { Integer, IntegerPair }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Integer(i64),
    Parameter { definition: usize, pin: usize },
    Pair(Box<Term>, Box<Term>),
    Select { target: Box<Term>, index: usize },
    Apply { definition: usize, argument: Box<Term> },
    Add(Box<Term>, Box<Term>),
    Subtract(Box<Term>, Box<Term>),
}

impl Term {
    /// Check only structural types against the supplied signature candidates.
    /// The source owner must independently bind each index to an actual definition.
    pub fn term_type(&self, signatures: &[(TermType, TermType)]) -> Option<TermType> {
        self.type_bounded(signatures, 0, &mut 512)
    }
    fn type_bounded(&self, signatures: &[(TermType, TermType)], depth: usize,
        budget: &mut usize) -> Option<TermType> {
        if depth > 32 || *budget == 0 { return None; }
        *budget -= 1;
        Some(match self {
            Self::Integer(_) => TermType::Integer,
            Self::Parameter { definition, pin: 0 } => signatures.get(*definition)?.0,
            Self::Parameter { .. } => return None,
            Self::Pair(a,b) => {
                if a.type_bounded(signatures,depth+1,budget)? != TermType::Integer
                    || b.type_bounded(signatures,depth+1,budget)? != TermType::Integer { return None; }
                TermType::IntegerPair
            },
            Self::Select {target,index} => {
                if *index>1 || target.type_bounded(signatures,depth+1,budget)? != TermType::IntegerPair { return None; }
                TermType::Integer
            },
            Self::Apply {definition,argument} => {
                let (input,output)=signatures.get(*definition)?;
                if argument.type_bounded(signatures,depth+1,budget)? != *input { return None; }
                *output
            },
            Self::Add(a,b) | Self::Subtract(a,b) => {
                if a.type_bounded(signatures,depth+1,budget)? != TermType::Integer
                    || b.type_bounded(signatures,depth+1,budget)? != TermType::Integer { return None; }
                TermType::Integer
            },
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Goal {
    Defined { occurrence: usize, term: Term },
    Equal { left: Term, right: Term },
}

/// A role candidate, not a public numeric profile or proof of success.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericRole { CheckedDimensionlessIntegerAdd, CheckedDimensionlessIntegerSubtract }
