use crate::matcher::Matcher;
use crate::meta_var::{is_valid_meta_var_char, MetaVariableID, Underlying};
use crate::{Doc, Node, NodeMatch, Root};
use std::ops::Range;

pub(crate) use indent::formatted_slice;

use crate::source::Edit as E;
type Edit<D> = E<<D as Doc>::Source>;

mod indent;
mod structural;
mod template;

pub use crate::source::Content;
pub use template::{TemplateFix, TemplateFixError};

