#![feature(rust_2018_preview)]

use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::io::Read;
use std::io::{stdout, BufWriter, Write};

const STAGEWIDTH: usize = 10;
const STAGEHEIGHT: usize = 8;

#[derive(Copy, Clone, Debug)]
enum Object {
    ObjSpace,
    ObjWall,
    ObjGoal,
    ObjBlock,
    ObjBlockOnGoal,
    ObjMan,
    ObjManOnGoal,

    ObjUnknown,
}

struct Stage {
    origin: String,
    objects: [Object; STAGEWIDTH * STAGEHEIGHT],
}

impl Stage {
    fn initialize(filepath: &str) -> Stage {
        Stage {
            origin: read_stage_file(filepath),
            objects: [Object::ObjUnknown; STAGEWIDTH * STAGEHEIGHT],
        }
    }

    fn load(&mut self) {
        let mut object_map = HashMap::new();
        object_map.insert(' ', Object::ObjSpace);
        object_map.insert('#', Object::ObjWall);
        object_map.insert('.', Object::ObjGoal);
        object_map.insert('o', Object::ObjBlock);
        object_map.insert('O', Object::ObjBlockOnGoal);
        object_map.insert('p', Object::ObjMan);
        object_map.insert('P', Object::ObjManOnGoal);

        for (y, line) in self.origin.lines().enumerate() {
            for (x, data) in line.chars().enumerate() {
                self.objects[y * STAGEWIDTH + x] = object_map[&data];
            }
        }
    }

    fn draw(&mut self) {
        // draw stage using buffer for large data set.
        let out = stdout();
        let mut out = BufWriter::new(out.lock());

        // clear the entire screen.
        write!(out, "{}[2J", 27 as char).unwrap();

        // order of the elements in the dict is same as enum Object.
        let font = [" ", "#", ".", "o", "O", "p", "P"];

        for y in 0..STAGEHEIGHT {
            for x in 0..STAGEWIDTH {
                write!(out, "{}", font[self.objects[y * STAGEWIDTH + x] as usize]).unwrap();
            }
            writeln!(out, "").unwrap();
        }
    }

    fn action(&mut self, x: i32, dx: i32, y: i32, dy: i32) {
        // check whether 1 space forward from current position is under valid range.
        let tx = x + dx;
        let ty = y + dy;
        if tx < 0 || ty < 0 || tx >= (STAGEWIDTH as i32) || ty >= (STAGEHEIGHT as i32) {
            return;
        }

        // position of person.
        let p = (y * (STAGEWIDTH as i32) + x) as usize;
        // target position to move forward.
        let tp = (ty * (STAGEWIDTH as i32) + tx) as usize;

        match self.objects[tp] {
            Object::ObjSpace | Object::ObjGoal => {
                Stage::update_goal_for_man(self, tp);
                Stage::update_man_on_goal(self, p);
            }
            Object::ObjBlock | Object::ObjBlockOnGoal => {
                // check whether 2 spaces forward from current position is under the valid range.
                let tx2 = (tp as i32) + dx;
                let ty2 = (tp as i32) + dy;
                if tx2 < 0
                    || ty2 < 0
                    || tx2 >= ((STAGEWIDTH * STAGEHEIGHT) as i32)
                    || ty2 >= ((STAGEHEIGHT * STAGEWIDTH) as i32)
                {
                    return;
                }

                // 2 spaces forward from current position.
                let tp2 = ((ty + dy) * (STAGEWIDTH as i32) + (tx + dx)) as usize;

                // check the object on current position, target position and 1 space forward from target position.
                match self.objects[tp2] {
                    Object::ObjSpace | Object::ObjGoal => {
                        Stage::update_goal_for_block(self, tp2);
                        Stage::update_block_on_goal(self, tp);
                        Stage::update_man_on_goal(self, p);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn update_goal_for_man(&mut self, idx: usize) {
        if let Object::ObjGoal = self.objects[idx] {
            self.objects[idx] = Object::ObjManOnGoal;
            return;
        }
        self.objects[idx] = Object::ObjMan;
    }

    fn update_goal_for_block(&mut self, idx: usize) {
        if let Object::ObjGoal = self.objects[idx] {
            self.objects[idx] = Object::ObjBlockOnGoal;
            return;
        }
        self.objects[idx] = Object::ObjBlock;
    }

    fn update_block_on_goal(&mut self, idx: usize) {
        if let Object::ObjBlockOnGoal = self.objects[idx] {
            self.objects[idx] = Object::ObjManOnGoal;
            return;
        }
        self.objects[idx] = Object::ObjMan;
    }

    fn update_man_on_goal(&mut self, idx: usize) {
        if let Object::ObjManOnGoal = self.objects[idx] {
            self.objects[idx] = Object::ObjGoal;
            return;
        }
        self.objects[idx] = Object::ObjSpace;
    }

    fn update(&mut self, input: char) {
        let mut dx = 0;
        let mut dy = 0;
        match input {
            'a' => dx = -1,
            's' => dx = 1,
            'w' => dy = -1,
            'z' => dy = 1,
            'r' => {
                Stage::reset(self);
                return;
            }
            _ => println!("Input error: invalid input."),
        }
        let mut idx: usize = 0;
        for (i, object) in self.objects.iter().enumerate() {
            if let Object::ObjMan = *object {
                idx = i;
                break;
            }
            if let Object::ObjManOnGoal = *object {
                idx = i;
                break;
            }
        }

        let x = (idx % STAGEWIDTH) as i32;
        let y = (idx / STAGEWIDTH) as i32;

        Stage::action(self, x, dx, y, dy);
    }

    fn check_clear(&self) -> bool {
        for object in self.objects.iter() {
            if let Object::ObjBlock = *object {
                return false;
            }
        }
        return true;
    }

    fn reset(&mut self) {
        Stage::load(self);
        Stage::draw(self);
    }
}

fn read_stage_file(filepath: &str) -> String {
    let mut f = File::open(filepath).expect("file not found");
    let mut stage_data: String = String::new();
    f.read_to_string(&mut stage_data)
        .expect("failed to read file contents");
    return stage_data;
}

fn main() {
    let stage_files = read_dir("./src/stage").expect("directory not found");
    for file in stage_files {
        let mut state: Stage =
            Stage::initialize(file.expect("file not found").path().to_str().unwrap());

        Stage::load(&mut state);

        loop {
            Stage::draw(&mut state);

            if Stage::check_clear(&state) {
                println!("STAGE CLEAR!");
                break;
            }

            println!("a: left s: right w: up z: down r: reset. Input command?");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            Stage::update(&mut state, input.chars().nth(0).unwrap());
        }
    }
}
