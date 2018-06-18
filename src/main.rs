extern crate tcod;
extern crate rand;

use rand::Rng;

use tcod::console::*;
use tcod::colors::{self, Color};

const CONSOLE_W: i32 = 80;
const CONSOLE_H: i32 = 50;
const FPS_LIMIT: i32 = 60;

const MAP_W: i32 = CONSOLE_W - 10;
const MAP_H: i32 = CONSOLE_H - 5;

const COLOR_DARK_GROUND: Color = Color {r:75, g:75, b:75};
const COLOR_DARK_WALL: Color = Color {r:25, g:25, b:25};

const MIN_ROOM_SIZE: i32 = 3;
const MAX_ROOM_SIZE: i32 = 15;
const MAX_ROOM_NUMBER: i32 = 30;
const MAX_ROOM_DISTANCE: i32 = 999;

#[derive(PartialEq, Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect{x1: x, y1: y, x2: x + w, y2: y + h}
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersect_with(&self, other: &Rect) -> bool {
        println!("PWET");
        (self.x1 <= other.x2) && (self.x2 >= other.x1) &&
        (self.y1 <= other.y2) && (self.y2 >= other.y1)
    }

    pub fn distance_to(&self, other: &Rect) -> i32 {
        let s_center = self.center();
        let o_center = other.center();
        (((o_center.0 - s_center.0).pow(2) + (o_center.1 - s_center.1).pow(2)) as f64).sqrt().round() as i32
    }

    pub fn find_closest(&self, vector: &Vec<Rect>) -> usize {
        let mut i = 0;
        let mut max_dst = std::cmp::max(MAP_W, MAP_H) + 1;

        for (j, value) in vector.iter().enumerate() {
            if self != value {
                if self.distance_to(value) < max_dst {
                    max_dst = self.distance_to(value);
                    i = j;
                }
            }
        }

        i
    }
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    ch: char,
    blocks: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile{ch: '.', blocks: false, block_sight: false}
    }

    pub fn wall() -> Self {
        Tile{ch: '#', blocks: true, block_sight: true}
    }
}

type Map = Vec<Vec<Tile>>;

fn create_room(room: Rect, map: &mut Map) {
    // Currently, a room does not include walls
    for x in room.x1 .. room.x2 {
        for y in room.y1 .. room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn make_h_tunnel(y: i32, x1: i32, x2: i32, map: &mut Map) {
    for x in std::cmp::min(x1, x2) .. std::cmp::max(x1, x2) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn make_v_tunnel(x: i32, y1: i32, y2: i32, map: &mut Map) {
    for y in std::cmp::min(y1, y2) .. std::cmp::max(y1, y2) {
        map[x as usize][y as usize] = Tile::empty();
    }
}


fn make_map() -> (Map, (i32, i32)) {
    let mut map = vec![vec![Tile::wall(); MAP_H as usize]; MAP_W as usize];
    let mut rooms = vec![];
    let mut starting_pos = (0, 0);

    for _ in 0 .. MAX_ROOM_NUMBER {
        let w = rand::thread_rng().gen_range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
        let h = rand::thread_rng().gen_range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);

        let x = rand::thread_rng().gen_range(0, MAP_W - w);
        let y = rand::thread_rng().gen_range(0, MAP_H - h);

        let new_room = Rect::new(x, y, w, h);

        let inter_failed = rooms.iter().any(|other_room| new_room.intersect_with(other_room));
        let dst_success = rooms.iter().any(|other_room| new_room.distance_to(other_room) < MAX_ROOM_DISTANCE);

        if !inter_failed && dst_success {
            create_room(new_room, &mut map);
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                starting_pos = (new_x, new_y);
            } else {
                let idx = new_room.find_closest(&rooms);
                let (closer_x, closer_y) = rooms[idx].center();

                if rand::random() {
                    make_h_tunnel(closer_y, closer_x, new_x, &mut map);
                    make_v_tunnel(new_x, closer_y, new_y, &mut map);
                } else {
                    make_v_tunnel(closer_x, closer_y, new_y, &mut map);
                    make_h_tunnel(new_y, closer_x, new_x, &mut map);
                }

            }

            rooms.push(new_room);
        }
    }

    (map, starting_pos)
}

#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    ch: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, ch: char, color: Color) -> Self {
        Object {
            x: x,
            y: y,
            ch: ch,
            color: color,
        }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, map: &Map) {
        if (self.x + dx) >= 0 && (self.x + dx) <= MAP_W - 1 && (self.y + dy) >= 0 && (self.y + dy) <= MAP_H - 1 && !map[(self.x + dx) as usize][(self.y + dy) as usize].blocks{
            self.x += dx;
            self.y += dy;
        }

    }

    pub fn draw(&self, con: &mut Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.ch, BackgroundFlag::None);
    }

    pub fn clear(&self, con: &mut Console) {
        con.put_char(self.x, self.y, ' ', BackgroundFlag::None);
    }
}

fn handle_keys(root: &mut Root, player: &mut Object, map: &Map) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = root.wait_for_keypress(true);
    match key {

        // Movement
        Key {code: Up, ..} => player.move_by(0, -1, map),
        Key {code: Down, ..} => player.move_by(0, 1, map),
        Key {code: Left, ..} => player.move_by(-1, 0, map),
        Key {code: Right, ..} => player.move_by(1, 0, map),

        // Fullscreen
        Key {code: Enter, alt: true, ..} => {
            let fscreen = root.is_fullscreen();
            root.set_fullscreen(!fscreen);
        },

        // Quit
        Key {code: Escape, ..} => return true,

        _ => {}
    }

    false
}

fn render_all(root: &mut Root, con: &mut Offscreen, objs: &[Object], map: &Map) {

    for y in 0 .. MAP_H {
        for x in 0 .. MAP_W {
            let wall = map[x as usize][y as usize].block_sight;
            if wall {
                con.set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                con.set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
        }
    }

    for obj in objs {
        obj.draw(con);
    }

    blit(con, (0, 0), (MAP_W, MAP_H), root, (0, 0), 1.0, 1.0);
}

fn main() {
    println!("Hello, world!");

    let mut root = Root::initializer()
    .font("arial10x10.png", FontLayout::Tcod)
    .font_type(FontType::Greyscale)
    .size(CONSOLE_W, CONSOLE_H)
    .title("R U S T L I K E")
    .init();

    let mut con = Offscreen::new(MAP_W, MAP_H);

    tcod::system::set_fps(FPS_LIMIT);

    let (map, (p_x, p_y)) = make_map();

    let player = Object::new(p_x, p_y, '@', colors::WHITE);
    let npc = Object::new(p_x+1, p_y+1, 'z', colors::YELLOW);
    let mut objects = [player, npc];

    while !root.window_closed() {
        root.clear();

        render_all(&mut root, &mut con, &objects, &map);

        root.flush();

        for object in &objects {
            object.clear(&mut con);
        }

        let player = &mut objects[0];

        let exit = handle_keys(&mut root, player, &map);
        if exit {
            break
        }


    }
}
