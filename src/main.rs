use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;


fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .add_systems(Startup, create_points)
    .add_systems(Update, quadtree_system)
    .add_systems(Update, draw_quadtree.after(quadtree_system))
    .add_systems(Update, mouse_movement.after(quadtree_system))
    .add_systems(Update, quadtree_search.after(mouse_movement))
    .add_systems(Update, click_add_points)
    .insert_resource(InitialBoundary{ rect: Rect::from_center_size(Vec2::from([0.0, 0.0]), Vec2::from([600.0, 600.0]))})
    .insert_resource(Points{ points: Vec::default() })
    .insert_resource(QuadTreeResource{ quadtree: None})
    .insert_resource(SearchBounds{ bounds: None})
    .run();
}

pub fn setup(mut commands: Commands){
    commands.spawn(Camera2dBundle::default());
}

pub struct Point {
    pub position: Vec2
}

#[derive(Resource)]
pub struct InitialBoundary {
    rect: Rect,
}

#[derive(Resource)]
pub struct Points {
    pub points: Vec<Point>
}

#[derive(Resource)]
pub struct QuadTreeResource {
    pub quadtree: Option<QuadTree>
}

#[derive(Resource)]
pub struct SearchBounds {
    pub bounds: Option<Rect>
}

pub struct QuadTree {
    pub boundary: Rect,
    capacity: usize,
    divided: bool,
    nw: Option<Box<QuadTree>>,
    ne: Option<Box<QuadTree>>,
    sw: Option<Box<QuadTree>>,
    se: Option<Box<QuadTree>>,
    points: Vec<Vec2>,
    depth: i32
}

pub const MAX_DEPTH: i32 = 5;

impl QuadTree {
    fn new(boundary: Rect, depth: i32) -> Self {
        QuadTree { divided: false, capacity: 4, boundary, nw: None, ne: None, sw: None, se: None, points: Vec::default(), depth }
    }

    pub fn search(&self, boundary: Rect, mut items: Option<Vec<Vec2>>) -> Option<Vec<Vec2>> {
        if let None = items{
            items = Some(Vec::default());
        }
        if self.boundary.intersect(boundary).is_empty() { return items; }

        if self.divided {
            if let Some(test) = &self.nw{
                items = test.search(boundary, items);
            }
            if let Some(test) = &self.ne{
                items = test.search(boundary, items);
            }
            if let Some(test) = &self.se{
                items = test.search(boundary, items);
            }
            if let Some(test) = &self.sw{
                items = test.search(boundary, items);
            }

            return items;
        }

        for p in &self.points {
            if self.boundary.contains(*p){
                 items.as_mut().unwrap().push(*p);
            }
        }

        return items;
    }

    // move drawing into own system
    // add remove function

    fn get_all_children(&self, mut items: Option<Vec<QuadTree>>) -> Option<Vec<QuadTree>> {
        if let None = items{
            items = Some(Vec::default());
        }

        if self.divided {
            if let Some(test) = &self.nw{
                items = test.get_all_children(items);
            }
            if let Some(test) = &self.ne{
                items = test.get_all_children(items);
            }
            if let Some(test) = &self.se{
                items = test.get_all_children(items);
            }
            if let Some(test) = &self.sw{
                items = test.get_all_children(items);
            }

            return items;
        }
        items
    }

    fn draw(&self, gizmo: &mut Gizmos){
        gizmo.rect_2d(self.boundary.center(), 0.0, self.boundary.size(), Color::GREEN);

        for point in self.points.iter() {
            gizmo.circle_2d(*point, 1.0, Color::RED);
        }

        match &self.nw {
            Some(qt) => qt.draw(gizmo),
            _ => { }
        };

        match &self.ne {
            Some(qt) => qt.draw(gizmo),
            _ => { }
        };

        match &self.sw {
            Some(qt) => qt.draw(gizmo),               
            _ => { }
        };

        match &self.se {
            Some(qt) => qt.draw(gizmo),
            _ => { }
        };
    }

    fn insert(&mut self, point: &Vec2) -> bool {
        if self.boundary.contains(*point) == false {
            return false;
        } 

        if self.depth >= MAX_DEPTH {
            self.points.push(*point);
            return true;
        }

        if self.points.len() < self.capacity && self.divided == false {
            self.points.push(*point);
            return true;
        }

        if let None = self.nw  {
            self.subdivide();
        }

        if self.nw.as_mut().unwrap().insert(point) {
            return true;
        } else if self.ne.as_mut().unwrap().insert(point) {
            return true;
        } else if self.sw.as_mut().unwrap().insert(point) {
            return true;
        } else if self.se.as_mut().unwrap().insert(point) {
            return true;
        }

        return false;
    }

    fn subdivide(&mut self){
        let nw_origin = Vec2::from([self.boundary.center().x  - self.boundary.half_size().x / 2.0, self.boundary.center().y + self.boundary.half_size().y / 2.0]);
        let mut nw_quad = QuadTree::new( Rect::from_center_size(nw_origin, self.boundary.size() / 2.0), self.depth + 1);
 
        let ne_origin = Vec2::from([self.boundary.center().x  + self.boundary.half_size().x / 2.0, self.boundary.center().y + self.boundary.half_size().y / 2.0]);
        let mut ne_quad = QuadTree::new( Rect::from_center_size(ne_origin, self.boundary.size() / 2.0), self.depth + 1);

        let sw_origin = Vec2::from([self.boundary.center().x  - self.boundary.half_size().x / 2.0, self.boundary.center().y - self.boundary.half_size().y / 2.0]);
        let mut sw_quad = QuadTree::new( Rect::from_center_size(sw_origin, self.boundary.size() / 2.0), self.depth + 1);

        let se_origin = Vec2::from([self.boundary.center().x  + self.boundary.half_size().x / 2.0, self.boundary.center().y - self.boundary.half_size().y / 2.0]);
        let mut se_quad = QuadTree::new( Rect::from_center_size(se_origin, self.boundary.size() / 2.0), self.depth + 1);

        self.divided = true;

        for p in self.points.iter() {
            if nw_quad.boundary.contains(*p){
                nw_quad.insert(p);
            } else if ne_quad.boundary.contains(*p){
                ne_quad.insert(p);
            }
             else if se_quad.boundary.contains(*p){
                se_quad.insert(p);
            
            } else if sw_quad.boundary.contains(*p){
                sw_quad.insert(p);
            
            }
        }

        self.points.clear();

        self.nw = Some(Box::new(nw_quad));
        self.ne = Some(Box::new(ne_quad));
        self.sw = Some(Box::new(sw_quad));
        self.se = Some(Box::new(se_quad));
    }
}

pub fn click_add_points(mouse: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut points: ResMut<Points>)
{
    if mouse.just_released(MouseButton::Left){
        let (camera, camera_transform) = camera_q.single();
        if let Some(position) = q_windows.single().cursor_position(){

            let pos =  Vec3::from((camera.viewport_to_world_2d(camera_transform, position).unwrap(), 1.0));
            points.points.push(Point{position:Vec2::from([pos.x, pos.y])});
        }
    }
}

fn create_points(mut points: ResMut<Points>, initial_boundary: Res<InitialBoundary>){
    let boundary =  initial_boundary.rect;
    for _ in 0..20 {
        let mut rand = rand::thread_rng();
        let x = rand.gen_range(boundary.min.x..=boundary.max.x);
        let y = rand.gen_range(boundary.min.y..=boundary.max.y);
        points.points.push(Point{ position: Vec2::from([x, -y]) });
    }
}

pub fn mouse_movement(mut gizmos: Gizmos, q_windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut search_bounds: ResMut<SearchBounds>)
    {
        let (camera, camera_transform) = camera_q.single();
        if let Some(position) = q_windows.single().cursor_position(){

            let pos =  Vec2::from((camera.viewport_to_world_2d(camera_transform, position).unwrap()));
            let rect: Rect = Rect::from_center_size(pos, Vec2::from([100.0, 100.0]));
            search_bounds.bounds = Some(rect);
            gizmos.rect_2d(rect.center(), 0.0, rect.size(), Color::RED);
        }
    }

pub fn quadtree_search(mut gizmos: Gizmos, quadtree: ResMut<QuadTreeResource>, search_bounds: Res<SearchBounds>){
    if let None = search_bounds.bounds {
        return;
    }

    let qt = match &quadtree.quadtree {
        Some(qt) => qt,
        None => { return; }
    };

    let search =  &qt.search(search_bounds.bounds.unwrap(), None);

    let points = match search {
        Some(points) => points,
        None => { return; }
    };

    for point in points.iter() {
        if search_bounds.bounds.unwrap().contains(*point){
            gizmos.rect_2d(*point, 0.0, Vec2::from([6.0, 6.0]), Color::WHITE);
        }
    }
}

pub fn quadtree_system(points: ResMut<Points>, initial_boundary: Res<InitialBoundary>, mut quadtree: ResMut<QuadTreeResource>){
    let boundary = initial_boundary.rect;
    let mut qt = QuadTree::new(boundary, 0);

    for point in points.points.iter() {
        qt.insert(&point.position);
    }

    quadtree.quadtree = Some(qt);
}

pub fn draw_quadtree(quadtree: ResMut<QuadTreeResource>, mut gizmo: Gizmos,){
    if let Some(qt) = &quadtree.quadtree {
       qt.draw(&mut gizmo);

        // let children = qt.get_all_children(None);


        // gizmo.rect_2d(qt.boundary.center(), 0.0 , size, color)
        // for child in children.unwrap() {
        //     gizmo.rect_2d(child.boundary.center(), 0.0, child.boundary.size(), Color::GREEN)
        // }

        // let quads = qt.get_all();
        // for quad in quads
        //{
        // gizmo.rect_2d(qt.boundary.center(), 0.0, qt.boundary.size(), Color::GREEN);  
        //}
    }
}