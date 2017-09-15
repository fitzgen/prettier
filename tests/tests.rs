extern crate prettier;

mod tree {
    use prettier::*;
    use std::rc::Rc;

    pub struct Tree(&'static str, pub Vec<Tree>);

    pub fn show(t: &Tree) -> Doc {
        group(Rc::new(
            concat(text(t.0), nest(t.0.len(), show_bracket(&t.1[..]))),
        ))
    }

    fn show_bracket(t: &[Tree]) -> Doc {
        if t.is_empty() {
            nil()
        } else {
            concat(text("["), concat(nest(1, show_trees(t)), text("]")))
        }
    }

    fn show_trees(t: &[Tree]) -> Doc {
        assert!(!t.is_empty());
        if t.len() == 1 {
            show(&t[0])
        } else {
            concat(
                show(&t[0]),
                concat(text(","), concat(line(), show_trees(&t[1..]))),
            )
        }
    }

    impl Default for Tree {
        fn default() -> Tree {
            Tree(
                "aaa",
                vec![
                    Tree("bbbbb", vec![Tree("ccc", vec![]), Tree("dd", vec![])]),
                    Tree("eee", vec![]),
                    Tree(
                        "ffff",
                        vec![Tree("gg", vec![]), Tree("hhh", vec![]), Tree("ii", vec![])],
                    ),
                ],
            )
        }
    }

    #[test]
    fn test1() {
        let expected = "\
aaa[bbbbb[ccc, dd],
    eee,
    ffff[gg, hhh, ii]]";
        assert_eq!(pretty(30, show(&Tree::default())), expected);
    }

    #[test]
    fn test2() {
        let expected = "\
aaa[bbbbb[ccc,
          dd],
    eee,
    ffff[gg,
         hhh,
         ii]]";
        assert_eq!(pretty(10, show(&Tree::default())), expected);
    }
}

// TODO: xml examples?
