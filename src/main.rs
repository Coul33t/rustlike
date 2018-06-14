extern crate tcod;

use tcod::console::*;
use tcod::colors::{self, Color};

const CONSOLE_W: i32 = 80;
const CONSOLE_H: i32 = 50;
const FPS_LIMIT: i32 = 60;

const MAP_W: i32 = CONSOLE_W - 10;
const MAP_H: i32 = CONSOLE_H - 5;

const COLOR_DARK_GROUND: Color = Color {r:75, g:75, b:75};
const COLOR_DARK_WALL: Color = Color {r:25, g:25, b:25};

#[derive(Clone, Copy, Debug)]
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

fn make_map() -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_H as usize]; MAP_W as usize];

    let room1 = Rect::new(10,10,20,20);
    let room2 = Rect::new(35,35,15,5);

    create_room(room1, &mut map);
    create_room(room2, &mut map);

    map
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

    let player = Object::new(20, 20, '@', colors::WHITE);
    let npc = Object::new(25, 25, 'z', colors::YELLOW);
    let mut objects = [player, npc];

    let map = make_map();

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
