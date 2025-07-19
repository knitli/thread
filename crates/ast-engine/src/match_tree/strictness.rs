// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

use crate::Doc;
pub use crate::matcher::MatchStrictness;
use crate::matcher::{PatternNode, kind_utils};
use crate::meta_var::MetaVariable;
use crate::node::Node;
use std::iter::Peekable;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum MatchOneNode {
    MatchedBoth,
    SkipBoth,
    SkipGoal,
    SkipCandidate,
    NoMatch,
}

fn skip_comment_or_unnamed(n: &Node<impl Doc>) -> bool {
    if !n.is_named() {
        return true;
    }
    let kind = n.kind();
    kind.contains("comment")
}

impl MatchStrictness {
    pub(crate) fn match_terminal(
        &self,
        is_named: bool,
        text: &str,
        goal_kind: u16,
        candidate: &Node<impl Doc>,
    ) -> MatchOneNode {
        let cand_kind = candidate.kind_id();
        let is_kind_matched = kind_utils::are_kinds_matching(goal_kind, cand_kind);
        // work around ast-grep/ast-grep#1419 and tree-sitter/tree-sitter-typescript#306
        // tree-sitter-typescript has wrong span of unnamed node so text would not match
        // just compare kind for unnamed node
        if is_kind_matched && (!is_named || text == candidate.text()) {
            return MatchOneNode::MatchedBoth;
        }
        let (skip_goal, skip_candidate) = match self {
            Self::Cst => (false, false),
            Self::Smart => (false, !candidate.is_named()),
            Self::Ast => (!is_named, !candidate.is_named()),
            Self::Relaxed => (!is_named, skip_comment_or_unnamed(candidate)),
            Self::Signature => {
                if is_kind_matched {
                    return MatchOneNode::MatchedBoth;
                }
                (!is_named, skip_comment_or_unnamed(candidate))
            }
        };
        match (skip_goal, skip_candidate) {
            (true, true) => MatchOneNode::SkipBoth,
            (true, false) => MatchOneNode::SkipGoal,
            (false, true) => MatchOneNode::SkipCandidate,
            (false, false) => MatchOneNode::NoMatch,
        }
    }

    // TODO: this is a method for working around trailing nodes after pattern is matched
    pub(crate) fn should_skip_trailing<D: Doc>(&self, candidate: &Node<D>) -> bool {
        match self {
            Self::Cst | Self::Ast => false,
            Self::Smart => true,
            Self::Relaxed |
            Self::Signature => skip_comment_or_unnamed(candidate),
        }
    }

    pub(crate) fn should_skip_goal<'p>(
        &self,
        goal_children: &mut Peekable<impl Iterator<Item = &'p PatternNode>>,
    ) -> bool {
        while let Some(pattern) = goal_children.peek() {
            let skipped = match self {
                Self::Cst => false,
                Self::Smart => match pattern {
                    PatternNode::MetaVar { meta_var } => match meta_var {
                        MetaVariable::Multiple |
                        MetaVariable::MultiCapture(_) => true,
                        MetaVariable::Dropped(_) |
                        MetaVariable::Capture(..) => false,
                    },
                    PatternNode::Terminal { .. } |
                    PatternNode::Internal { .. } => false,
                },
                Self::Ast | Self::Relaxed | Self::Signature => match pattern {
                    PatternNode::MetaVar { meta_var } => match meta_var {
                        MetaVariable::Multiple | MetaVariable::MultiCapture(_) => true,
                        MetaVariable::Dropped(named) | MetaVariable::Capture(_, named) => !named,
                    },
                    PatternNode::Terminal { is_named, .. } => !is_named,
                    PatternNode::Internal { .. } => false,
                },
            };
            if !skipped {
                return false;
            }
            goal_children.next();
        }
        true
    }
}

impl FromStr for MatchStrictness {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cst" => Ok(Self::Cst),
            "smart" => Ok(Self::Smart),
            "ast" => Ok(Self::Ast),
            "relaxed" => Ok(Self::Relaxed),
            "signature" => Ok(Self::Signature),
            _ => Err("invalid strictness, valid options are: cst, smart, ast, relaxed, signature"),
        }
    }
}
