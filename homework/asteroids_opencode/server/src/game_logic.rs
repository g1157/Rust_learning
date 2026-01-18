//! 游戏逻辑和物理模块

use uuid::Uuid;

use crate::types::{
    GameMode, GameState, PowerupType, ServerAsteroidState, ServerBulletState,
    ServerPowerupState, ServerVortexState,
};

/// 游戏常量（与客户端匹配）
pub mod game_constants {
    pub const SCREEN_WIDTH: f32 = 1024.0;
    pub const SCREEN_HEIGHT: f32 = 768.0;

    pub const SHIP_ACCEL: f32 = 200.0;
    pub const SHIP_ROTATION_SPEED: f32 = 4.0;
    pub const MAX_SPEED: f32 = 300.0;
    pub const FRICTION: f32 = 0.99;
    pub const SHIP_RADIUS: f32 = 12.5;
    pub const BULLET_SPEED: f32 = 500.0;
    pub const BULLET_RADIUS: f32 = 3.0;
    pub const BULLET_LIFETIME: f32 = 1.5;
    pub const SHOOT_COOLDOWN: f32 = 0.15;

    pub const VORTEX_STRENGTH: f32 = 220.0;
    pub const VORTEX_PULL_STRENGTH: f32 = 320.0;
    pub const VORTEX_RADIUS: f32 = 200.0;
    pub const VORTEX_LIFETIME: f32 = 15.0;
    pub const VORTEX_SPAWN_INTERVAL: f32 = 20.0;

    pub const POWERUP_PICKUP_RADIUS: f32 = 30.0;
    pub const POWERUP_LIFETIME: f32 = 8.0;
    pub const POWERUP_SPAWN_INTERVAL: f32 = 12.0;

    pub fn asteroid_radius(size: u32) -> f32 {
        match size {
            3 => 40.0,
            2 => 25.0,
            _ => 15.0,
        }
    }
}

/// 更新游戏物理
pub fn update_game_physics(
    state: &mut GameState,
    dt: f32,
    screen_width: f32,
    screen_height: f32,
    mode: GameMode,
) {
    use game_constants::*;

    const INPUT_TIMEOUT: f32 = 0.3;
    let now_secs = state.start_time.elapsed().as_secs_f32();

    let mut new_bullets: Vec<ServerBulletState> = Vec::new();
    let mut next_bullet_id = state.bullets.iter().map(|b| b.id).max().unwrap_or(0) + 1;

    // 更新玩家
    for (player_id, player) in state.players.iter_mut() {
        if !player.alive {
            continue;
        }

        // 输入超时保护
        if now_secs - player.last_input_at > INPUT_TIMEOUT {
            player.thrust = false;
            player.turn_left = false;
            player.turn_right = false;
            player.shoot = false;
        }

        // 旋转
        if player.turn_left {
            player.angle -= SHIP_ROTATION_SPEED * dt;
        }
        if player.turn_right {
            player.angle += SHIP_ROTATION_SPEED * dt;
        }

        // 推进
        if player.thrust {
            player.vel_x += player.angle.cos() * SHIP_ACCEL * dt;
            player.vel_y += player.angle.sin() * SHIP_ACCEL * dt;
        }

        // 限速
        let speed = (player.vel_x * player.vel_x + player.vel_y * player.vel_y).sqrt();
        if speed > MAX_SPEED {
            let scale = MAX_SPEED / speed;
            player.vel_x *= scale;
            player.vel_y *= scale;
        }

        // 摩擦
        player.vel_x *= FRICTION;
        player.vel_y *= FRICTION;

        // 移动
        player.x += player.vel_x * dt;
        player.y += player.vel_y * dt;

        // 屏幕环绕
        wrap_position(&mut player.x, &mut player.y, screen_width, screen_height);

        // 射击
        if player.shoot && player.shoot_cooldown <= 0.0 {
            let bullet = ServerBulletState {
                id: next_bullet_id,
                owner_id: *player_id,
                x: player.x + player.angle.cos() * SHIP_RADIUS,
                y: player.y + player.angle.sin() * SHIP_RADIUS,
                vx: player.angle.cos() * BULLET_SPEED,
                vy: player.angle.sin() * BULLET_SPEED,
                lifetime: BULLET_LIFETIME,
            };
            new_bullets.push(bullet);
            next_bullet_id += 1;
            player.shoot_cooldown = SHOOT_COOLDOWN;
        }

        player.shoot_cooldown = (player.shoot_cooldown - dt).max(0.0);
    }

    state.bullets.extend(new_bullets);

    // 漩涡对玩家影响
    apply_vortex_forces(state, dt);

    // 更新小行星
    for asteroid in &mut state.asteroids {
        asteroid.x += asteroid.vx * dt;
        asteroid.y += asteroid.vy * dt;
        asteroid.angle += 0.5 * dt;
        wrap_position(&mut asteroid.x, &mut asteroid.y, screen_width, screen_height);
    }

    // 更新子弹
    for bullet in &mut state.bullets {
        bullet.x += bullet.vx * dt;
        bullet.y += bullet.vy * dt;
        bullet.lifetime -= dt;
        wrap_position(&mut bullet.x, &mut bullet.y, screen_width, screen_height);
    }

    state.bullets.retain(|b| b.lifetime > 0.0);

    // 碰撞检测
    process_bullet_asteroid_collisions(state);
    process_ship_collisions(state, dt, screen_width, screen_height, mode);
}

/// 屏幕环绕
fn wrap_position(x: &mut f32, y: &mut f32, width: f32, height: f32) {
    if *x < 0.0 {
        *x += width;
    }
    if *x > width {
        *x -= width;
    }
    if *y < 0.0 {
        *y += height;
    }
    if *y > height {
        *y -= height;
    }
}

/// 应用漩涡力
fn apply_vortex_forces(state: &mut GameState, dt: f32) {
    use game_constants::*;

    for player in state.players.values_mut() {
        if !player.alive {
            continue;
        }
        for vortex in &state.vortices {
            let dx = vortex.x - player.x;
            let dy = vortex.y - player.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < vortex.radius * vortex.radius && dist_sq > 1.0 {
                let dist = dist_sq.sqrt();
                let tangent_x = -dy / dist;
                let tangent_y = dx / dist;
                let tangent_force = VORTEX_STRENGTH * vortex.strength * dt / dist;
                player.vel_x += tangent_x * tangent_force;
                player.vel_y += tangent_y * tangent_force;
                let pull_force = VORTEX_PULL_STRENGTH * dt / dist;
                player.vel_x += dx / dist * pull_force;
                player.vel_y += dy / dist * pull_force;
            }
        }
    }
}

/// 子弹-小行星碰撞
fn process_bullet_asteroid_collisions(state: &mut GameState) {
    use game_constants::*;

    let mut bullets_to_remove: Vec<u32> = Vec::new();
    let mut asteroids_to_remove: Vec<u32> = Vec::new();
    let mut new_asteroids: Vec<ServerAsteroidState> = Vec::new();
    let mut score_updates: Vec<(Uuid, u32)> = Vec::new();
    let mut next_asteroid_id = state.asteroids.iter().map(|a| a.id).max().unwrap_or(0) + 1;

    for bullet in &state.bullets {
        for asteroid in &state.asteroids {
            let dx = bullet.x - asteroid.x;
            let dy = bullet.y - asteroid.y;
            let dist_sq = dx * dx + dy * dy;
            let hit_dist = BULLET_RADIUS + asteroid_radius(asteroid.size);

            if dist_sq < hit_dist * hit_dist {
                bullets_to_remove.push(bullet.id);
                asteroids_to_remove.push(asteroid.id);

                let points = match asteroid.size {
                    3 => 10,
                    2 => 20,
                    _ => 50,
                };
                score_updates.push((bullet.owner_id, points));

                if asteroid.size > 1 {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let new_size = asteroid.size - 1;

                    for _ in 0..2 {
                        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                        let speed = rng.gen_range(50.0..100.0);
                        new_asteroids.push(ServerAsteroidState {
                            id: next_asteroid_id,
                            x: asteroid.x,
                            y: asteroid.y,
                            vx: angle.cos() * speed,
                            vy: angle.sin() * speed,
                            size: new_size,
                            angle: rng.gen_range(0.0..std::f32::consts::TAU),
                        });
                        next_asteroid_id += 1;
                    }
                }
                break;
            }
        }
    }

    state.bullets.retain(|b| !bullets_to_remove.contains(&b.id));
    state.asteroids.retain(|a| !asteroids_to_remove.contains(&a.id));
    state.asteroids.extend(new_asteroids);

    for (player_id, points) in score_updates {
        if let Some(player) = state.players.get_mut(&player_id) {
            player.score += points;
        }
    }
}

/// 飞船碰撞检测
fn process_ship_collisions(
    state: &mut GameState,
    dt: f32,
    screen_width: f32,
    screen_height: f32,
    mode: GameMode,
) {
    use game_constants::*;

    let player_ids: Vec<Uuid> = state.players.keys().copied().collect();
    let mut ship_bullets_to_remove: Vec<u32> = Vec::new();

    for &player_id in &player_ids {
        let player = match state.players.get(&player_id) {
            Some(p) => p.clone(),
            None => continue,
        };

        if !player.alive {
            continue;
        }

        if player.invulnerable_until > 0.0 {
            if let Some(p) = state.players.get_mut(&player_id) {
                p.invulnerable_until = (p.invulnerable_until - dt).max(0.0);
            }
            continue;
        }

        let mut hit = false;

        // Duel 模式敌方子弹碰撞
        if mode == GameMode::Duel {
            for bullet in &state.bullets {
                if bullet.owner_id == player_id || ship_bullets_to_remove.contains(&bullet.id) {
                    continue;
                }

                let dx = player.x - bullet.x;
                let dy = player.y - bullet.y;
                let dist_sq = dx * dx + dy * dy;
                let hit_dist = SHIP_RADIUS + BULLET_RADIUS;

                if dist_sq < hit_dist * hit_dist {
                    ship_bullets_to_remove.push(bullet.id);
                    hit = true;
                    break;
                }
            }
        }

        // 小行星碰撞
        if !hit {
            for asteroid in &state.asteroids {
                let dx = player.x - asteroid.x;
                let dy = player.y - asteroid.y;
                let dist_sq = dx * dx + dy * dy;
                let hit_dist = SHIP_RADIUS + asteroid_radius(asteroid.size);

                if dist_sq < hit_dist * hit_dist {
                    hit = true;
                    break;
                }
            }
        }

        if hit {
            if let Some(p) = state.players.get_mut(&player_id) {
                if p.lives > 0 {
                    p.lives -= 1;
                }
                if p.lives == 0 {
                    p.alive = false;
                } else {
                    p.invulnerable_until = 2.0;
                    p.x = screen_width / 2.0;
                    p.y = screen_height / 2.0;
                    p.vel_x = 0.0;
                    p.vel_y = 0.0;
                }
            }
        }
    }

    state.bullets.retain(|b| !ship_bullets_to_remove.contains(&b.id));
}

/// 生成、更新漩涡与道具
pub fn update_world_events(state: &mut GameState, screen_width: f32, screen_height: f32) {
    use game_constants::*;
    use rand::Rng;

    let now_secs = state.start_time.elapsed().as_secs_f32();
    let now_secs_f64 = state.start_time.elapsed().as_secs_f64();

    // 生成漩涡
    if state.next_vortex_spawn <= now_secs {
        let mut rng = rand::thread_rng();
        let id = state.vortices.iter().map(|v| v.id).max().unwrap_or(0) + 1;
        let margin = VORTEX_RADIUS;
        let x = rng.gen_range(margin..screen_width - margin);
        let y = rng.gen_range(margin..screen_height - margin);
        let strength = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };

        state.vortices.push(ServerVortexState {
            id,
            x,
            y,
            strength,
            radius: VORTEX_RADIUS,
            created_at: now_secs,
            lifetime: VORTEX_LIFETIME,
        });
        state.next_vortex_spawn = now_secs + VORTEX_SPAWN_INTERVAL;
    }

    // 生成道具
    if state.next_powerup_spawn <= now_secs {
        let mut rng = rand::thread_rng();
        let id = state.powerups.iter().map(|p| p.id).max().unwrap_or(0) + 1;
        let powerup_type = match rng.gen_range(0..3) {
            0 => PowerupType::Shield,
            1 => PowerupType::DualShot,
            _ => PowerupType::TripleShot,
        };
        let x = rng.gen_range(50.0..screen_width - 50.0);
        let y = rng.gen_range(50.0..screen_height - 50.0);

        state.powerups.push(ServerPowerupState {
            id,
            x,
            y,
            expires_at: now_secs_f64 + POWERUP_LIFETIME as f64,
            collected: false,
            powerup_type,
        });
        state.next_powerup_spawn = now_secs + POWERUP_SPAWN_INTERVAL;
    }

    // 清理过期漩涡
    state.vortices.retain(|v| now_secs - v.created_at < v.lifetime);

    // 道具拾取
    for powerup in &mut state.powerups {
        if powerup.collected {
            continue;
        }
        if now_secs_f64 >= powerup.expires_at {
            powerup.collected = true;
            continue;
        }

        for player in state.players.values() {
            if !player.alive {
                continue;
            }
            let dx = player.x - powerup.x;
            let dy = player.y - powerup.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < POWERUP_PICKUP_RADIUS * POWERUP_PICKUP_RADIUS {
                powerup.collected = true;
                break;
            }
        }
    }

    state.powerups.retain(|p| !p.collected);
}

/// 检查游戏结束条件
pub fn check_game_over(state: &GameState, mode: GameMode) -> bool {
    match mode {
        GameMode::Duel => {
            let alive_count = state.players.values().filter(|p| p.alive).count();
            alive_count <= 1
        }
        GameMode::Survival => !state.players.values().any(|p| p.alive),
    }
}

/// 计算两点间距离的平方
pub fn distance_squared(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    dx * dx + dy * dy
}

/// 圆-圆碰撞检测
pub fn circles_collide(x1: f32, y1: f32, r1: f32, x2: f32, y2: f32, r2: f32) -> bool {
    let dist_sq = distance_squared(x1, y1, x2, y2);
    let radii_sum = r1 + r2;
    dist_sq < radii_sum * radii_sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asteroid_radius() {
        assert_eq!(game_constants::asteroid_radius(3), 40.0);
        assert_eq!(game_constants::asteroid_radius(2), 25.0);
        assert_eq!(game_constants::asteroid_radius(1), 15.0);
        assert_eq!(game_constants::asteroid_radius(0), 15.0);
    }

    #[test]
    fn test_wrap_position() {
        let mut x = -10.0;
        let mut y = 800.0;
        wrap_position(&mut x, &mut y, 1024.0, 768.0);
        assert!(x > 0.0);
        assert!(y < 768.0);
    }

    #[test]
    fn test_distance_squared() {
        assert_eq!(distance_squared(0.0, 0.0, 3.0, 4.0), 25.0);
        assert_eq!(distance_squared(1.0, 1.0, 1.0, 1.0), 0.0);
    }

    #[test]
    fn test_circles_collide() {
        // 重叠的圆
        assert!(circles_collide(0.0, 0.0, 10.0, 5.0, 0.0, 10.0));
        // 不重叠的圆
        assert!(!circles_collide(0.0, 0.0, 5.0, 20.0, 0.0, 5.0));
        // 刚好接触
        assert!(circles_collide(0.0, 0.0, 5.0, 10.0, 0.0, 5.01));
    }

    #[test]
    fn test_check_game_over_duel() {
        use std::collections::HashMap;
        use std::time::Instant;

        let mut players = HashMap::new();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();

        players.insert(p1, crate::types::ServerPlayerState {
            alive: true,
            ..Default::default()
        });
        players.insert(p2, crate::types::ServerPlayerState {
            alive: true,
            ..Default::default()
        });

        let state = GameState {
            players,
            asteroids: vec![],
            bullets: vec![],
            vortices: vec![],
            powerups: vec![],
            next_vortex_spawn: 20.0,
            next_powerup_spawn: 12.0,
            start_time: Instant::now(),
            last_update: Instant::now(),
        };

        // 两人存活，游戏继续
        assert!(!check_game_over(&state, GameMode::Duel));
    }

    #[test]
    fn test_check_game_over_survival() {
        use std::collections::HashMap;
        use std::time::Instant;

        let mut players = HashMap::new();
        let p1 = Uuid::new_v4();

        players.insert(p1, crate::types::ServerPlayerState {
            alive: false,
            ..Default::default()
        });

        let state = GameState {
            players,
            asteroids: vec![],
            bullets: vec![],
            vortices: vec![],
            powerups: vec![],
            next_vortex_spawn: 20.0,
            next_powerup_spawn: 12.0,
            start_time: Instant::now(),
            last_update: Instant::now(),
        };

        // 所有玩家死亡，游戏结束
        assert!(check_game_over(&state, GameMode::Survival));
    }
}
