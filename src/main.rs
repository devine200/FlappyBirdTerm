use bracket_lib::prelude::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;

enum GameMode {
    Menu,
    Playing,
    End,
}

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(0, self.y, YELLOW, BLACK, to_cp437('@'));
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }
        self.y += self.velocity as i32;
        self.x += 1;

        if self.y < 0 {
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
    width: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score),
            width: 5
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;
        let gap_y_start = self.gap_y - half_size;
        let gap_y_end = self.gap_y + half_size;

        for y in 0..gap_y_start {
            for i in 0..self.width {
                ctx.set(screen_x+i, y, RED, BLACK, to_cp437('|'));
            }
        }
        
        for y in gap_y_end..SCREEN_HEIGHT {
            for i in 0..self.width {
                ctx.set(screen_x+i, y, RED, BLACK, to_cp437('|'));
            }
        }
    }

    fn hit_obstacle(&mut self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x >= self.x && player.x <= self.x + self.width;
        let gap_y_start = self.gap_y - half_size;
        let gap_y_end = self.gap_y + half_size;
        let player_above_gap = player.y < gap_y_start;
        let player_below_gap = player.y > gap_y_end;
        does_x_match && (player_above_gap || player_below_gap)
    }
}

struct State {
    player: Player,
    mode: GameMode,
    frame_time: f32,
    obstacle: Obstacle,
    score: i32,
}

impl State {
    fn new() -> Self {
        State {
            mode: GameMode::Menu,
            frame_time: 0.0,
            player: Player::new(5, 25),
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            score: 0,
        }
    }

    fn restart(&mut self) {
        self.frame_time = 0.0;
        self.player = Player::new(5, 25);
        self.mode = GameMode::Playing;
        self.score = 0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0)
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key_pressed) = ctx.key {
            match key_pressed {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => (),
            }
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        if self.player.x > self.obstacle.x + self.obstacle.width {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }
        self.obstacle.render(ctx, self.player.x);
        self.player.render(ctx);
        ctx.print_centered(0, "PRESS SPACE BAR TO FLAP");
        ctx.print_centered(1, &format!("Score: {}", self.score));

        if self.player.y > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;

    main_loop(context, State::new())
}
