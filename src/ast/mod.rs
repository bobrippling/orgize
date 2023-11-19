#[rustfmt::skip]
mod generated;


mod affiliated_keyword;
mod drawer;
mod entity;
mod headline;
mod inline_call;
mod link;
mod list;
mod planning;
mod snippet;
mod table;
mod timestamp;

pub use generated::*;
pub use rowan::ast::support::*;
pub use timestamp::*;

use crate::syntax::{SyntaxKind, SyntaxNode};
use rowan::{ast::AstNode, Language, NodeOrToken};

pub fn blank_lines(parent: &SyntaxNode) -> usize {
    parent
        .children_with_tokens()
        .filter(|n| n.kind() == SyntaxKind::BLANK_LINE)
        .count()
}

pub fn last_child<N: AstNode>(parent: &rowan::SyntaxNode<N::Language>) -> Option<N> {
    parent.children().filter_map(N::cast).last()
}

pub fn last_token<L: Language>(
    parent: &rowan::SyntaxNode<L>,
    kind: L::Kind,
) -> Option<rowan::SyntaxToken<L>> {
    parent
        .children_with_tokens()
        .filter_map(filter_token(kind))
        .last()
}

pub fn filter_token<L: Language>(
    kind: L::Kind,
) -> impl Fn(NodeOrToken<rowan::SyntaxNode<L>, rowan::SyntaxToken<L>>) -> Option<rowan::SyntaxToken<L>>
{
    move |elem| match elem {
        NodeOrToken::Token(tk) if tk.kind() == kind => Some(tk),
        _ => None,
    }
}
