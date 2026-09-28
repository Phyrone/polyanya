use std::collections::HashSet;

use glam::{vec2, Vec2};
use polyanya::{Coords, Layer, Mesh, Polygon, Triangulation, Vertex};

/// A ground with a stair standing on it against a wall: the footprint of both
/// is cut out of the ground, and the stair's foot is stitched to the ground
/// along the edge they share. The stair's top leads nowhere.
fn ground_and_stair() -> Mesh {
    let foot = [vec2(4.0, 5.0), vec2(6.0, 5.0)];
    let top = [vec2(6.0, 7.0), vec2(4.0, 7.0)];

    let mut ground = Triangulation::from_outer_edges(&[
        vec2(0.0, 0.0),
        vec2(10.0, 0.0),
        vec2(10.0, 10.0),
        vec2(0.0, 10.0),
    ]);
    // The stair and the wall it climbs, behind it.
    ground.add_obstacle(vec![foot[0], foot[1], vec2(6.0, 8.0), vec2(4.0, 8.0)]);
    let mut ground = ground.as_layer();
    ground.bake();

    // One quad, as a stair's foot is: every corner has the one polygon and
    // the outside around it.
    let mut stair = Layer::new(
        [foot[0], foot[1], top[0], top[1]]
            .into_iter()
            .map(|corner| Vertex::new(corner, vec![0, u32::MAX]))
            .collect(),
        vec![Polygon::new(vec![0, 1, 2, 3], false)],
    )
    .unwrap();
    stair.bake();

    let at = |layer: &polyanya::Layer, point: Vec2| {
        layer
            .vertices
            .iter()
            .position(|vertex| vertex.coords == point)
            .unwrap()
    };
    let pairs: Vec<(usize, usize)> = foot
        .iter()
        .map(|point| (at(&ground, *point), at(&stair, *point)))
        .collect();
    let back = pairs.iter().map(|(g, s)| (*s, *g)).collect();

    let mut mesh = Mesh::default();
    mesh.layers = vec![ground, stair];
    // Both ways as two one-way joins, in one call: a second call would
    // offset the polygon indices a second time.
    mesh.stitch_at_vertices(vec![((0, 1), pairs), ((1, 0), back)], true);
    mesh
}

/// A point is found on a polygon up to `search_delta` away from it, so a
/// point just off the stair's foot, on the ground, is found on the stair too.
/// A search from there must start on the stair, not from where the point
/// lies — outside the stair, where no edge of it leads anywhere.
#[test]
fn a_search_from_just_off_a_stitched_layer_finds_its_way() {
    let mesh = ground_and_stair();
    let start = vec2(5.0, 4.95);
    let goal = vec2(9.0, 9.0);

    let on_stair = mesh
        .get_point_layer(start)
        .into_iter()
        .find(|coords| coords.layer() == Some(1))
        .expect("the point is within `search_delta` of the stair");
    for from in [on_stair, Coords::on_layer(start, 1)] {
        let path = mesh
            .search_path(from, Coords::on_layer(goal, 0), HashSet::default())
            .expect("the way off the stair onto the ground");
        assert!(
            path.path.last().unwrap().distance(goal) < 1e-3,
            "it ends at {:?}",
            path.path.last()
        );
        let straight: f32 = Vec2::distance(start, goal);
        assert!(
            path.length < straight * 1.2,
            "a path of {} where the straight line is {straight}",
            path.length
        );
    }
}
