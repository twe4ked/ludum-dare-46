use crate::entities::*;
use legion::prelude::*;

pub fn input(world: &mut World) {
    // Handle input
    let query = <(Write<Velocity>, Write<Player>)>::query();
    for (mut velocity, mut player) in query.iter(world) {
        let input = crate::GLOBAL_KEY.lock().unwrap();
        if let Some(key_code) = *input {
            match key_code {
                87 => {
                    // up

                    // We're not jumping, so we can start jumping
                    if player.jumping == 0. {
                        player.jumping = 120.;
                    }

                    if player.jumping > 100. {
                        // keep adding velocity
                        velocity.dy -= 10.;
                    }
                }
                83 => velocity.dy += 1., // down
                65 => velocity.dx -= 1., // left
                68 => velocity.dx += 1., // right
                _ => {}
            }
            crate::log!("{:?}", input);
        }

        // Jumping
        player.jumping = clamp(player.jumping - 1.0, 0., 120.);
    }
}

pub fn update_velocity(world: &mut World) {
    let query = <Write<Velocity>>::query();
    for mut velocity in query.iter(world) {
        // Gravity
        velocity.dy += 0.1;

        // Friction
        if velocity.dx > 0. {
            // moving right
            velocity.dx -= 0.1;
            velocity.dx = clamp(velocity.dx, 0., 3.);
        }
        if velocity.dx < 0. {
            // moving left
            velocity.dx += 0.1;
            velocity.dx = clamp(velocity.dx, -3., 0.);
        }

        // Clamp velocity
        velocity.dy = clamp(velocity.dy, -3., 3.);
        velocity.dx = clamp(velocity.dx, -3., 3.);
    }
}

pub fn apply_velocity(world: &mut World) {
    let query = <(Write<Position>, Read<Velocity>)>::query();
    for (mut position, velocity) in query.iter(world) {
        position.y += velocity.dy;
        position.x += velocity.dx;
    }
}

pub fn player_collision(world: &mut World) {
    let query = <(Read<Position>, Read<Rect>)>::query().filter(!component::<Player>());
    let static_objects: Vec<(Position, Rect)> = query
        .iter_immutable(world)
        .map(|(pos, rect)| (*pos, *rect))
        .collect();

    let query = <(Write<Position>, Read<Rect>, Write<Velocity>, Read<Player>)>::query();
    for (mut position, rect, mut velocity, _player) in query.iter(world) {
        for (wall_position, wall_rect) in static_objects.iter() {
            if collision(
                position.x,
                position.y,
                rect.width,
                rect.height,
                wall_position.x,
                wall_position.y,
                wall_rect.width,
                wall_rect.height,
            ) {
                position.y -= velocity.dy;
                velocity.dy = 0.0;
            }
        }
    }
}

fn collision(
    r1x: f32,
    r1y: f32,
    r1w: f32,
    r1h: f32,
    r2x: f32,
    r2y: f32,
    r2w: f32,
    r2h: f32,
) -> bool {
    r1x < r2x + r2w && r1x + r1w > r2x && r1y < r2y + r2h && r1y + r1h > r2y
}

fn clamp(x: f32, min: f32, max: f32) -> f32 {
    x.max(min).min(max)
}
