#![feature(test)]
extern crate test;

include!("../tests/tests.rs");

// TODO: This should hopefully get faster after switching to lazy thunks and persistent lists

#[bench]
fn bench_tree(b: &mut test::Bencher) {
    let mut tree = tree::Tree::default();
    tree.1.extend(vec![
        tree::Tree::default(),
        tree::Tree::default(),
        tree::Tree::default(),
        tree::Tree::default(),
    ]);

    b.iter(|| {
        test::black_box(tree::show(&tree));
    });
}
