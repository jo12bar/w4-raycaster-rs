#![no_main]
#![no_std]
#![allow(internal_features)]
#![feature(core_intrinsics)]

use core::{
    arch::wasm32,
    f32::consts::{FRAC_PI_2, PI, TAU},
    panic::PanicInfo,
};

const DRAW_COLORS: *mut u16 = 0x14 as *mut u16;
const GAMEPAD1: *const u8 = 0x16 as *const u8;

const BUTTON_LEFT: u8 = 16; // 00010000
const BUTTON_RIGHT: u8 = 32; // 00100000
const BUTTON_UP: u8 = 64; // 01000000
const BUTTON_DOWN: u8 = 128; // 10000000

/// How far the player moves per update.
const STEP_SIZE: f32 = 0.045;

const FIVE_PI_SQUARED: f32 = 5.0 * (PI * PI);

/// The player's field of view.
const FOV: f32 = PI / 2.7;
/// Half the player's field of view.
const HALF_FOV: f32 = FOV * 0.5;
/// The angle between each ray used in raycasting.
const ANGLE_STEP: f32 = FOV / 160.0;
/// The height, in pixels, that a wall will appear as when it is one unit away.
const WALL_HEIGHT: f32 = 100.0;

extern "C" {
    fn vline(x: i32, y: i32, len: u32);
}

#[panic_handler]
fn phandler(_: &PanicInfo<'_>) -> ! {
    wasm32::unreachable()
}

#[no_mangle]
unsafe fn update() {
    STATE.update(
        *GAMEPAD1 & BUTTON_UP != 0,
        *GAMEPAD1 & BUTTON_DOWN != 0,
        *GAMEPAD1 & BUTTON_LEFT != 0,
        *GAMEPAD1 & BUTTON_RIGHT != 0,
    );

    // go through each column on screen and draw walls in the center.
    for (x, wall) in STATE.get_view().iter().enumerate() {
        let (height, shadow) = wall;

        if *shadow {
            *DRAW_COLORS = 0x2;
        } else {
            *DRAW_COLORS = 0x3;
        }

        vline(x as i32, 80 - (height / 2), *height as u32);
    }
}

const MAP: [u16; 8] = [
    0b1111111111111111,
    0b1000001010000101,
    0b1011100000110101,
    0b1000111010010001,
    0b1010001011110111,
    0b1011101001100001,
    0b1000100000001101,
    0b1111111111111111,
];

/// Check if the map contains a wall at a point.
fn point_in_wall(x: f32, y: f32) -> bool {
    match MAP.get(y as usize) {
        Some(line) => (line & (0b1 << x as usize)) != 0,
        None => true,
    }
}

fn sinf(mut x: f32) -> f32 {
    let y = x / TAU;
    let z = y - floorf(y);
    x = z * TAU;

    let sinf_imp = |x: f32| -> f32 {
        // these magic numbers were discovered 1400 years ago!
        (16.0 * x * (PI - x)) / (FIVE_PI_SQUARED - (4.0 * x * (PI - x)))
    };

    if x > PI {
        -sinf_imp(x - PI)
    } else {
        sinf_imp(x)
    }
}

fn cosf(x: f32) -> f32 {
    sinf(x + FRAC_PI_2)
}

fn tanf(x: f32) -> f32 {
    sinf(x) / cosf(x)
}

fn sqrtf(x: f32) -> f32 {
    unsafe { core::intrinsics::sqrtf32(x) }
}

fn floorf(x: f32) -> f32 {
    unsafe { core::intrinsics::floorf32(x) }
}

fn ceilf(x: f32) -> f32 {
    unsafe { core::intrinsics::ceilf32(x) }
}

fn fabsf(x: f32) -> f32 {
    unsafe { core::intrinsics::fabsf32(x) }
}

/// Get the distance from (0.0, 0.0) to (x, y).
fn distance(a: f32, b: f32) -> f32 {
    sqrtf((a * a) + (b * b))
}

struct State {
    player_x: f32,
    player_y: f32,
    player_angle: f32,
}

static mut STATE: State = State {
    player_x: 1.5,
    player_y: 1.5,
    player_angle: 0.0,
};

impl State {
    /// Move the character.
    pub fn update(&mut self, up: bool, down: bool, left: bool, right: bool) {
        let prev_pos = (self.player_x, self.player_y);

        if up {
            self.player_x += cosf(self.player_angle) * STEP_SIZE;
            self.player_y += -sinf(self.player_angle) * STEP_SIZE;
        }

        if down {
            self.player_x -= cosf(self.player_angle) * STEP_SIZE;
            self.player_y -= -sinf(self.player_angle) * STEP_SIZE;
        }

        if right {
            self.player_angle -= STEP_SIZE;
        }

        if left {
            self.player_angle += STEP_SIZE;
        }

        // if moving us on this frame put us into a wall just revert it
        if point_in_wall(self.player_x, self.player_y) {
            (self.player_x, self.player_y) = prev_pos;
        }
    }

    /// Returns the nearest wall the ray intersects with on a **horizontal** grid line.
    fn horizontal_intersection(&self, angle: f32) -> f32 {
        // Figure out if the angle is "facing up" on the map.
        let up = fabsf(floorf(angle / PI) % 2.0) != 0.0;

        // first_y and first_x are the first grid intersections that the ray intersects with.
        let first_y = if up {
            ceilf(self.player_y) - self.player_y
        } else {
            floorf(self.player_y) - self.player_y
        };
        let first_x = -first_y / tanf(angle);

        // The vertical and horizontal ray extensions.
        let dy = if up { 1.0 } else { -1.0 };
        let dx = -dy / tanf(angle);

        // next_x and next_y keep track of how far away the ray is from the player.
        // Note that these are relative coordinates.
        let mut next_x = first_x;
        let mut next_y = first_y;

        // Our draw distance is 256 ray extensions.
        for _ in 0..256 {
            // current_x and current_y are absolute coordinate for where the ray
            // currently is on the map.
            let current_x = next_x + self.player_x;
            let current_y = if up {
                next_y + self.player_y
            } else {
                next_y + self.player_y - 1.0
            };

            // Break if we hit a wall
            if point_in_wall(current_x, current_y) {
                break;
            }

            // Otherwise, add dx and dy to our current position and keep going.
            next_x += dx;
            next_y += dy;
        }

        // return the distance from next_x and next_y to the player.
        distance(next_x, next_y)
    }

    /// Returns the nearest wall the ray intersects with on a **vertical** grid line.
    fn vertical_intersection(&self, angle: f32) -> f32 {
        // Figure out if the angle is "facing right" on the map.
        let right = fabsf(floorf((angle - FRAC_PI_2) / PI) % 2.0) != 0.0;

        // first_y and first_x are the first grid intersections that the ray intersects with.
        let first_x = if right {
            ceilf(self.player_x) - self.player_x
        } else {
            floorf(self.player_x) - self.player_x
        };
        let first_y = -tanf(angle) * first_x;

        // The vertical and horizontal ray extensions.
        let dx = if right { 1.0 } else { -1.0 };
        let dy = dx * -tanf(angle);

        // next_x and next_y keep track of how far away the ray is from the player.
        // Note that these are relative coordinates.
        let mut next_x = first_x;
        let mut next_y = first_y;

        // Our draw distance is 256 ray extensions.
        for _ in 0..256 {
            // current_x and current_y are absolute coordinate for where the ray
            // currently is on the map.
            let current_x = if right {
                next_x + self.player_x
            } else {
                next_x + self.player_x - 1.0
            };
            let current_y = next_y + self.player_y;

            // Break if we hit a wall
            if point_in_wall(current_x, current_y) {
                break;
            }

            // Otherwise, add dx and dy to our current position and keep going.
            next_x += dx;
            next_y += dy;
        }

        // return the distance from next_x and next_y to the player.
        distance(next_x, next_y)
    }

    /// Returns 160 wall heights and their color from the player's perspective.
    pub fn get_view(&self) -> [(i32, bool); 160] {
        // The player's FOV is split in half by their viewing angle.
        // In order to get the ray's first angle we must
        // add half the FOV to the player's angle to get
        // the edge of the player's FOV.
        let starting_angle = self.player_angle + HALF_FOV;

        let mut walls = [(0, false); 160];

        // `idx` is what number ray we're on, `wall` is a mutable reference to
        // a value in `walls`.
        for (idx, wall) in walls.iter_mut().enumerate() {
            let angle = starting_angle - idx as f32 * ANGLE_STEP;

            // Get the closest horizontal and vertical wall intersections for this angle.
            let h_dist = self.horizontal_intersection(angle);
            let v_dist = self.vertical_intersection(angle);

            let (min_dist, shadow) = if h_dist < v_dist {
                (h_dist, false)
            } else {
                (v_dist, true)
            };

            // Get the minimum of the two distances and convert it into a wall height.
            *wall = (
                (WALL_HEIGHT / (min_dist * cosf(angle - self.player_angle))) as i32,
                shadow,
            );
        }

        walls
    }
}
