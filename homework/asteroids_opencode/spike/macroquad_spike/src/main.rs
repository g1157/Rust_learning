//! Macroquad Spike - ECS概念验证
//!
//! 使用简单的结构体模拟ECS，验证飞船移动+射击+小行星的概念

use macroquad::prelude::*;
use std::collections::VecDeque;

// ============================================================================
// 简单ECS模拟
// ============================================================================

struct World {
    ships: Vec<Ship>,
    bullets: Vec<Bullet>,
    asteroids: Vec<Asteroid>,
}

impl World {
    fn new() -> Self {
        Self {
            ships: Vec::new(),
            bullets: Vec::new(),
            asteroids: Vec::new(),
        }
    }

    fn update(&mut self, dt: f32) {
        // 飞船移动系统
        for ship in &mut self.ships {
            ship.update(dt);
        }

        // 子弹移动和生命周期系统
        self.bullets.retain_mut(|bullet| {
            bullet.update(dt);
            bullet.lifetime > 0.0
        });

        // 碰撞检测系统
        self.collision_system();

        // 边界包裹系统
        for ship in &mut self.ships {
            ship.wrap_around();
        }
        for bullet in &mut self.bullets {
            bullet.wrap_around();
        }
        for asteroid in &mut self.asteroids {
            asteroid.wrap_around();
        }
    }

    fn collision_system(&mut self) {
        let mut bullets_to_remove = Vec::new();
        let mut asteroids_to_remove = Vec::new();

        for (bullet_idx, bullet) in self.bullets.iter().enumerate() {
            for (asteroid_idx, asteroid) in self.asteroids.iter().enumerate() {
                let dx = bullet.x - asteroid.x;
                let dy = bullet.y - asteroid.y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance < asteroid.size + 5.0 {
                    bullets_to_remove.push(bullet_idx);
                    asteroids_to_remove.push(asteroid_idx);
                    println!("小行星被摧毁！");
                    break;
                }
            }
        }

        // 从后往前移除以保持索引有效
        bullets_to_remove.sort_unstable_by(|a, b| b.cmp(a));
        for idx in bullets_to_remove {
            if idx < self.bullets.len() {
                self.bullets.remove(idx);
            }
        }

        asteroids_to_remove.sort_unstable_by(|a, b| b.cmp(a));
        for idx in asteroids_to_remove {
            if idx < self.asteroids.len() {
                self.asteroids.remove(idx);
            }
        }
    }

    fn render(&self) {
        // 渲染飞船
        for ship in &self.ships {
            ship.render();
        }

        // 渲染子弹
        for bullet in &self.bullets {
            bullet.render();
        }

        // 渲染小行星
        for asteroid in &self.asteroids {
            asteroid.render();
        }
    }
}

// ============================================================================
// 实体定义
// ============================================================================

struct Ship {
    x: f32,
    y: f32,
    angle: f32,
    thrust: bool,
    turn_left: bool,
    turn_right: bool,
    shoot: bool,
    shoot_timer: f32,
    vel_x: f32,
    vel_y: f32,
}

impl Ship {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            angle: 0.0,
            thrust: false,
            turn_left: false,
            turn_right: false,
            shoot: false,
            shoot_timer: 0.0,
            vel_x: 0.0,
            vel_y: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        // 旋转
        if self.turn_left {
            self.angle -= 3.0 * dt;
        }
        if self.turn_right {
            self.angle += 3.0 * dt;
        }

        // 推进
        if self.thrust {
            self.vel_x += self.angle.cos() * 100.0 * dt;
            self.vel_y += self.angle.sin() * 100.0 * dt;
        }

        // 限速
        let speed = (self.vel_x * self.vel_x + self.vel_y * self.vel_y).sqrt();
        if speed > 150.0 {
            let factor = 150.0 / speed;
            self.vel_x *= factor;
            self.vel_y *= factor;
        }

        // 更新位置
        self.x += self.vel_x * dt;
        self.y += self.vel_y * dt;

        // 射击冷却
        if self.shoot_timer > 0.0 {
            self.shoot_timer -= dt;
        }
    }

    fn wrap_around(&mut self) {
        if self.x > 400.0 {
            self.x = -400.0;
        } else if self.x < -400.0 {
            self.x = 400.0;
        }
        if self.y > 300.0 {
            self.y = -300.0;
        } else if self.y < -300.0 {
            self.y = 300.0;
        }
    }

    fn render(&self) {
        // 绘制飞船（简单的三角形）
        let p1 = Vec2::new(self.x + 15.0 * self.angle.cos(), self.y + 15.0 * self.angle.sin());
        let p2 = Vec2::new(self.x - 7.5 * self.angle.cos() - 13.0 * self.angle.sin(),
                          self.y - 7.5 * self.angle.sin() + 13.0 * self.angle.cos());
        let p3 = Vec2::new(self.x - 7.5 * self.angle.cos() + 13.0 * self.angle.sin(),
                          self.y - 7.5 * self.angle.sin() - 13.0 * self.angle.cos());

        draw_triangle(p1, p2, p3, BLUE);
    }

    fn can_shoot(&self) -> bool {
        self.shoot_timer <= 0.0
    }

    fn shoot(&mut self, world: &mut World) {
        if self.can_shoot() {
            let bullet = Bullet::new(
                self.x + self.angle.cos() * 20.0,
                self.y + self.angle.sin() * 20.0,
                self.angle.cos() * 200.0,
                self.angle.sin() * 200.0,
            );
            world.bullets.push(bullet);
            self.shoot_timer = 0.2;
        }
    }
}

struct Bullet {
    x: f32,
    y: f32,
    vel_x: f32,
    vel_y: f32,
    lifetime: f32,
}

impl Bullet {
    fn new(x: f32, y: f32, vel_x: f32, vel_y: f32) -> Self {
        Self {
            x,
            y,
            vel_x,
            vel_y,
            lifetime: 2.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.x += self.vel_x * dt;
        self.y += self.vel_y * dt;
        self.lifetime -= dt;
    }

    fn wrap_around(&mut self) {
        if self.x > 400.0 {
            self.x = -400.0;
        } else if self.x < -400.0 {
            self.x = 400.0;
        }
        if self.y > 300.0 {
            self.y = -300.0;
        } else if self.y < -300.0 {
            self.y = 300.0;
        }
    }

    fn render(&self) {
        draw_circle(self.x, self.y, 3.0, YELLOW);
    }
}

struct Asteroid {
    x: f32,
    y: f32,
    vel_x: f32,
    vel_y: f32,
    size: f32,
}

impl Asteroid {
    fn new(x: f32, y: f32, vel_x: f32, vel_y: f32, size: f32) -> Self {
        Self {
            x,
            y,
            vel_x,
            vel_y,
            size,
        }
    }

    fn update(&mut self, dt: f32) {
        self.x += self.vel_x * dt;
        self.y += self.vel_y * dt;
    }

    fn wrap_around(&mut self) {
        if self.x > 400.0 {
            self.x = -400.0;
        } else if self.x < -400.0 {
            self.x = 400.0;
        }
        if self.y > 300.0 {
            self.y = -300.0;
        } else if self.y < -300.0 {
            self.y = 300.0;
        }
    }

    fn render(&self) {
        draw_circle(self.x, self.y, self.size, GRAY);
    }
}

// ============================================================================
// 主函数
// ============================================================================

#[macroquad::main("Macroquad Spike")]
async fn main() {
    let mut world = World::new();

    // 创建飞船
    world.ships.push(Ship::new(0.0, 0.0));

    // 创建小行星
    world.asteroids.push(Asteroid::new(200.0, 100.0, -30.0, -20.0, 20.0));

    println!("Macroquad Spike 已启动！");
    println!("控制：W(推进) A/D(转向) 空格(射击)");

    loop {
        clear_background(BLACK);

        // 输入处理
        let thrust = is_key_down(KeyCode::W);
        let turn_left = is_key_down(KeyCode::A);
        let turn_right = is_key_down(KeyCode::D);
        let shoot_pressed = is_key_down(KeyCode::Space);

        if let Some(ship) = world.ships.first_mut() {
            ship.thrust = thrust;
            ship.turn_left = turn_left;
            ship.turn_right = turn_right;
            ship.shoot = shoot_pressed;
        }

        // 射击处理（避免借用冲突）
        if shoot_pressed && !world.ships.is_empty() {
            let ship_x = world.ships[0].x;
            let ship_y = world.ships[0].y;
            let ship_angle = world.ships[0].angle;

            if world.ships[0].can_shoot() {
                let bullet = Bullet::new(
                    ship_x + ship_angle.cos() * 20.0,
                    ship_y + ship_angle.sin() * 20.0,
                    ship_angle.cos() * 200.0,
                    ship_angle.sin() * 200.0,
                );
                world.bullets.push(bullet);
                world.ships[0].shoot_timer = 0.2;
            }
        }

        // 更新世界
        world.update(get_frame_time());

        // 渲染
        world.render();

        // UI
        draw_text("Macroquad ECS Spike", 10.0, 20.0, 20.0, WHITE);
        draw_text(&format!("实体数: {}", world.ships.len() + world.bullets.len() + world.asteroids.len()),
                 10.0, 40.0, 16.0, WHITE);

        next_frame().await
    }
}