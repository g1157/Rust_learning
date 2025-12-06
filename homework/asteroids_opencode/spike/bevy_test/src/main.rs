//! Bevy Spike - 最小场景验证
//!
//! 验证 Bevy 是否适合我们的项目
//! 实现：飞船移动 + 射击 + 单颗小行星

use bevy::prelude::*;
use std::time::Duration;

// ============================================================================
// 组件定义
// ============================================================================

/// 飞船组件
#[derive(Component)]
struct Ship {
    thrust: bool,
    turn_left: bool,
    turn_right: bool,
    shoot: bool,
    shoot_timer: Timer,
}

/// 子弹组件
#[derive(Component)]
struct Bullet {
    lifetime: Timer,
}

/// 小行星组件
#[derive(Component)]
struct Asteroid {
    size: f32,
}

// ============================================================================
// 系统实现
// ============================================================================

/// 飞船移动系统
fn ship_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Ship)>,
) {
    for (mut transform, mut ship) in query.iter_mut() {
        // 旋转
        if ship.turn_left {
            transform.rotate_z(-3.0 * time.delta_seconds());
        }
        if ship.turn_right {
            transform.rotate_z(3.0 * time.delta_seconds());
        }

        // 推进（简化：直接修改位置）
        if ship.thrust {
            let forward = transform.forward();
            let thrust = Vec3::new(forward.x, forward.y, 0.0) * 100.0 * time.delta_seconds();
            transform.translation += thrust;
        }
    }
}

/// 边界包裹系统
fn wrap_around_system(
    mut query: Query<&mut Transform>,
) {
    for mut transform in query.iter_mut() {
        let pos = &mut transform.translation;

        if pos.x > 400.0 {
            pos.x = -400.0;
        } else if pos.x < -400.0 {
            pos.x = 400.0;
        }

        if pos.y > 300.0 {
            pos.y = -300.0;
        } else if pos.y < -300.0 {
            pos.y = 300.0;
        }
    }
}

/// 射击系统
fn shooting_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&Transform, &mut Ship)>,
) {
    for (transform, mut ship) in query.iter_mut() {
        ship.shoot_timer.tick(time.delta());

        if ship.shoot && ship.shoot_timer.finished() {
            // 创建子弹
            let bullet_pos = transform.translation + transform.forward() * 20.0;
            let bullet_vel = transform.forward() * 200.0;

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::YELLOW,
                        custom_size: Some(Vec2::new(6.0, 6.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(bullet_pos),
                    ..default()
                },
                Bullet {
                    lifetime: Timer::from_seconds(2.0, TimerMode::Once),
                },
                Velocity(bullet_vel),
            ));

            ship.shoot_timer.reset();
        }
    }
}

/// 子弹移动和生命周期系统
fn bullet_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet, &mut Transform, &Velocity)>,
) {
    for (entity, mut bullet, mut transform, velocity) in query.iter_mut() {
        bullet.lifetime.tick(time.delta());

        // 移动子弹
        transform.translation += velocity.0 * time.delta_seconds();

        // 生命周期结束
        if bullet.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// 碰撞检测系统
fn collision_system(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    asteroid_query: Query<(Entity, &Transform), With<Asteroid>>,
) {
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        for (asteroid_entity, asteroid_transform) in asteroid_query.iter() {
            let distance = bullet_transform.translation.distance(asteroid_transform.translation);
            if distance < 25.0 { // 碰撞距离
                commands.entity(bullet_entity).despawn();
                commands.entity(asteroid_entity).despawn();

                println!("小行星被摧毁！");

                // 这里可以创建新的小行星碎片
                break;
            }
        }
    }
}

// ============================================================================
// 输入处理
// ============================================================================

/// 输入处理系统
fn input_system(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<&mut Ship>,
) {
    for mut ship in query.iter_mut() {
        ship.thrust = keyboard.pressed(KeyCode::W);
        ship.turn_left = keyboard.pressed(KeyCode::A);
        ship.turn_right = keyboard.pressed(KeyCode::D);
        ship.shoot = keyboard.pressed(KeyCode::Space);
    }
}

/// 通用速度组件（用于子弹等）
#[derive(Component)]
struct Velocity(Vec3);

// ============================================================================
// 主函数
// ============================================================================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_system)
        .add_system(input_system)
        .add_system(ship_movement_system)
        .add_system(wrap_around_system)
        .add_system(shooting_system)
        .add_system(bullet_system)
        .add_system(collision_system)
        .run();
}

/// 初始化系统
fn setup_system(mut commands: Commands) {
    // 创建相机
    commands.spawn(Camera2dBundle::default());

    // 创建飞船
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
        Ship {
            thrust: false,
            turn_left: false,
            turn_right: false,
            shoot: false,
            shoot_timer: Timer::from_seconds(0.2, TimerMode::Once),
        },
    ));

    // 创建小行星
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::GRAY,
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(200.0, 100.0, 0.0)),
            ..default()
        },
        Asteroid { size: 20.0 },
    ));

    println!("Bevy Spike 已启动！");
    println!("控制：W(推进) A/D(转向) 空格(射击)");
}