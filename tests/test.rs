extern crate ntree;

use ntree::{NTree, Region};

#[deriving(Clone, Show, PartialEq)]
struct QuadTreeRegion {
    x: f64,
    y: f64,
    width: f64,
    height: f64
}

struct Vec2 {
    x: f64, y:f64
}

impl QuadTreeRegion {
    fn square(x: f64, y:f64, wh: f64) -> QuadTreeRegion {
        QuadTreeRegion { x: x, y: y, width: wh, height: wh }
    }
}

impl Region<Vec2> for QuadTreeRegion {
    fn contains(&self, p: &Vec2) -> bool {
        self.x < p.x && self.y < p.y && (self.x + self.width) > p.x && (self.y + self.height) > p.y
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

#[test] fn test_contains() {
    assert!(QuadTreeRegion::square(0.0, 0.0, 100.0).contains(&Vec2 { x: 50.0, y: 50.0 }));
}

#[test] fn test_overlaps() {
    assert!(QuadTreeRegion::square(0.0, 0.0, 100.0).overlaps(&QuadTreeRegion::square(50.0, 50.0, 100.0)));
}

#[test] fn test_split() {
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
