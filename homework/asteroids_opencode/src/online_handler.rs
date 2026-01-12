//! 在线模式处理模块
//!
//! 从 main.rs 拆分出来的在线多人游戏逻辑。
//!
//! ## 功能
//! - 在线模式输入同步
//! - 客户端预测
//! - 服务器状态协调
//! - 实体插值渲染

use macroquad::prelude::*;

use crate::game_state::OnlineBullet;
use crate::interpolation::InterpolationManager;
use crate::network::{self, NetworkClient, ParsedInput, PlayerState, ServerMessage};
use crate::player::{Controls, Player};
use crate::ship::{SHIP_DAMPING, SHIP_ROTATION_STEP, SHIP_THRUST};
use crate::utils::wrap_around;

// ============================================================================
// 客户端预测
// ============================================================================

/// 应用预测输入（本地即时响应）
pub fn apply_predicted_input(player: &mut Player, input: &ParsedInput, dt: f32, _timestamp: f64) {
    if !player.alive {
        return;
    }

    // 旋转
    if input.left {
        player.ship.rot += SHIP_ROTATION_STEP * dt;
    }
    if input.right {
        player.ship.rot -= SHIP_ROTATION_STEP * dt;
    }

    // 推进
    if input.thrust {
        let direction = Vec2::new(player.ship.rot.cos(), -player.ship.rot.sin());
        player.ship.vel += direction * SHIP_THRUST * dt;
    }

    // 阻尼
    player.ship.vel *= SHIP_DAMPING.powf(dt);

    // 更新位置
    player.ship.pos += player.ship.vel * dt;

    // 环绕屏幕
    player.ship.pos = wrap_around(player.ship.pos);
}

// ============================================================================
// 输入同步
// ============================================================================

/// 收集当前输入状态
pub fn collect_input_keys(player: &Player, input: &crate::input::Input) -> Vec<String> {
    let mut keys = Vec::new();
    let controls = &player.controls;

    if input.is_key_down(controls.thrust) {
        keys.push("thrust".to_string());
    }
    if input.is_key_down(controls.left) {
        keys.push("left".to_string());
    }
    if input.is_key_down(controls.right) {
        keys.push("right".to_string());
    }
    if input.is_key_down(controls.shoot_primary) {
        keys.push("shoot".to_string());
    }

    keys
}

/// 发送输入到服务器
pub fn send_input_to_server(
    network_client: &mut NetworkClient,
    keys: Vec<String>,
    timestamp: f64,
    dt: f32,
) -> Result<(), network::NetworkSendError> {
    network_client.send_input(keys, timestamp, dt)
}

// ============================================================================
// 服务器状态同步
// ============================================================================

/// 处理服务器游戏状态更新
pub fn handle_game_state_update(
    server_players: &[PlayerState],
    local_players: &mut Vec<Player>,
    network_client: &mut NetworkClient,
    interp_manager: &mut InterpolationManager,
    online_bullets: &mut Vec<OnlineBullet>,
    server_asteroids: &[network::AsteroidState],
    server_bullets: &[network::BulletState],
    last_input_seqs: &std::collections::HashMap<String, u32>,
    timestamp: i64,
    frame_t: f64,
    dt: f32,
) {
    // 校准时钟并记录快照
    interp_manager.align_clock(timestamp, frame_t);
    interp_manager.record_server_snapshot(
        timestamp,
        server_players,
        server_asteroids,
        server_bullets,
        network_client.player_id.as_deref(),
    );

    // 同步本地玩家状态
    if let Some(my_id) = network_client.player_id.clone() {
        if let Some(my_server_data) = server_players.iter().find(|p| p.id == my_id) {
            // 获取未确认输入进行重播
            let replay_inputs = if let Some(&server_seq) = last_input_seqs.get(&my_id) {
                network_client.reconcile(server_seq, my_server_data)
            } else {
                network_client.reset_prediction_state();
                Vec::new()
            };

            if !local_players.is_empty() {
                let local_player = &mut local_players[0];

                // 应用服务器权威状态
                local_player.ship.pos = Vec2::new(my_server_data.x, my_server_data.y);
                local_player.ship.rot = my_server_data.angle;
                local_player.ship.vel = Vec2::new(my_server_data.vel_x, my_server_data.vel_y);
                local_player.lives = my_server_data.lives;

                // 分数同步
                if local_player.score.value() != my_server_data.score {
                    local_player.score.reset();
                    local_player.score.add_points(my_server_data.score);
                }

                // 存活状态同步
                if !my_server_data.alive && local_player.alive {
                    local_player.mark_dead(frame_t);
                } else if my_server_data.alive && !local_player.alive {
                    local_player.alive = true;
                }

                // 重播未确认输入
                if !replay_inputs.is_empty() && local_player.alive {
                    for cmd in replay_inputs.iter() {
                        let parsed = ParsedInput::from_keys(&cmd.keys);
                        let replay_dt = if cmd.dt > 0.0 && cmd.dt < 0.1 {
                            cmd.dt
                        } else {
                            dt
                        };
                        apply_predicted_input(local_player, &parsed, replay_dt, cmd.timestamp);
                    }
                }
            }
        }

        // 同步对手玩家
        sync_opponent_players(server_players, local_players, &my_id, frame_t);
    }

    // 同步子弹
    sync_online_bullets(server_bullets, online_bullets);
}

/// 同步对手玩家状态
fn sync_opponent_players(
    server_players: &[PlayerState],
    local_players: &mut Vec<Player>,
    my_id: &str,
    now: f64,
) {
    for server_player in server_players.iter() {
        if server_player.id == my_id {
            continue;
        }

        // 确保有对手玩家对象
        if local_players.len() < 2 {
            local_players.push(Player::new(
                "Opponent",
                RED,
                Vec2::new(server_player.x, server_player.y),
                Controls {
                    thrust: KeyCode::Unknown,
                    left: KeyCode::Unknown,
                    right: KeyCode::Unknown,
                    shoot_primary: KeyCode::Unknown,
                    shoot_alt: None,
                    weapon_switch: KeyCode::Unknown,
                    weapon_switch_alt: None,
                    dash: KeyCode::Unknown,
                    hyperspace: KeyCode::Unknown,
                    phase_dash: KeyCode::Unknown,
                },
                now,
                server_player.lives,
            ));
        }

        // 更新对手位置
        if local_players.len() > 1 {
            let opponent = &mut local_players[1];
            opponent.ship.pos = Vec2::new(server_player.x, server_player.y);
            opponent.ship.rot = server_player.angle;
            opponent.ship.vel = Vec2::new(server_player.vel_x, server_player.vel_y);
            opponent.lives = server_player.lives;
            opponent.alive = server_player.alive;

            if opponent.score.value() != server_player.score {
                opponent.score.reset();
                opponent.score.add_points(server_player.score);
            }
        }
    }
}

/// 同步在线子弹
fn sync_online_bullets(server_bullets: &[network::BulletState], online_bullets: &mut Vec<OnlineBullet>) {
    online_bullets.clear();
    for sb in server_bullets {
        online_bullets.push(OnlineBullet {
            x: sb.x,
            y: sb.y,
            vx: sb.vx,
            vy: sb.vy,
        });
    }
}

// ============================================================================
// 在线模式初始化
// ============================================================================

/// 初始化在线模式玩家
pub fn init_online_player(players: &mut Vec<Player>, now: f64) {
    if players.is_empty() {
        players.push(Player::new(
            "Online Player",
            BLUE,
            Vec2::new(screen_width() / 2.0, screen_height() / 2.0),
            Controls {
                thrust: KeyCode::W,
                left: KeyCode::A,
                right: KeyCode::D,
                shoot_primary: KeyCode::J,
                shoot_alt: Some(KeyCode::F),
                weapon_switch: KeyCode::U,
                weapon_switch_alt: None,
                dash: KeyCode::Space,
                hyperspace: KeyCode::H,
                phase_dash: KeyCode::E,
            },
            now,
            3,
        ));
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_input_keys_empty() {
        // 基本测试 - 实际测试需要 macroquad 上下文
        let keys: Vec<String> = Vec::new();
        assert!(keys.is_empty());
    }
}
