use std::f64::consts::PI;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

const SCREEN_WIDTH: usize = 80;
const SCREEN_HEIGHT: usize = 35;
const THRESHOLD: f64 = 1.0;
const ASPECT_RATIO: f64 = 2.0;

#[derive(Clone, Copy)]
enum RenderMode {
    Gradient,      // Original gradient fill
    Contour,       // Outline only - shows merging clearly
    Solid,         // Binary solid fill
    Blocks,        // Unicode block characters
    Gooey,         // Emphasizes merge points
}

impl RenderMode {
    fn next(self) -> Self {
        match self {
            RenderMode::Gradient => RenderMode::Contour,
            RenderMode::Contour => RenderMode::Solid,
            RenderMode::Solid => RenderMode::Blocks,
            RenderMode::Blocks => RenderMode::Gooey,
            RenderMode::Gooey => RenderMode::Gradient,
        }
    }

    fn name(self) -> &'static str {
        match self {
            RenderMode::Gradient => "Gradient",
            RenderMode::Contour => "Contour",
            RenderMode::Solid => "Solid",
            RenderMode::Blocks => "Blocks",
            RenderMode::Gooey => "Gooey",
        }
    }
}

struct Blob {
    x: f64,
    y: f64,
    radius: f64,
}

impl Blob {
    fn new(x: f64, y: f64, radius: f64) -> Self {
        Self { x, y, radius }
    }

    fn field_at(&self, px: f64, py: f64) -> f64 {
        let dx = (px - self.x) / ASPECT_RATIO;
        let dy = py - self.y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < 0.0001 {
            return 1000.0;
        }
        (self.radius * self.radius) / dist_sq
    }
}

struct MetaballScene {
    blobs: Vec<Blob>,
    time: f64,
    mode: RenderMode,
    mode_timer: f64,
}

impl MetaballScene {
    fn new() -> Self {
        Self {
            blobs: vec![
                Blob::new(0.0, 0.0, 4.0),
                Blob::new(0.0, 0.0, 3.0),
                Blob::new(0.0, 0.0, 3.5),
                Blob::new(0.0, 0.0, 2.5),
                Blob::new(0.0, 0.0, 3.2),
            ],
            time: 0.0,
            mode: RenderMode::Gradient,
            mode_timer: 0.0,
        }
    }

    fn update(&mut self, dt: f64) {
        self.time += dt;
        self.mode_timer += dt;

        // Cycle modes every 5 seconds
        if self.mode_timer > 5.0 {
            self.mode_timer = 0.0;
            self.mode = self.mode.next();
        }

        let t = self.time;
        let cx = SCREEN_WIDTH as f64 / 2.0;
        let cy = SCREEN_HEIGHT as f64 / 2.0;

        // Main blob - slight wobble at center
        self.blobs[0].x = cx + (t * 0.5).sin() * 8.0;
        self.blobs[0].y = cy + (t * 0.7).cos() * 4.0;

        // Orbiting blobs
        self.blobs[1].x = cx + (t * 1.2).cos() * 20.0;
        self.blobs[1].y = cy + (t * 1.2).sin() * 10.0;

        self.blobs[2].x = cx + (t * 0.8 + PI * 0.5).cos() * 25.0;
        self.blobs[2].y = cy + (t * 0.8 + PI * 0.5).sin() * 11.0;

        self.blobs[3].x = cx + (t * 1.5 + PI).cos() * 18.0;
        self.blobs[3].y = cy + (t * 1.5 + PI).sin() * 8.0;

        self.blobs[4].x = cx + (t * 0.6 + PI * 1.5).cos() * 28.0;
        self.blobs[4].y = cy + (t * 0.6 + PI * 1.5).sin() * 12.0;
    }

    fn calculate_field(&self, x: f64, y: f64) -> f64 {
        self.blobs.iter().map(|b| b.field_at(x, y)).sum()
    }

    fn render(&self) -> String {
        let mut buffer = String::with_capacity(SCREEN_WIDTH * SCREEN_HEIGHT * 4);

        // Pre-calculate field values for edge detection
        let mut field_grid: Vec<Vec<f64>> = vec![vec![0.0; SCREEN_WIDTH + 1]; SCREEN_HEIGHT + 1];
        for row in 0..=SCREEN_HEIGHT {
            for col in 0..=SCREEN_WIDTH {
                field_grid[row][col] = self.calculate_field(col as f64, row as f64);
            }
        }

        for row in 0..SCREEN_HEIGHT {
            for col in 0..SCREEN_WIDTH {
                let field = field_grid[row][col];
                let ch = match self.mode {
                    RenderMode::Gradient => self.render_gradient(field),
                    RenderMode::Contour => self.render_contour(&field_grid, row, col),
                    RenderMode::Solid => self.render_solid(field),
                    RenderMode::Blocks => self.render_blocks(&field_grid, row, col),
                    RenderMode::Gooey => self.render_gooey(field),
                };
                buffer.push(ch);
            }
            buffer.push('\n');
        }

        buffer
    }

    fn render_gradient(&self, field: f64) -> char {
        const GRADIENT: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
        if field < THRESHOLD * 0.1 {
            ' '
        } else if field >= THRESHOLD {
            let intensity = (field - THRESHOLD).min(3.0) / 3.0;
            let idx = 5 + (intensity * 4.0) as usize;
            GRADIENT[idx.min(GRADIENT.len() - 1)]
        } else {
            let intensity = field / THRESHOLD;
            let idx = (intensity * 5.0) as usize;
            GRADIENT[idx.min(4)]
        }
    }

    fn render_contour(&self, grid: &[Vec<f64>], row: usize, col: usize) -> char {
        let field = grid[row][col];
        let inside = field >= THRESHOLD;

        // Check neighbors for edge detection
        let neighbors = [
            (row.saturating_sub(1), col),
            (row + 1, col),
            (row, col.saturating_sub(1)),
            (row, col + 1),
        ];

        let mut is_edge = false;
        for (nr, nc) in neighbors {
            if nr < grid.len() && nc < grid[0].len() {
                let neighbor_inside = grid[nr][nc] >= THRESHOLD;
                if inside != neighbor_inside {
                    is_edge = true;
                    break;
                }
            }
        }

        if is_edge {
            // Edge character based on field strength (thicker where blobs merge)
            if field > THRESHOLD * 1.5 {
                '@'
            } else if field > THRESHOLD * 1.2 {
                '#'
            } else {
                'O'
            }
        } else if inside {
            // Inside - show subtle fill
            '.'
        } else {
            ' '
        }
    }

    fn render_solid(&self, field: f64) -> char {
        if field >= THRESHOLD {
            if field > THRESHOLD * 3.0 {
                '@'
            } else if field > THRESHOLD * 2.0 {
                '#'
            } else {
                '*'
            }
        } else {
            ' '
        }
    }

    fn render_blocks(&self, grid: &[Vec<f64>], row: usize, col: usize) -> char {
        // Use 2x2 sub-pixel sampling for smoother edges
        let mut count = 0;
        for dy in [0.0, 0.5] {
            for dx in [0.0, 0.5] {
                let x = col as f64 + dx;
                let y = row as f64 + dy;
                if self.calculate_field(x, y) >= THRESHOLD {
                    count += 1;
                }
            }
        }

        // Map to block characters
        let field = grid[row][col];
        match count {
            0 => ' ',
            1 => '░',
            2 => '▒',
            3 => '▓',
            4 => if field > THRESHOLD * 2.0 { '█' } else { '▓' },
            _ => '█',
        }
    }

    fn render_gooey(&self, field: f64) -> char {
        // Emphasize the "gooey" merge areas with special characters
        if field < THRESHOLD * 0.3 {
            ' '
        } else if field < THRESHOLD * 0.6 {
            '·'
        } else if field < THRESHOLD * 0.9 {
            '○'
        } else if field < THRESHOLD {
            '◯'
        } else if field < THRESHOLD * 1.3 {
            // Just inside - the "skin"
            '●'
        } else if field < THRESHOLD * 2.0 {
            // Deeper inside
            '◉'
        } else {
            // Core / merge zone
            '◈'
        }
    }
}

fn main() {
    let mut scene = MetaballScene::new();
    let mut stdout = io::stdout();

    print!("\x1B[?25l"); // Hide cursor
    print!("\x1B[2J");   // Clear screen

    let frame_duration = Duration::from_millis(33);
    let start_time = Instant::now();
    let mut frame_count: u64 = 0;

    loop {
        let frame_start = Instant::now();

        print!("\x1B[H");

        scene.update(0.05);
        let frame = scene.render();
        print!("{}", frame);

        let elapsed = start_time.elapsed().as_secs_f64();
        frame_count += 1;
        print!(
            "Metaballs [{}] | Frame: {} | FPS: {:.1}\x1B[K",
            scene.mode.name(),
            frame_count,
            frame_count as f64 / elapsed
        );

        stdout.flush().unwrap();

        let elapsed_frame = frame_start.elapsed();
        if elapsed_frame < frame_duration {
            thread::sleep(frame_duration - elapsed_frame);
        }
    }
}
