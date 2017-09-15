use std::fmt;
use std::iter;
use std::rc::Rc;

fn unbox<T: Clone>(rc: Rc<T>) -> T {
    match Rc::try_unwrap(rc) {
        Ok(t) => t,
        Err(rc) => (*rc).clone(),
    }
}

// TODO: make everything into thunks

// `DOC` in the paper; this should techinically be opaque to API consumers to
// force preservation of the `Group` invariant.
#[derive(Debug, Clone)]
pub enum Doc {
    Nil,
    Concat(Rc<Doc>, Rc<Doc>),
    Nest(usize, Rc<Doc>),
    Text(String),
    Line,
    // Invariant: in Group(x, y), flatten(x) == flatten(y)
    Group(Rc<Doc>, Rc<Doc>),
}

use Doc::*;

// `Doc` in the paper
#[derive(Debug, Clone)]
pub enum LowDoc {
    Nil,
    Text(String, Rc<LowDoc>),
    Line(usize, Rc<LowDoc>),
}

pub fn nil() -> Doc {
    Nil
}

pub fn concat(x: Doc, y: Doc) -> Doc {
    Concat(Rc::new(x), Rc::new(y))
}

pub fn nest(i: usize, x: Doc) -> Doc {
    Nest(i, Rc::new(x))
}

pub fn text<S: Into<String>>(s: S) -> Doc {
    Text(s.into())
}

pub fn line() -> Doc {
    Line
}

pub fn group(x: Rc<Doc>) -> Doc {
    Group(Rc::new(flatten((*x).clone())), x)
}

fn flatten(x: Doc) -> Doc {
    match x {
        Nil => Nil,
        Text(s) => Text(s),
        Line => text(" "),
        Concat(x, y) => {
            let x = unbox(x);
            let y = unbox(y);
            concat(flatten(x), flatten(y))
        }
        Nest(i, x) => {
            let x = unbox(x);
            nest(i, flatten(x))
        }
        Group(x, _) => {
            let x = unbox(x);
            flatten(x)
        }
    }
}

// `layout` in the paper
impl fmt::Display for LowDoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LowDoc::Nil => Ok(()),
            LowDoc::Text(ref s, ref d) => {
                write!(f, "{}", s)?;
                write!(f, "{}", d)
            }
            LowDoc::Line(i, ref x) => {
                writeln!(f)?;
                for _ in 0..i {
                    write!(f, " ")?;
                }
                write!(f, "{}", x)
            }
        }
    }
}

fn best(width: usize, x: Doc) -> LowDoc {
    best_helper(width, 0, vec![(0, x)])
}

// TODO: persistent list for `options` rather than a vec

fn best_helper(width: usize, used: usize, mut options: Vec<(usize, Doc)>) -> LowDoc {
    match options.pop() {
        None => LowDoc::Nil,
        Some((_, Nil)) => best_helper(width, used, options),
        Some((i, Concat(x, y))) => best_helper(width, used, {
            options.push((i, unbox(y)));
            options.push((i, unbox(x)));
            options
        }),
        Some((i, Nest(j, x))) => best_helper(width, used, {
            options.push((i + j, unbox(x)));
            options
        }),
        Some((_, Text(s))) => {
            let s_len = s.len();
            LowDoc::Text(s, Rc::new(best_helper(width, used + s_len, options)))
        }
        Some((i, Line)) => LowDoc::Line(i, Rc::new(best_helper(width, i, options))),
        Some((i, Group(x, y))) => better(
            width,
            used,
            best_helper(width, used, {
                let mut options = options.clone();
                options.push((i, unbox(x)));
                options
            }),
            best_helper(width, used, {
                options.push((i, unbox(y)));
                options
            }),
        ),
    }
}

fn better(width: usize, used: usize, x: LowDoc, y: LowDoc) -> LowDoc {
    if fits(width, used, &x) {
        x
    } else {
        y
    }
}

fn fits(width: usize, used: usize, x: &LowDoc) -> bool {
    if used > width {
        return false;
    }

    match *x {
        LowDoc::Nil | LowDoc::Line(..) => true,
        LowDoc::Text(ref s, ref x) => fits(width, used + s.len(), x),
    }
}

pub fn pretty(width: usize, x: Doc) -> String {
    best(width, x).to_string()
}

pub mod utils {
    use super::*;

    // <+>
    pub fn space_concat(x: Doc, y: Doc) -> Doc {
        concat(x, concat(text(" "), y))
    }

    // </>
    pub fn line_concat(x: Doc, y: Doc) -> Doc {
        concat(x, concat(line(), y))
    }

    pub fn spread<I>(iter: I) -> Doc
    where
        I: IntoIterator<Item = Doc>,
    {
        iter.into_iter().fold(Nil, space_concat)
    }

    pub fn stack<I>(iter: I) -> Doc
    where
        I: IntoIterator<Item = Doc>,
    {
        iter.into_iter().fold(Nil, line_concat)
    }

    // <+/>
    pub fn space_or_line_concat(x: Doc, y: Doc) -> Doc {
        concat(x, concat(group(Rc::new(line())), y))
    }

    pub fn fill_words<S: AsRef<str>>(words: S) -> Doc {
        words
            .as_ref()
            .split_whitespace()
            .map(text)
            .fold(Nil, space_or_line_concat)
    }

    pub fn fill<I>(iter: I) -> Doc
    where
        I: IntoIterator<Item = Doc>,
        <I as IntoIterator>::IntoIter: Clone,
    {
        let mut iter = iter.into_iter();
        match (iter.next(), iter.next()) {
            (None, None) => Nil,
            (Some(x), None) => x,
            (Some(x), Some(y)) => space_or_line_concat(
                space_concat(
                    flatten(x.clone()),
                    fill(iter::once(flatten(y.clone())).chain(iter.clone())),
                ),
                line_concat(x, fill(iter::once(y).chain(iter))),
            ),
            (None, Some(_)) => panic!("non-fused iterator"),
        }
    }
}
