#[cfg(feature = "bench")]
extern crate rand;

#[cfg(feature = "bench")]
extern crate test;

#[cfg(feature = "bench")]
use self::test::Bencher;

#[cfg(feature = "bench")]
use self::rand::{random, XorShiftRng, Rng};

use self::fixtures::{QuadTreeRegion, Vec2};
use {NTree, Region};

#[test]
fn test_contains() {
    let ntree = NTree::new(QuadTreeRegion::square(0.0, 0.0, 100.0), 4);
    assert!(ntree.contains(&Vec2 { x: 50.0, y: 50.0 }));
}

#[test]
fn test_insert() {
    let mut ntree = NTree::new(QuadTreeRegion::square(0.0, 0.0, 100.0), 4);
    assert!(ntree.insert(Vec2 { x: 50.0, y: 50.0 }));
    assert_eq!(ntree.nearby(&Vec2 { x: 40.0, y: 40.0 }), Some(&[Vec2 { x: 50.0, y: 50.0 }] as &[_]));
}

#[test]
fn test_nearby() {
    let mut ntree = NTree::new(QuadTreeRegion::square(0.0, 0.0, 100.0), 4);

    // Bottom left corner
    ntree.insert(Vec2 { x: 30.0, y: 30.0 });
    ntree.insert(Vec2 { x: 20.0, y: 20.0 });
    ntree.insert(Vec2 { x: 10.0, y: 10.0 });

    // Top right corner
    ntree.insert(Vec2 { x: 75.0, y: 75.0 });

    // Top left corner
    ntree.insert(Vec2 { x: 40.0, y: 70.0 });

    // Bottom right corner
    ntree.insert(Vec2 { x: 80.0, y: 20.0 });

    // Bottom left corner
    assert_eq!(ntree.nearby(&Vec2 { x: 40.0, y: 40.0 }),
        Some(&[Vec2 { x: 30.0, y: 30.0 },
              Vec2 { x: 20.0, y: 20.0 },
              Vec2 { x: 10.0, y: 10.0 }] as &[_]));

    // Top right corner
    assert_eq!(ntree.nearby(&Vec2 { x: 90.0, y: 90.0 }), Some(&[Vec2 { x: 75.0, y: 75.0 }] as &[_]));

    // Top left corner
    assert_eq!(ntree.nearby(&Vec2 { x: 20.0, y: 80.0 }), Some(&[Vec2 { x: 40.0, y: 70.0 }] as &[_]));

    // Bottom right corner
    assert_eq!(ntree.nearby(&Vec2 { x: 94.0, y: 12.0 }), Some(&[Vec2 { x: 80.0, y: 20.0 }] as &[_]));
}

#[test]
fn test_range_query() {
    let mut ntree = NTree::new(QuadTreeRegion::square(0.0, 0.0, 100.0), 4);

    // Inside (y < 40)
    ntree.insert(Vec2 { x: 30.0, y: 30.0 });
    ntree.insert(Vec2 { x: 20.0, y: 20.0 });
    ntree.insert(Vec2 { x: 10.0, y: 10.0 });
    ntree.insert(Vec2 { x: 60.0, y: 20.0 });

    // Outside (y > 40)
    ntree.insert(Vec2 { x: 60.0, y: 59.0 });
    ntree.insert(Vec2 { x: 60.0, y: 45.0 });

    assert_eq!(ntree.range_query(&QuadTreeRegion { x: 0.0, y: 0.0, width: 100.0, height: 40.0 })
                   .map(|x| x.clone()).collect::<Vec<Vec2>>(),
               vec![Vec2 { x: 30.0, y: 30.0 },
                    Vec2 { x: 20.0, y: 20.0 },
                    Vec2 { x: 10.0, y: 10.0 },
                    Vec2 { x: 60.0, y: 20.0 }]);
}

#[cfg(feature = "bench")]
fn range_query_bench(b: &mut Bencher, n: usize) {
    let mut rng: XorShiftRng = random();

    let mut ntree = NTree::new(QuadTreeRegion::square(0.0, 0.0, 1.0), 4);
    for _ in 0..n {
        ntree.insert(Vec2 { x: rng.gen(), y: rng.gen() });
    }

    b.iter(|| {
        let r = QuadTreeRegion {
            x: rng.gen(),
            y: rng.gen(),
            width: rng.gen(),
            height: rng.gen()
        };

        for p in ntree.range_query(&r) { test::black_box(p); }
    })
}

#[cfg(feature = "bench")]
#[bench]
fn bench_range_query_small(b: &mut Bencher) {
    range_query_bench(b, 10);
}

#[cfg(feature = "bench")]
#[bench]
fn bench_range_query_medium(b: &mut Bencher) {
    range_query_bench(b, 100);
}

#[cfg(feature = "bench")]
#[bench]
fn bench_range_query_large(b: &mut Bencher) {
    range_query_bench(b, 10000);
}

mod fixtures {
    use {Region};

    #[derive(Clone, Debug, PartialEq)]
    pub struct QuadTreeRegion {
        pub x: f64,
        pub y: f64,
        pub width: f64,
        pub height: f64
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Vec2 {
        pub x: f64, pub y:f64
    }

    impl QuadTreeRegion {
        pub fn square(x: f64, y:f64, wh: f64) -> QuadTreeRegion {
            QuadTreeRegion { x: x, y: y, width: wh, height: wh }
        }
    }

    impl Region<Vec2> for QuadTreeRegion {
        fn contains(&self, p: &Vec2) -> bool {
            self.x <= p.x && self.y <= p.y && (self.x + self.width) >= p.x && (self.y + self.height) >= p.y
        }

        fn split(&self) -> Vec<QuadTreeRegion> {
            let halfwidth = self.width / 2.0;
            let halfheight = self.height / 2.0;
            vec![
                QuadTreeRegion {
                    x: self.x,
                    y: self.y,
                    width: halfwidth,
                    height: halfheight
                },

                QuadTreeRegion {
                    x: self.x,
                    y: self.y + halfheight,
                    width: halfwidth,
                    height: halfheight
                },

                QuadTreeRegion {
                    x: self.x + halfwidth,
                    y: self.y,
                    width: halfwidth,
                    height: halfheight
                },

                QuadTreeRegion {
                    x: self.x + halfwidth,
                    y: self.y + halfheight,
                    width: halfwidth,
                    height: halfheight
                }
            ]
        }

        fn overlaps(&self, other: &QuadTreeRegion) -> bool {
            other.contains(&Vec2 { x: self.x, y: self.y })
                || other.contains(&Vec2 { x: self.x + self.width, y: self.y })
                || other.contains(&Vec2 { x: self.x, y: self.y + self.height })
                || other.contains(&Vec2 { x: self.x + self.width, y: self.y + self.height })
        }
    }

    #[test]
    fn test_contains() {
        assert!(QuadTreeRegion::square(0.0, 0.0, 100.0).contains(&Vec2 { x: 50.0, y: 50.0 }));
    }

    #[test]
    fn test_overlaps() {
        assert!(QuadTreeRegion::square(0.0, 0.0, 100.0).overlaps(&QuadTreeRegion::square(50.0, 50.0, 100.0)));
    }

    #[test]
    fn test_split() {
        let fifty = 100.0 / 2.0;
        assert_eq!(QuadTreeRegion::square(0.0, 0.0, 100.0).split(),
            vec![
                QuadTreeRegion {
                    x: 0.0,
                    y: 0.0,
                    width: fifty,
                    height: fifty
                },

                QuadTreeRegion {
                    x: 0.0,
                    y: 0.0 + fifty,
                    width: fifty,
                    height: fifty
                },

                QuadTreeRegion {
                    x: 0.0 + fifty,
                    y: 0.0,
                    width: fifty,
                    height: fifty
                },

                QuadTreeRegion {
                    x: 0.0 + fifty,
                    y: 0.0 + fifty,
                    width: fifty,
                    height: fifty
                }
            ]
        )
    }
}
