//! 新手引导系统
//!
//! 提供分层的引导体验：
//! - 即时提示：首次进入某界面/阶段时显示操作提示（自动消失）
//! - 目标提示：持续显示当前目标
//! - 内置帮助：暂停菜单中的详细说明（未来扩展）
//!
//! 注意：部分功能预留用于未来扩展

#![allow(dead_code)]

use std::collections::HashSet;

use macroquad::prelude::*;
use macroquad::text::Font;

// ============================================================================
// 引导屏幕和步骤定义
// ============================================================================

/// 游戏屏幕/阶段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TutorialScreen {
    /// 普通游戏（Survival/TimeAttack/Duel）
    Gameplay,
    /// Roguelike 战斗阶段
    RoguelikeCombat,
    /// Roguelike 奖励选择
    RoguelikeReward,
    /// Roguelike 商店
    RoguelikeShop,
    /// Roguelike Boss 战
    RoguelikeBoss,
    /// Roguelike 休息阶段
    RoguelikeRest,
}

/// 引导步骤（用于追踪玩家是否完成特定操作）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TutorialStep {
    /// 移动和射击
    MoveAndShoot,
    /// 清空波次
    ClearWave,
    /// 选择奖励
    ChooseReward,
    /// 商店购买
    ShopBuy,
    /// 商店刷新
    ShopRefresh,
    /// 使用相位闪现
    UsePhaseDash,
    /// 击败 Boss
    DefeatBoss,
}

// ============================================================================
// 引导状态
// ============================================================================

/// 新手引导状态管理器
pub struct TutorialState {
    /// 上一个屏幕（用于检测屏幕切换）
    last_screen: Option<TutorialScreen>,
    /// 已看过的屏幕（不再显示首次提示）
    seen_screens: HashSet<TutorialScreen>,
    /// 已完成的步骤
    completed_steps: HashSet<TutorialStep>,
    /// 当前显示的 toast 提示 (文本, 消失时间)
    toast: Option<(String, f64)>,
    /// 当前目标提示
    current_objective: Option<&'static str>,
    /// 是否启用引导（可在设置中关闭）
    pub enabled: bool,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self::new()
    }
}

impl TutorialState {
    /// 创建新的引导状态
    pub fn new() -> Self {
        Self {
            last_screen: None,
            seen_screens: HashSet::new(),
            completed_steps: HashSet::new(),
            toast: None,
            current_objective: None,
            enabled: true,
        }
    }

    /// 设置当前屏幕（触发首次进入提示）
    pub fn set_screen(&mut self, screen: TutorialScreen, now: f64) {
        if !self.enabled {
            return;
        }

        let changed = self.last_screen != Some(screen);
        self.last_screen = Some(screen);

        // 更新当前目标
        self.current_objective = Some(match screen {
            TutorialScreen::Gameplay => "目标：存活并清空小行星",
            TutorialScreen::RoguelikeCombat => "目标：清空本波敌人",
            TutorialScreen::RoguelikeReward => "目标：选择一个奖励",
            TutorialScreen::RoguelikeShop => "目标：购买装备或继续",
            TutorialScreen::RoguelikeBoss => "目标：击败 Boss",
            TutorialScreen::RoguelikeRest => "目标：选择休息选项",
        });

        // 首次进入时显示操作提示
        if changed && !self.seen_screens.contains(&screen) {
            self.seen_screens.insert(screen);
            let (text, duration) = match screen {
                TutorialScreen::Gameplay => (
                    "操作：W/↑ 推进 · A/D/←/→ 转向 · J/Space 射击 · K/F 闪现 · Esc 暂停",
                    7.0,
                ),
                TutorialScreen::RoguelikeCombat => {
                    ("Roguelike：清空波次后选择奖励，然后前往商店", 6.0)
                }
                TutorialScreen::RoguelikeReward => ("奖励选择：按 1/2/3 或点击卡片选择", 5.0),
                TutorialScreen::RoguelikeShop => ("商店：点击商品购买 · R 刷新 · Enter 继续", 5.0),
                TutorialScreen::RoguelikeBoss => {
                    ("Boss 战：小心 Boss 的攻击模式，狂暴时更危险！", 5.0)
                }
                TutorialScreen::RoguelikeRest => {
                    ("休息站：恢复生命、升级卡牌或移除卡牌", 5.0)
                }
            };
            self.toast = Some((text.to_string(), now + duration));
        }
    }

    /// 标记步骤完成
    pub fn mark_step_done(&mut self, step: TutorialStep) {
        self.completed_steps.insert(step);
    }

    /// 检查步骤是否完成
    #[allow(dead_code)]
    pub fn is_step_done(&self, step: TutorialStep) -> bool {
        self.completed_steps.contains(&step)
    }

    /// 显示自定义 toast 提示
    pub fn show_toast(&mut self, text: &str, duration: f64, now: f64) {
        if self.enabled {
            self.toast = Some((text.to_string(), now + duration));
        }
    }

    /// 清除当前目标（用于非游戏状态）
    pub fn clear_objective(&mut self) {
        self.current_objective = None;
        self.last_screen = None;
    }

    /// 绘制引导 UI
    pub fn draw(&mut self, now: f64, font: Option<&Font>) {
        if !self.enabled {
            return;
        }

        // 绘制目标提示（屏幕顶部中央）
        if let Some(objective) = self.current_objective {
            self.draw_objective(objective, font);
        }

        // 绘制 toast 提示（屏幕底部中央）
        if let Some((ref text, hide_at)) = self.toast.clone() {
            if now >= hide_at {
                self.toast = None;
            } else {
                let alpha = if hide_at - now < 1.0 {
                    (hide_at - now) as f32
                } else {
                    1.0
                };
                self.draw_toast(text, alpha, font);
            }
        }
    }

    /// 绘制目标提示
    fn draw_objective(&self, text: &str, font: Option<&Font>) {
        let padding = 14.0;
        let font_size = 20u16;
        let text_width = measure_text(text, font, font_size, 1.0).width;
        let box_width = text_width + padding * 2.0;
        let box_height = 32.0;
        let x = screen_width() / 2.0 - box_width / 2.0;
        let y = 12.0;

        // 背景
        draw_rectangle(x, y, box_width, box_height, Color::new(0.0, 0.0, 0.0, 0.5));
        draw_rectangle_lines(
            x,
            y,
            box_width,
            box_height,
            1.5,
            Color::new(0.4, 0.6, 0.9, 0.7),
        );

        // 文字
        draw_text_ex(
            text,
            x + padding,
            y + 22.0,
            TextParams {
                font,
                font_size,
                color: Color::new(0.85, 0.9, 0.98, 1.0),
                ..Default::default()
            },
        );
    }

    /// 绘制 toast 提示
    fn draw_toast(&self, text: &str, alpha: f32, font: Option<&Font>) {
        let padding = 16.0;
        let font_size = 18u16;
        let text_width = measure_text(text, font, font_size, 1.0).width;
        let box_width = text_width + padding * 2.0;
        let box_height = 36.0;
        let x = screen_width() / 2.0 - box_width / 2.0;
        let y = screen_height() - 65.0;

        // 背景
        draw_rectangle(
            x,
            y,
            box_width,
            box_height,
            Color::new(0.0, 0.0, 0.0, 0.6 * alpha),
        );
        draw_rectangle_lines(
            x,
            y,
            box_width,
            box_height,
            1.5,
            Color::new(0.3, 0.55, 0.9, 0.75 * alpha),
        );

        // 文字
        draw_text_ex(
            text,
            x + padding,
            y + 24.0,
            TextParams {
                font,
                font_size,
                color: Color::new(0.9, 0.92, 0.98, alpha),
                ..Default::default()
            },
        );
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tutorial_state_creation() {
        let state = TutorialState::new();
        assert!(state.enabled);
        assert!(state.seen_screens.is_empty());
        assert!(state.completed_steps.is_empty());
        assert!(state.toast.is_none());
        assert!(state.current_objective.is_none());
    }

    #[test]
    fn test_set_screen_updates_objective() {
        let mut state = TutorialState::new();
        state.set_screen(TutorialScreen::Gameplay, 0.0);
        assert!(state.current_objective.is_some());
        assert!(state.seen_screens.contains(&TutorialScreen::Gameplay));
    }

    #[test]
    fn test_mark_step_done() {
        let mut state = TutorialState::new();
        assert!(!state.is_step_done(TutorialStep::MoveAndShoot));
        state.mark_step_done(TutorialStep::MoveAndShoot);
        assert!(state.is_step_done(TutorialStep::MoveAndShoot));
    }

    #[test]
    fn test_disabled_state() {
        let mut state = TutorialState::new();
        state.enabled = false;
        state.set_screen(TutorialScreen::Gameplay, 0.0);
        // 禁用时不应更新状态
        assert!(state.current_objective.is_none());
        assert!(state.seen_screens.is_empty());
    }
}
