use std::rc::Rc;

use crate::mutators::grammar::Grammar;
use regex_syntax::hir::{Class, HirKind, Literal, RepetitionKind, RepetitionRange};

use crate::mutators::grammar::{alternation, concatenation, literal, literal_ranges, repetition};

#[no_coverage]
pub(crate) fn grammar_from_regex(regex: &str) -> Rc<Grammar> {
    let mut parser = regex_syntax::Parser::new();
    let hir = parser.parse(regex).unwrap();
    grammar_from_regex_hir_kind(hir.kind())
}
#[no_coverage]
pub fn grammar_from_regex_hir_kind(hir: &HirKind) -> Rc<Grammar> {
    match hir {
        HirKind::Empty => panic!("emoty regexes are not supported"),
        HirKind::Literal(l) => match l {
            Literal::Unicode(l) => literal(*l),
            Literal::Byte(_) => panic!("non-unicode regexes are not supported"),
        },
        HirKind::Class(class) => match class {
            Class::Unicode(class) => {
                let ranges = class
                    .ranges()
                    .iter()
                    .map(
                        #[no_coverage]
                        |r| r.start()..=r.end(),
                    )
                    .collect::<Vec<_>>();
                literal_ranges(ranges)
            }
            Class::Bytes(_) => panic!("non-unicode regexes are not supported"),
        },
        HirKind::Anchor(_) => panic!("anchors are not supported"),
        HirKind::WordBoundary(_) => panic!("word boundaries are not supported"),
        HirKind::Repetition(rep) => {
            let range = match rep.kind.clone() {
                RepetitionKind::ZeroOrOne => 0..=1u32,
                RepetitionKind::ZeroOrMore => 0..=u32::MAX,
                RepetitionKind::OneOrMore => 1..=u32::MAX,
                RepetitionKind::Range(range) => match range {
                    RepetitionRange::Exactly(n) => n..=n,
                    RepetitionRange::AtLeast(n) => n..=u32::MAX,
                    RepetitionRange::Bounded(n, m) => n..=m,
                },
            };
            let range = (*range.start() as usize)..=(*range.end() as usize);
            let grammar = grammar_from_regex_hir_kind(rep.hir.kind());
            repetition(grammar, range)
        }
        HirKind::Group(group) => grammar_from_regex_hir_kind(group.hir.kind()),
        HirKind::Concat(concat) => concatenation(concat.iter().map(
            #[no_coverage]
            |hir| grammar_from_regex_hir_kind(hir.kind()),
        )),
        HirKind::Alternation(alt) => alternation(alt.iter().map(
            #[no_coverage]
            |hir| grammar_from_regex_hir_kind(hir.kind()),
        )),
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::Mutator;

//     use super::*;
//     #[no_coverage]
//     #[test]
//     fn t() {
//         let s = "[0-9]{4}-[0-9]{2}-[0-9]{2}";
//         let g = grammar_from_regex(s);
//         println!("{:?}", g);
//         let mutator = crate::mutators::grammar::grammar_based_string_mutator(g);
//         let (s, cplx) = mutator.random_arbitrary(1000.0);
//         println!("\n{}\n{}", s, cplx);
//     }
// }
