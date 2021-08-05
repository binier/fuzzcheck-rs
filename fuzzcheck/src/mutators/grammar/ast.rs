extern crate self as fuzzcheck;

/// An abstract syntax tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AST {
    Token(char),
    Sequence(Vec<AST>),
    Box(Box<AST>),
}

/// Like an abstract syntax tree, but augmented with the string indices that correspond to each node
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ASTMap {
    pub start_index: usize,
    pub len: usize,
    pub content: ASTMappingKind,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ASTMappingKind {
    Token,
    Sequence(Vec<ASTMap>),
    Box(Box<ASTMap>),
}

impl AST {
    #[no_coverage]
    pub fn generate_string_in(&self, s: &mut String, start_index: &mut usize) -> ASTMap {
        match self {
            AST::Token(c) => {
                let len = c.len_utf8();
                let orig_start_index = *start_index;
                s.push(*c);
                *start_index += len;
                ASTMap {
                    start_index: orig_start_index,
                    len,
                    content: ASTMappingKind::Token,
                }
            }
            AST::Sequence(asts) => {
                let original_start_idx = *start_index;
                let mut cs = vec![];
                for ast in asts {
                    let c = ast.generate_string_in(s, start_index);
                    cs.push(c);
                }
                ASTMap {
                    start_index: original_start_idx,
                    len: *start_index - original_start_idx,
                    content: ASTMappingKind::Sequence(cs),
                }
            }
            AST::Box(ast) => {
                let mapping = ast.generate_string_in(s, start_index);
                ASTMap {
                    start_index: mapping.start_index,
                    len: mapping.len,
                    content: ASTMappingKind::Box(Box::new(mapping)),
                }
            }
        }
    }
    #[no_coverage]
    pub fn generate_string(&self) -> (String, ASTMap) {
        let mut s = String::new();
        let mut start_index = 0;
        let c = self.generate_string_in(&mut s, &mut start_index);
        (s, c)
    }
    #[no_coverage]
    pub fn generate_string_starting_at_idx(&self, idx: usize) -> (String, ASTMap) {
        let mut s = String::new();
        let mut start_index = idx;
        let c = self.generate_string_in(&mut s, &mut start_index);
        (s, c)
    }
}

impl From<&AST> for ASTMap {
    #[no_coverage]
    fn from(ast: &AST) -> Self {
        ast.generate_string().1
    }
}
impl From<AST> for String {
    #[no_coverage]
    fn from(ast: AST) -> Self {
        ast.generate_string().0
    }
}