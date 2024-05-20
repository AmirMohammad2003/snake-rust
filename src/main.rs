use std::{
    cmp,
    collections::VecDeque,
    io::{self, stdout, Stdout, Write},
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    style::{Print, PrintStyledContent, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use rand::Rng;

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    let (cols, rows) = size().unwrap_or((30, 30));
    let mut land = Land::new((cols - 2) / 2, rows);
    let mut world = vec![vec![-1i8; land.cols as usize]; land.rows as usize];
    // let mut prev_world = world.clone();
    let mut snake = Snake::new(land.rows, land.cols);
    stdout.queue(Hide)?.queue(Clear(ClearType::All))?;
    enable_raw_mode()?;

    let mut playing = true;
    while playing {
        let ret = handle_key_input(&mut snake)?;
        if ret == "q" || ret == "c" {
            playing = false;
        } else {
            if snake.body[0] == land.food.pos {
                snake.eat();
                land.refood(&snake.body);
            }
            make_world(&mut world, &land, &snake)?;
            draw(&mut stdout, &world)?;
            stdout.flush()?;
        }
    }

    disable_raw_mode()?;
    stdout.execute(Show)?;
    Ok(())
}

fn handle_key_input(snake: &mut Snake) -> io::Result<String> {
    let mut ret = String::from("");
    if poll(Duration::from_millis(100))? {
        let key = read()?;
        while poll(Duration::from_millis(0)).unwrap() {
            let _ = read();
        }
        if let Event::Key(event) = key {
            match event.code {
                KeyCode::Char('w') | KeyCode::Up => {
                    if snake.move_up() {
                        ret = String::from("c")
                    }
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    if snake.move_down() {
                        ret = String::from("c")
                    }
                }
                KeyCode::Char('a') | KeyCode::Left => {
                    if snake.move_left() {
                        ret = String::from("c")
                    }
                }
                KeyCode::Char('d') | KeyCode::Right => {
                    if snake.move_right() {
                        ret = String::from("c")
                    }
                }
                KeyCode::Char('q') => ret = String::from("q"),
                KeyCode::Char(n) => ret = n.to_string(),
                _ => return Ok(ret),
            }
        }
    }
    Ok(ret)
}

fn make_world(world: &mut Vec<Vec<i8>>, land: &Land, snake: &Snake) -> io::Result<()> {
    for row in &mut *world {
        for v in row {
            *v = 0;
        }
    }
    for p in &snake.body {
        let (r, c) = (p.1 as usize, p.0 as usize);
        world[r][c] = 1i8;
    }

    world[land.food.pos.1 as usize][land.food.pos.0 as usize] = 2i8;
    Ok(())
}

fn draw(stdout: &mut Stdout, world: &Vec<Vec<i8>>) -> io::Result<()> {
    const SQUARE: &str = "██";
    stdout.queue(MoveTo(0, 2))?;
    for row in world {
        for val in row {
            match val {
                1 => stdout.queue(PrintStyledContent(SQUARE.blue()))?,
                2 => stdout.queue(PrintStyledContent(SQUARE.red()))?,
                _ => stdout.queue(PrintStyledContent(SQUARE.green()))?,
            };
        }
        stdout.queue(Print("\r\n"))?;
    }
    Ok(())
}

#[derive(Debug)]
struct Point(i16, i16);

impl PartialEq<Point> for Point {
    fn eq(&self, other: &Point) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

struct Snake {
    body: VecDeque<Point>,
    rows: u16,
    cols: u16,
    poped: Option<Point>,
}

impl Snake {
    fn new(rows: u16, cols: u16) -> Snake {
        Snake {
            body: VecDeque::from([Point(10, 10)]),
            rows,
            cols,
            poped: None,
        }
    }

    fn _move(&mut self, mut new_pos: Point) -> bool {
        new_pos.0 = new_pos.0.rem_euclid(self.cols as i16);
        new_pos.1 = new_pos.1.rem_euclid(self.rows as i16);
        if self.body.contains(&new_pos) {
            return true;
        }
        self.body.push_front(new_pos);
        self.poped = self.body.pop_back();
        false
    }

    fn eat(&mut self) {
        self.body.push_back(self.poped.take().unwrap());
    }

    fn move_right(&mut self) -> bool {
        self._move(Point(self.body[0].0 + 1, self.body[0].1))
    }

    fn move_left(&mut self) -> bool {
        self._move(Point(self.body[0].0 - 1, self.body[0].1))
    }

    fn move_down(&mut self) -> bool {
        self._move(Point(self.body[0].0, self.body[0].1 + 1))
    }

    fn move_up(&mut self) -> bool {
        self._move(Point(self.body[0].0, self.body[0].1 - 1))
    }
}

struct Food {
    pos: Point,
}
struct Land {
    cols: u16,
    rows: u16,

    food: Food,
}

impl Land {
    fn new(cols: u16, rows: u16) -> Land {
        let min = cmp::min(cols, rows);
        Land {
            cols: min,
            rows: min,
            food: Food { pos: Point(5, 5) },
        }
    }

    fn refood(&mut self, snake_body: &VecDeque<Point>) {
        self.food.pos = random_point(self.cols, self.rows, &self.food.pos, &snake_body);
    }
}

fn random_point(cols: u16, rows: u16, food_pos: &Point, snake: &VecDeque<Point>) -> Point {
    let mut rng = rand::thread_rng();
    let mut new_pos = Point(rng.gen_range(0..cols as i16), rng.gen_range(0..rows as i16));
    while snake.contains(&new_pos) || new_pos == *food_pos {
        new_pos = Point(rng.gen_range(0..cols as i16), rng.gen_range(0..rows as i16));
    }
    new_pos
}
