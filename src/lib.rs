#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]
#![feature(struct_variant)]

//! A generic, n-dimensional quadtree for fast neighbor lookups on multiple axes.

use std::{mem, slice};

/// The required interface for Regions in this n-tree.
///
/// Regions must be able to split themselves, tell if they overlap
/// other regions, and tell if a point is contained within the region.
pub trait Region<P>: Clone {
    /// Does this region contain this point?
    fn contains(&self, &P) -> bool;

    /// Split this region, returning a Vec of sub-regions.
    ///
    /// Invariants:
    ///   - The sub-regions must NOT overlap.
    ///   - All points in self must be contained within one and only one sub-region.
    fn split(&self) -> Vec<Self>;

    /// Does this region overlap with this other region?
    fn overlaps(&self, other: &Self) -> bool;
}

/// A quadtree-like structure, but for arbitrary arity.
///
/// Regions can split themselves into arbitrary numbers of splits,
/// allowing this structure to be used to index data by any number
/// of attributes and quickly query for data that falls within a
/// specific range.
pub enum NTree<R, P> {
    /// A leaf of the tree, which contains points.
    Bucket {
        region: R,
        points: Vec<P>,
        bucket_limit: u8
    },
    /// An interior node of the tree, which contains n subtrees.
    Branch {
        region: R,
        subregions: Vec<NTree<R, P>>
    }
}

impl<P, R: Region<P>> NTree<R, P> {
    /// Create a new n-tree which contains points within
    /// the region and whose buckets are limited to the passed-in size.
    ///
    /// The number of regions returned by region.split() dictates
    /// the arity of the tree.
    pub fn new(region: R, size: u8) -> NTree<R, P> {
        Branch {
            subregions: region
                            .split()
                            .move_iter()
                            .map(|r| Bucket { region: r, points: vec![], bucket_limit: size })
                            .collect(),
            region: region
        }
    }

    /// Insert a point into the n-tree, returns true if the point
    /// is within the n-tree and was inserted and false if not.
    pub fn insert(&mut self, point: P) -> bool {
        match *self {
            Bucket { ref mut points, ref region, ref bucket_limit } => {
                if region.contains(&point) {
                    if points.len() as u8 != *bucket_limit {
                        points.push(point);
                        return true
                    }
                } else {
                    return false
                }
            },
            Branch { ref region, ref mut subregions, .. } => {
                if region.contains(&point) {
                    match subregions.mut_iter().find(|r| r.contains(&point)) {
                        Some(ref mut subregion) => return subregion.insert(point),
                        None => return false
                    }
                } else {
                    return false
                }
            }
        };

        // Bucket is full
        split_and_insert(self, point);
        true
    }

    /// Get all the points which within the queried region.
    ///
    /// Finds all points which are located in regions overlapping
    /// the passed in region, then filters out all points which
    /// are not strictly within the region.
    pub fn range_query<'t, 'q>(&'t self, query: &'q R) -> RangeQuery<'t, 'q, R, P> {
        RangeQuery {
            query: query,
            points: empty_iterator(),
            stack: vec![slice::ref_slice(self).iter()],
        }
    }

    /// Is the point contained in the n-tree?
    pub fn contains(&self, point: &P) -> bool {
        match *self {
            Bucket { ref region, .. } => region.contains(point),
            Branch { ref region, .. } => region.contains(point)
        }
    }

    /// Get all the points nearby a specified point.
    ///
    /// This will return no more than bucket_limit points.
    pub fn nearby<'a>(&'a self, point: &P) -> Option<&'a[P]> {
        match *self {
            Bucket { ref region, ref points, .. } => {
                if region.contains(point) {
                    Some(points.as_slice())
                } else {
                    None
                }
            },
            Branch { ref region, ref subregions } => {
                if region.contains(point) {
                    subregions
                        .iter()
                        .find(|r| r.contains(point))
                        .and_then(|r| r.nearby(point))
                } else {
                    None
                }
            }
        }
    }
}

fn split_and_insert<P, R: Region<P>>(bucket: &mut NTree<R, P>, point: P) {
    let mut old_points;
    let mut old_region;
    let mut old_bucket_limit;

    match *bucket {
        // Get the old region, points, and bucket limit.
        Bucket { ref region, ref mut points, bucket_limit } => {
            old_region = region.clone();
            old_points = mem::replace(points, vec![]);
            old_bucket_limit = bucket_limit;
        },
        Branch { .. } => unreachable!()
    }

    // Replace the bucket with a split branch.
    *bucket = NTree::new(old_region, old_bucket_limit);

    // Insert all the old points into the right place.
    for old_point in old_points.move_iter() {
        bucket.insert(old_point);
    }

    // Finally, insert the new point.
    bucket.insert(point);
}

/// An iterator over the points within a region.

// This iterates over the leaves of the tree from left-to-right by
// maintaining (a) the sequence of points at the current level
// (possibly empty), and (b) stack of iterators over the remaining
// children of the parents of the current point.
pub struct RangeQuery<'t,'q, R, P> {
    query: &'q R,
    points: slice::Items<'t, P>,
    stack: Vec<slice::Items<'t, NTree<R, P>>>
}

impl<'t, 'q, R: Region<P>, P> Iterator<&'t P> for RangeQuery<'t, 'q, R, P> {
    fn next(&mut self) -> Option<&'t P> {
        'outer: loop {
            // try to find the next point in the region we're
            // currently examining.
            for p in self.points {
                if self.query.contains(p) {
                    return Some(p)
                }
            }

            // no relevant points, so lets find a new region.

            'region_search: loop {
                let mut children_iter = match self.stack.pop() {
                    Some(x) => x,

                    // no more regions, so we're over.
                    None => return None,
                };

                'children: loop {
                    // look at the next item in the current sequence
                    // of children.
                    match children_iter.next() {
                        // this region is empty, next region!
                        None => continue 'region_search,

                        Some(&Bucket { ref region, ref points, .. }) => {
                            if region.overlaps(self.query) {
                                // found something with points
                                self.points = points.iter();

                                // this top region might have more
                                // buckets/branches, so save it to
                                // look at next time.
                                self.stack.push(children_iter);
                                // lets look at the points.
                                continue 'outer;
                            }
                        }
                        Some(&Branch { ref region, ref subregions }) => {
                            // a new subregion

                            if region.overlaps(self.query) {
                                // recur by saving the current state
                                // and iterating over the children
                                // right now.
                                self.stack.push(children_iter);
                                children_iter = subregions.iter()
                            }
                        }
                    }
                }
            }
        }
    }
}

fn empty_iterator<'a, P>() -> slice::Items<'a, P> {
    use std::raw;

    static MARKER: () = ();
    // this is safe because the length is 0, so the pointer is never
    // ever dereferenced or read from.
    //
    // FIXME: move this upstream
    unsafe {
        let array: &'a [P] = mem::transmute(raw::Slice {
            data: &MARKER,
            len: 0
        });
        array.iter()
    }
}
