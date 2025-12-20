//! 成就系统模块
//!
//! 提供情绪价值的成就追踪系统，包含38个成就。
//!
//! ## 功能
//! - 成就定义和分类（新手、连击、生存、对战、完美、探索、累计）
//! - 进度追踪和持久化存储
//! - 成就解锁检测
//! - 数据保存/加载（JSON格式）
//! - 重置功能

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::storage;

/// 成就唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AchievementId {
    // 新手村系列 (7个)
    FirstFlight,
    FirstBlood,
    Marksman,
    Protected,
    Armed,
    QuickStart,
    Lucky,

    // 连击大师系列 (6个)
    DoubleTrouble,
    TripleThreat,
    MegaKiller,
    Unstoppable,
    Godlike,
    ComboMaster,

    // 生存模式专精 (8个)
    Survivor,
    TimeWarrior,
    Endurance,
    WaveRider,
    WaveMaster,
    WaveGod,
    Century,
    Champion,

    // 对战模式荣耀 (5个)
    Warrior,
    Duelist,
    FlagHunter,
    Assassin,
    Dominator,

    // 完美主义系列 (4个)
    FlawlessVictory,
    Sharpshooter,
    ShieldMaster,
    Deadeye,

    // 探索与实验 (3个)
    Arsenal,
    Adventurer,
    Tinkerer,

    // 累计成就 (2个)
    Veteran,
    Legend,

    // 隐藏成就 (3个)
    ThePacifist,
    LuckySeven,
    MidnightWarrior,

    // UFO 猎手系列 (3个)
    FirstContact, // 首次击毁 UFO
    SkyHunter,    // 累计击毁 10 个 UFO
    CleanSweep,   // 无伤击毁 UFO
}

impl AchievementId {
    /// 获取所有成就ID
    pub fn all() -> Vec<Self> {
        vec![
            // 新手村
            Self::FirstFlight,
            Self::FirstBlood,
            Self::Marksman,
            Self::Protected,
            Self::Armed,
            Self::QuickStart,
            Self::Lucky,
            // 连击大师
            Self::DoubleTrouble,
            Self::TripleThreat,
            Self::MegaKiller,
            Self::Unstoppable,
            Self::Godlike,
            Self::ComboMaster,
            // 生存模式
            Self::Survivor,
            Self::TimeWarrior,
            Self::Endurance,
            Self::WaveRider,
            Self::WaveMaster,
            Self::WaveGod,
            Self::Century,
            Self::Champion,
            // 对战模式
            Self::Warrior,
            Self::Duelist,
            Self::FlagHunter,
            Self::Assassin,
            Self::Dominator,
            // 完美主义
            Self::FlawlessVictory,
            Self::Sharpshooter,
            Self::ShieldMaster,
            Self::Deadeye,
            // 探索
            Self::Arsenal,
            Self::Adventurer,
            Self::Tinkerer,
            // 累计
            Self::Veteran,
            Self::Legend,
            // 隐藏
            Self::ThePacifist,
            Self::LuckySeven,
            Self::MidnightWarrior,
            // UFO 猎手
            Self::FirstContact,
            Self::SkyHunter,
            Self::CleanSweep,
        ]
    }
}

/// 成就等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AchievementTier {
    Bronze,  // 🥉 青铜（新手友好）
    Silver,  // 🥈 银牌（需要技巧）
    Gold,    // 🥇 金牌（有挑战性）
    Diamond, // 💎 钻石（顶级成就）
}

impl AchievementTier {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Bronze => "[B]",
            Self::Silver => "[S]",
            Self::Gold => "[G]",
            Self::Diamond => "[D]",
        }
    }

    pub fn color(&self) -> macroquad::color::Color {
        use macroquad::color::Color;
        match self {
            Self::Bronze => Color::new(0.8, 0.5, 0.2, 1.0),
            Self::Silver => Color::new(0.75, 0.75, 0.75, 1.0),
            Self::Gold => Color::new(1.0, 0.84, 0.0, 1.0),
            Self::Diamond => Color::new(0.7, 0.9, 1.0, 1.0),
        }
    }
}

/// 成就分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AchievementCategory {
    Beginner,      // 新手村
    Combo,         // 连击大师
    Survival,      // 生存模式
    Duel,          // 对战模式
    Perfectionist, // 完美主义
    Explorer,      // 探索实验
    Veteran,       // 累计成就
    Hidden,        // 隐藏成就
}

impl AchievementCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Beginner => "新手村",
            Self::Combo => "连击大师",
            Self::Survival => "生存模式专精",
            Self::Duel => "对战模式荣耀",
            Self::Perfectionist => "完美主义",
            Self::Explorer => "探索与实验",
            Self::Veteran => "累计成就",
            Self::Hidden => "隐藏成就",
        }
    }
}

/// 成就定义
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Achievement {
    pub id: AchievementId,
    pub name: &'static str,
    pub description: &'static str,
    pub quote: &'static str, // 鼓励性文案
    pub icon: &'static str,  // emoji图标
    pub tier: AchievementTier,
    pub category: AchievementCategory,
    pub hidden: bool,
    pub target: u32, // 目标进度（0表示一次性成就）
}

impl Achievement {
    /// 获取成就定义
    pub fn get(id: AchievementId) -> Self {
        match id {
            // ========== 新手村系列 ==========
            AchievementId::FirstFlight => Self {
                id,
                name: "First Flight",
                description: "完成第一次游戏",
                quote: "每个传奇都有开始！",
                icon: "*",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Beginner,
                hidden: false,
                target: 1,
            },
            AchievementId::FirstBlood => Self {
                id,
                name: "First Blood",
                description: "摧毁第一颗小行星",
                quote: "砰！就这么简单！",
                icon: "!",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Beginner,
                hidden: false,
                target: 1,
            },
            AchievementId::Marksman => Self {
                id,
                name: "Marksman",
                description: "击中10颗小行星",
                quote: "你的瞄准越来越准了！",
                icon: "o",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Beginner,
                hidden: false,
                target: 10,
            },
            AchievementId::Protected => Self {
                id,
                name: "Protected",
                description: "拾取第一个护盾道具",
                quote: "安全第一！",
                icon: "+",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Beginner,
                hidden: false,
                target: 1,
            },
            AchievementId::Armed => Self {
                id,
                name: "Armed",
                description: "发射100发子弹",
                quote: "弹药充足！",
                icon: ">",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Beginner,
                hidden: false,
                target: 100,
            },
            AchievementId::QuickStart => Self {
                id,
                name: "Quick Start",
                description: "在30秒内摧毁5颗小行星",
                quote: "速度与激情！",
                icon: "@",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Beginner,
                hidden: false,
                target: 0, // 条件型成就
            },
            AchievementId::Lucky => Self {
                id,
                name: "Lucky",
                description: "在无敌时间内躲避一次碰撞",
                quote: "运气也是实力的一部分！",
                icon: "~",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Beginner,
                hidden: false,
                target: 1,
            },

            // ========== 连击大师系列 ==========
            AchievementId::DoubleTrouble => Self {
                id,
                name: "Double Trouble",
                description: "达到2连击",
                quote: "不错的开始！",
                icon: "x2",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Combo,
                hidden: false,
                target: 2,
            },
            AchievementId::TripleThreat => Self {
                id,
                name: "Triple Threat",
                description: "达到3连击",
                quote: "你开始掌握节奏了！",
                icon: "x3",
                tier: AchievementTier::Silver,
                category: AchievementCategory::Combo,
                hidden: false,
                target: 3,
            },
            AchievementId::MegaKiller => Self {
                id,
                name: "Mega Killer",
                description: "达到5连击",
                quote: "势不可挡！",
                icon: "x5",
                tier: AchievementTier::Silver,
                category: AchievementCategory::Combo,
                hidden: false,
                target: 5,
            },
            AchievementId::Unstoppable => Self {
                id,
                name: "Unstoppable",
                description: "达到10连击",
                quote: "无人能挡！",
                icon: "!!",
                tier: AchievementTier::Gold,
                category: AchievementCategory::Combo,
                hidden: false,
                target: 10,
            },
            AchievementId::Godlike => Self {
                id,
                name: "Godlike",
                description: "达到15连击",
                quote: "你就是神！",
                icon: "##",
                tier: AchievementTier::Diamond,
                category: AchievementCategory::Combo,
                hidden: false,
                target: 15,
            },
            AchievementId::ComboMaster => Self {
                id,
                name: "Combo Master",
                description: "在一局游戏中达到3次以上5连击",
                quote: "连击专家！",
                icon: "**",
                tier: AchievementTier::Gold,
                category: AchievementCategory::Combo,
                hidden: false,
                target: 0, // 条件型
            },

            // ========== 生存模式专精 ==========
            AchievementId::Survivor => Self {
                id,
                name: "Survivor",
                description: "在生存模式存活超过60秒",
                quote: "活下去才是硬道理！",
                icon: "^",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Survival,
                hidden: false,
                target: 0,
            },
            AchievementId::TimeWarrior => Self {
                id,
                name: "Time Warrior",
                description: "在生存模式存活超过3分钟",
                quote: "时间的主宰！",
                icon: "&",
                tier: AchievementTier::Silver,
                category: AchievementCategory::Survival,
                hidden: false,
                target: 0,
            },
            AchievementId::Endurance => Self {
                id,
                name: "Endurance",
                description: "在生存模式存活超过5分钟",
                quote: "超凡的耐力！",
                icon: "@",
                tier: AchievementTier::Gold,
                category: AchievementCategory::Survival,
                hidden: false,
                target: 0,
            },
            AchievementId::WaveRider => Self {
                id,
                name: "Wave Rider",
                description: "通过第3波小行星",
                quote: "乘风破浪！",
                icon: "~3",
                tier: AchievementTier::Silver,
                category: AchievementCategory::Survival,
                hidden: false,
                target: 3,
            },
            AchievementId::WaveMaster => Self {
                id,
                name: "Wave Master",
                description: "通过第5波小行星",
                quote: "波次大师！",
                icon: "~5",
                tier: AchievementTier::Gold,
                category: AchievementCategory::Survival,
                hidden: false,
                target: 5,
            },
            AchievementId::WaveGod => Self {
                id,
                name: "Wave God",
                description: "通过第10波小行星",
                quote: "小行星都在颤抖！",
                icon: "@@",
                tier: AchievementTier::Diamond,
                category: AchievementCategory::Survival,
                hidden: false,
                target: 10,
            },
            AchievementId::Century => Self {
                id,
                name: "Century",
                description: "在生存模式获得1000分",
                quote: "百分百的努力！",
                icon: "100",
                tier: AchievementTier::Silver,
                category: AchievementCategory::Survival,
                hidden: false,
                target: 1000,
            },
            AchievementId::Champion => Self {
                id,
                name: "Champion",
                description: "在生存模式获得5000分",
                quote: "你是冠军！",
                icon: "#1",
                tier: AchievementTier::Diamond,
                category: AchievementCategory::Survival,
                hidden: false,
                target: 5000,
            },

            // ========== 对战模式荣耀 ==========
            AchievementId::Warrior => Self {
                id,
                name: "Warrior",
                description: "完成第一场对战",
                quote: "战斗的开始！",
                icon: "|",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Duel,
                hidden: false,
                target: 1,
            },
            AchievementId::Duelist => Self {
                id,
                name: "Duelist",
                description: "在对战模式赢得5场胜利",
                quote: "决斗高手！",
                icon: "[]",
                tier: AchievementTier::Silver,
                category: AchievementCategory::Duel,
                hidden: false,
                target: 5,
            },
            AchievementId::FlagHunter => Self {
                id,
                name: "Flag Hunter",
                description: "成功夺取旗帜10次",
                quote: "旗帜猎人！",
                icon: ">>",
                tier: AchievementTier::Silver,
                category: AchievementCategory::Duel,
                hidden: false,
                target: 10,
            },
            AchievementId::Assassin => Self {
                id,
                name: "Assassin",
                description: "在对战中击杀对手100次",
                quote: "致命刺客！",
                icon: "X",
                tier: AchievementTier::Gold,
                category: AchievementCategory::Duel,
                hidden: false,
                target: 100,
            },
            AchievementId::Dominator => Self {
                id,
                name: "Dominator",
                description: "连续赢得3场对战",
                quote: "主宰战场！",
                icon: "<>",
                tier: AchievementTier::Gold,
                category: AchievementCategory::Duel,
                hidden: false,
                target: 0, // 条件型
            },

            // ========== 完美主义系列 ==========
            AchievementId::FlawlessVictory => Self {
                id,
                name: "Flawless Victory",
                description: "在一局中不损失生命清除一波",
                quote: "完美无瑕！",
                icon: "<D>",
                tier: AchievementTier::Gold,
                category: AchievementCategory::Perfectionist,
                hidden: false,
                target: 0,
            },
            AchievementId::Sharpshooter => Self {
                id,
                name: "Sharpshooter",
                description: "命中率达到80%（一局内发射20发以上）",
                quote: "神枪手！",
                icon: "(o)",
                tier: AchievementTier::Gold,
                category: AchievementCategory::Perfectionist,
                hidden: false,
                target: 0,
            },
            AchievementId::ShieldMaster => Self {
                id,
                name: "Shield Master",
                description: "拾取20个护盾道具",
                quote: "护盾专家！",
                icon: "[+]",
                tier: AchievementTier::Silver,
                category: AchievementCategory::Perfectionist,
                hidden: false,
                target: 20,
            },
            AchievementId::Deadeye => Self {
                id,
                name: "Deadeye",
                description: "摧毁500颗小行星",
                quote: "死神之眼！",
                icon: "***",
                tier: AchievementTier::Diamond,
                category: AchievementCategory::Perfectionist,
                hidden: false,
                target: 500,
            },

            // ========== 探索与实验 ==========
            AchievementId::Arsenal => Self {
                id,
                name: "Arsenal",
                description: "使用所有三种武器类型",
                quote: "武器大师！",
                icon: "=/=",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Explorer,
                hidden: false,
                target: 3,
            },
            AchievementId::Adventurer => Self {
                id,
                name: "Adventurer",
                description: "尝试所有游戏模式",
                quote: "冒险家精神！",
                icon: "?!",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Explorer,
                hidden: false,
                target: 2, // 生存和对战
            },
            AchievementId::Tinkerer => Self {
                id,
                name: "Tinkerer",
                description: "修改过5次游戏设置",
                quote: "喜欢折腾的人！",
                icon: "{*}",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Explorer,
                hidden: false,
                target: 5,
            },

            // ========== 累计成就 ==========
            AchievementId::Veteran => Self {
                id,
                name: "Veteran",
                description: "总游戏时长达到30分钟",
                quote: "老兵不死！",
                icon: "[V]",
                tier: AchievementTier::Silver,
                category: AchievementCategory::Veteran,
                hidden: false,
                target: 1800, // 秒
            },
            AchievementId::Legend => Self {
                id,
                name: "Legend",
                description: "总游戏时长达到2小时",
                quote: "你已成为传奇！",
                icon: "[L]",
                tier: AchievementTier::Diamond,
                category: AchievementCategory::Veteran,
                hidden: false,
                target: 7200, // 秒
            },

            // ========== 隐藏成就 ==========
            AchievementId::ThePacifist => Self {
                id,
                name: "The Pacifist",
                description: "在一局中存活60秒但不发射任何子弹",
                quote: "和平主义者！",
                icon: "[-]",
                tier: AchievementTier::Gold,
                category: AchievementCategory::Hidden,
                hidden: true,
                target: 0,
            },
            AchievementId::LuckySeven => Self {
                id,
                name: "Lucky Seven",
                description: "在7秒内摧毁7颗小行星",
                quote: "幸运数字7！",
                icon: "=7=",
                tier: AchievementTier::Gold,
                category: AchievementCategory::Hidden,
                hidden: true,
                target: 0,
            },
            AchievementId::MidnightWarrior => Self {
                id,
                name: "Midnight Warrior",
                description: "在系统时间午夜(00:00-01:00)游玩",
                quote: "夜猫子！",
                icon: "(*))",
                tier: AchievementTier::Silver,
                category: AchievementCategory::Hidden,
                hidden: true,
                target: 0,
            },

            // ========== UFO 猎手系列 ==========
            AchievementId::FirstContact => Self {
                id,
                name: "First Contact",
                description: "首次击毁一艘 UFO",
                quote: "欢迎来到外星空域！",
                icon: "^",
                tier: AchievementTier::Bronze,
                category: AchievementCategory::Explorer,
                hidden: false,
                target: 1,
            },
            AchievementId::SkyHunter => Self {
                id,
                name: "Sky Hunter",
                description: "累计击毁 10 艘 UFO",
                quote: "没有飞碟能逃过你的瞄准。",
                icon: "@",
                tier: AchievementTier::Silver,
                category: AchievementCategory::Veteran,
                hidden: false,
                target: 10,
            },
            AchievementId::CleanSweep => Self {
                id,
                name: "Clean Sweep",
                description: "在未受伤的情况下击毁一艘 UFO",
                quote: "一尘不染的胜利。",
                icon: "#",
                tier: AchievementTier::Gold,
                category: AchievementCategory::Perfectionist,
                hidden: false,
                target: 1,
            },
        }
    }
}

/// 成就进度
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AchievementProgress {
    pub unlocked: bool,
    pub unlock_time: Option<f64>,
    pub current: u32,
}

/// 玩家统计数据（累计追踪）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStats {
    pub total_playtime: f64, // 总游戏时长（秒）
    pub total_kills: u32,    // 总击杀数
    #[serde(default)]
    pub ufo_kills_total: u32, // 累计击毁 UFO 数
    pub bullets_fired: u32,  // 发射的子弹总数
    pub shields_collected: u32, // 拾取的护盾数
    pub games_played: u32,   // 游戏局数
    pub survival_games: u32, // 生存模式局数
    pub duel_games: u32,     // 对战模式局数
    pub duel_wins: u32,      // 对战胜利数
    pub modes_played: HashSet<String>, // 已玩过的模式
    pub weapons_used: HashSet<String>, // 已使用的武器
    pub settings_changed: u32, // 设置修改次数
    pub max_wave: u32,       // 最高波次
    pub max_killstreak: u32, // 最高连击
    pub five_streaks: u32,   // 5连击次数
}

/// 成就管理器
pub struct AchievementManager {
    progress: HashMap<AchievementId, AchievementProgress>,
    recently_unlocked: Vec<(AchievementId, f64)>, // 最近解锁的成就（用于显示动画）
    pub stats: PlayerStats,                       // 玩家统计
}

/// 保存数据结构
#[derive(Serialize, Deserialize)]
struct SaveData {
    progress: HashMap<String, AchievementProgress>,
    stats: PlayerStats,
}

impl AchievementManager {
    /// 创建成就管理器
    pub fn new() -> Self {
        let mut manager = Self {
            progress: HashMap::new(),
            recently_unlocked: Vec::new(),
            stats: PlayerStats::default(),
        };

        // 初始化所有成就
        for id in AchievementId::all() {
            manager.progress.insert(id, AchievementProgress::default());
        }

        // 尝试加载保存的数据
        manager.load();
        manager
    }

    /// 检查并解锁成就
    pub fn unlock(&mut self, id: AchievementId, time: f64) -> bool {
        if let Some(progress) = self.progress.get_mut(&id)
            && !progress.unlocked
        {
            progress.unlocked = true;
            progress.unlock_time = Some(time);
            self.recently_unlocked.push((id, time));
            self.save();
            return true;
        }
        false
    }

    /// 更新进度
    pub fn update_progress(&mut self, id: AchievementId, value: u32, time: f64) {
        let achievement = Achievement::get(id);
        if let Some(progress) = self.progress.get_mut(&id)
            && !progress.unlocked
        {
            progress.current = value;
            // 检查是否达到目标
            if achievement.target > 0 && progress.current >= achievement.target {
                self.unlock(id, time);
            }
        }
    }

    /// 增加进度
    #[allow(dead_code)]
    pub fn increment_progress(&mut self, id: AchievementId, delta: u32, time: f64) {
        if let Some(progress) = self.progress.get(&id) {
            let new_value = progress.current + delta;
            self.update_progress(id, new_value, time);
        }
    }

    /// 获取进度
    pub fn get_progress(&self, id: AchievementId) -> Option<&AchievementProgress> {
        self.progress.get(&id)
    }

    /// 是否已解锁
    #[allow(dead_code)]
    pub fn is_unlocked(&self, id: AchievementId) -> bool {
        self.progress.get(&id).map(|p| p.unlocked).unwrap_or(false)
    }

    /// 获取最近解锁的成就（用于显示）
    pub fn get_recent_unlocks(&self, max_age: f64, current_time: f64) -> Vec<AchievementId> {
        self.recently_unlocked
            .iter()
            .filter(|(_, time)| current_time - time < max_age)
            .map(|(id, _)| *id)
            .collect()
    }

    /// 清理旧的解锁记录
    pub fn cleanup_recent_unlocks(&mut self, max_age: f64, current_time: f64) {
        self.recently_unlocked
            .retain(|(_, time)| current_time - time < max_age);
    }

    /// 获取解锁统计
    pub fn get_stats(&self) -> (usize, usize) {
        let total = self.progress.len();
        let unlocked = self.progress.values().filter(|p| p.unlocked).count();
        (unlocked, total)
    }

    /// 按分类获取成就列表
    pub fn get_by_category(&self, category: AchievementCategory) -> Vec<AchievementId> {
        AchievementId::all()
            .into_iter()
            .filter(|id| Achievement::get(*id).category == category)
            .collect()
    }

    /// 保存到文件
    pub fn save(&self) {
        // 转换progress的key为String（因为enum不能直接序列化为JSON key）
        let progress_map: HashMap<String, AchievementProgress> = self
            .progress
            .iter()
            .map(|(id, prog)| (format!("{:?}", id), prog.clone()))
            .collect();

        let save_data = SaveData {
            progress: progress_map,
            stats: self.stats.clone(),
        };

        if let Ok(json) = serde_json::to_string(&save_data)
            && let Err(e) = storage::save("achievements", &json)
        {
            eprintln!("Failed to save achievements: {}", e);
        }
    }

    /// 从文件加载
    pub fn load(&mut self) {
        if let Ok(data) = storage::load("achievements")
            && let Ok(save_data) = serde_json::from_str::<SaveData>(&data)
        {
            // 加载统计数据
            self.stats = save_data.stats;

            // 加载成就进度
            for (id_str, progress) in save_data.progress {
                if let Some(id) = self.parse_achievement_id(&id_str)
                    && let Some(existing_progress) = self.progress.get_mut(&id)
                {
                    *existing_progress = progress;
                }
            }
        }
    }

    /// 解析成就ID字符串
    fn parse_achievement_id(&self, s: &str) -> Option<AchievementId> {
        match s {
            "FirstFlight" => Some(AchievementId::FirstFlight),
            "FirstBlood" => Some(AchievementId::FirstBlood),
            "Marksman" => Some(AchievementId::Marksman),
            "Protected" => Some(AchievementId::Protected),
            "Armed" => Some(AchievementId::Armed),
            "QuickStart" => Some(AchievementId::QuickStart),
            "Lucky" => Some(AchievementId::Lucky),
            "DoubleTrouble" => Some(AchievementId::DoubleTrouble),
            "TripleThreat" => Some(AchievementId::TripleThreat),
            "MegaKiller" => Some(AchievementId::MegaKiller),
            "Unstoppable" => Some(AchievementId::Unstoppable),
            "Godlike" => Some(AchievementId::Godlike),
            "ComboMaster" => Some(AchievementId::ComboMaster),
            "Survivor" => Some(AchievementId::Survivor),
            "TimeWarrior" => Some(AchievementId::TimeWarrior),
            "Endurance" => Some(AchievementId::Endurance),
            "WaveRider" => Some(AchievementId::WaveRider),
            "WaveMaster" => Some(AchievementId::WaveMaster),
            "WaveGod" => Some(AchievementId::WaveGod),
            "Century" => Some(AchievementId::Century),
            "Champion" => Some(AchievementId::Champion),
            "Warrior" => Some(AchievementId::Warrior),
            "Duelist" => Some(AchievementId::Duelist),
            "FlagHunter" => Some(AchievementId::FlagHunter),
            "Assassin" => Some(AchievementId::Assassin),
            "Dominator" => Some(AchievementId::Dominator),
            "FlawlessVictory" => Some(AchievementId::FlawlessVictory),
            "Sharpshooter" => Some(AchievementId::Sharpshooter),
            "ShieldMaster" => Some(AchievementId::ShieldMaster),
            "Deadeye" => Some(AchievementId::Deadeye),
            "Arsenal" => Some(AchievementId::Arsenal),
            "Adventurer" => Some(AchievementId::Adventurer),
            "Tinkerer" => Some(AchievementId::Tinkerer),
            "Veteran" => Some(AchievementId::Veteran),
            "Legend" => Some(AchievementId::Legend),
            "ThePacifist" => Some(AchievementId::ThePacifist),
            "LuckySeven" => Some(AchievementId::LuckySeven),
            "MidnightWarrior" => Some(AchievementId::MidnightWarrior),
            "FirstContact" => Some(AchievementId::FirstContact),
            "SkyHunter" => Some(AchievementId::SkyHunter),
            "CleanSweep" => Some(AchievementId::CleanSweep),
            _ => None,
        }
    }

    /// 从字符串中提取数字
    #[allow(dead_code)]
    fn extract_number(&self, s: &str, key: &str) -> u32 {
        if let Some(pos) = s.find(key) {
            let after = &s[pos + key.len()..];
            let num_str: String = after
                .chars()
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            num_str.parse().unwrap_or(0)
        } else {
            0
        }
    }

    /// 从字符串中提取浮点数
    #[allow(dead_code)]
    fn extract_number_f64(&self, s: &str, key: &str) -> f64 {
        if let Some(pos) = s.find(key) {
            let after = &s[pos + key.len()..];
            let num_str: String = after
                .chars()
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            num_str.parse().unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// 重置所有成就
    pub fn reset(&mut self) {
        for progress in self.progress.values_mut() {
            *progress = AchievementProgress::default();
        }
        self.stats = PlayerStats::default(); // 重置统计数据
        self.recently_unlocked.clear();
        self.save();
    }

    /// 创建不加载存储的管理器（仅用于测试）
    #[cfg(test)]
    fn new_without_load() -> Self {
        let mut manager = Self {
            progress: HashMap::new(),
            recently_unlocked: Vec::new(),
            stats: PlayerStats::default(),
        };
        for id in AchievementId::all() {
            manager.progress.insert(id, AchievementProgress::default());
        }
        manager
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // AchievementId Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_achievement_id_all_returns_correct_count() {
        let all = AchievementId::all();
        // 7 新手 + 6 连击 + 8 生存 + 5 对战 + 4 完美 + 3 探索 + 2 累计 + 3 隐藏 + 3 UFO = 41
        assert_eq!(all.len(), 41);
    }

    #[test]
    fn test_achievement_id_all_no_duplicates() {
        let all = AchievementId::all();
        let set: std::collections::HashSet<_> = all.iter().collect();
        assert_eq!(
            set.len(),
            all.len(),
            "AchievementId::all() contains duplicates"
        );
    }

    // ------------------------------------------------------------------------
    // Achievement Definition Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_achievement_get_first_flight() {
        let achievement = Achievement::get(AchievementId::FirstFlight);
        assert_eq!(achievement.id, AchievementId::FirstFlight);
        assert_eq!(achievement.name, "First Flight");
        assert_eq!(achievement.tier, AchievementTier::Bronze);
        assert_eq!(achievement.category, AchievementCategory::Beginner);
        assert_eq!(achievement.target, 1);
        assert!(!achievement.hidden);
    }

    #[test]
    fn test_achievement_get_hidden_achievement() {
        let achievement = Achievement::get(AchievementId::ThePacifist);
        assert!(achievement.hidden);
        assert_eq!(achievement.category, AchievementCategory::Hidden);
        assert_eq!(achievement.tier, AchievementTier::Gold);
    }

    #[test]
    fn test_achievement_get_ufo_achievement() {
        let achievement = Achievement::get(AchievementId::FirstContact);
        assert_eq!(achievement.name, "First Contact");
        assert_eq!(achievement.tier, AchievementTier::Bronze);
        assert_eq!(achievement.target, 1);
    }

    #[test]
    fn test_achievement_get_diamond_tier() {
        let achievement = Achievement::get(AchievementId::Godlike);
        assert_eq!(achievement.tier, AchievementTier::Diamond);
        assert_eq!(achievement.target, 15);
    }

    // ------------------------------------------------------------------------
    // AchievementTier Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_achievement_tier_icon() {
        assert_eq!(AchievementTier::Bronze.icon(), "[B]");
        assert_eq!(AchievementTier::Silver.icon(), "[S]");
        assert_eq!(AchievementTier::Gold.icon(), "[G]");
        assert_eq!(AchievementTier::Diamond.icon(), "[D]");
    }

    // Note: color() test skipped - requires macroquad graphics context

    // ------------------------------------------------------------------------
    // AchievementCategory Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_achievement_category_name() {
        assert_eq!(AchievementCategory::Beginner.name(), "新手村");
        assert_eq!(AchievementCategory::Combo.name(), "连击大师");
        assert_eq!(AchievementCategory::Survival.name(), "生存模式专精");
        assert_eq!(AchievementCategory::Duel.name(), "对战模式荣耀");
        assert_eq!(AchievementCategory::Perfectionist.name(), "完美主义");
        assert_eq!(AchievementCategory::Explorer.name(), "探索与实验");
        assert_eq!(AchievementCategory::Veteran.name(), "累计成就");
        assert_eq!(AchievementCategory::Hidden.name(), "隐藏成就");
    }

    // ------------------------------------------------------------------------
    // PlayerStats Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_player_stats_default() {
        let stats = PlayerStats::default();
        assert_eq!(stats.total_playtime, 0.0);
        assert_eq!(stats.total_kills, 0);
        assert_eq!(stats.ufo_kills_total, 0);
        assert_eq!(stats.bullets_fired, 0);
        assert_eq!(stats.shields_collected, 0);
        assert_eq!(stats.games_played, 0);
        assert_eq!(stats.survival_games, 0);
        assert_eq!(stats.duel_games, 0);
        assert_eq!(stats.duel_wins, 0);
        assert_eq!(stats.settings_changed, 0);
        assert_eq!(stats.max_wave, 0);
        assert_eq!(stats.max_killstreak, 0);
        assert_eq!(stats.five_streaks, 0);
        assert!(stats.modes_played.is_empty());
        assert!(stats.weapons_used.is_empty());
    }

    // ------------------------------------------------------------------------
    // AchievementProgress Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_achievement_progress_default() {
        let progress = AchievementProgress::default();
        assert!(!progress.unlocked);
        assert!(progress.unlock_time.is_none());
        assert_eq!(progress.current, 0);
    }

    // ------------------------------------------------------------------------
    // AchievementManager Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_manager_new_without_load_initializes_all_achievements() {
        let manager = AchievementManager::new_without_load();
        assert_eq!(manager.progress.len(), AchievementId::all().len());
        for progress in manager.progress.values() {
            assert!(!progress.unlocked);
            assert_eq!(progress.current, 0);
        }
    }

    #[test]
    fn test_manager_unlock_success() {
        let mut manager = AchievementManager::new_without_load();
        let time = 10.0;
        let result = manager.unlock(AchievementId::FirstFlight, time);
        assert!(result, "First unlock should succeed");

        let progress = manager.get_progress(AchievementId::FirstFlight).unwrap();
        assert!(progress.unlocked);
        assert_eq!(progress.unlock_time, Some(time));
    }

    #[test]
    fn test_manager_unlock_already_unlocked_returns_false() {
        let mut manager = AchievementManager::new_without_load();
        manager.unlock(AchievementId::FirstFlight, 10.0);
        let result = manager.unlock(AchievementId::FirstFlight, 20.0);
        assert!(!result, "Second unlock should fail");

        // Time should remain original
        let progress = manager.get_progress(AchievementId::FirstFlight).unwrap();
        assert_eq!(progress.unlock_time, Some(10.0));
    }

    #[test]
    fn test_manager_unlock_adds_to_recently_unlocked() {
        let mut manager = AchievementManager::new_without_load();
        manager.unlock(AchievementId::FirstBlood, 5.0);
        assert_eq!(manager.recently_unlocked.len(), 1);
        assert_eq!(manager.recently_unlocked[0].0, AchievementId::FirstBlood);
        assert_eq!(manager.recently_unlocked[0].1, 5.0);
    }

    #[test]
    fn test_manager_update_progress_partial() {
        let mut manager = AchievementManager::new_without_load();
        // Marksman requires 10 hits
        manager.update_progress(AchievementId::Marksman, 5, 1.0);

        let progress = manager.get_progress(AchievementId::Marksman).unwrap();
        assert_eq!(progress.current, 5);
        assert!(!progress.unlocked, "Should not unlock at 5/10");
    }

    #[test]
    fn test_manager_update_progress_auto_unlock_at_target() {
        let mut manager = AchievementManager::new_without_load();
        // Marksman target = 10
        manager.update_progress(AchievementId::Marksman, 10, 2.0);

        let progress = manager.get_progress(AchievementId::Marksman).unwrap();
        assert!(progress.unlocked);
        assert_eq!(progress.unlock_time, Some(2.0));
    }

    #[test]
    fn test_manager_update_progress_over_target_still_unlocks() {
        let mut manager = AchievementManager::new_without_load();
        manager.update_progress(AchievementId::Marksman, 15, 3.0);

        let progress = manager.get_progress(AchievementId::Marksman).unwrap();
        assert!(progress.unlocked);
    }

    #[test]
    fn test_manager_update_progress_zero_target_no_auto_unlock() {
        let mut manager = AchievementManager::new_without_load();
        // QuickStart has target = 0 (condition-based)
        manager.update_progress(AchievementId::QuickStart, 100, 1.0);

        let progress = manager.get_progress(AchievementId::QuickStart).unwrap();
        assert!(
            !progress.unlocked,
            "Zero-target achievements need explicit unlock"
        );
    }

    #[test]
    fn test_manager_increment_progress() {
        let mut manager = AchievementManager::new_without_load();
        // Armed target = 100
        manager.increment_progress(AchievementId::Armed, 30, 1.0);
        assert_eq!(
            manager.get_progress(AchievementId::Armed).unwrap().current,
            30
        );

        manager.increment_progress(AchievementId::Armed, 70, 2.0);
        let progress = manager.get_progress(AchievementId::Armed).unwrap();
        assert_eq!(progress.current, 100);
        assert!(progress.unlocked);
    }

    #[test]
    fn test_manager_increment_progress_on_unlocked_no_change() {
        let mut manager = AchievementManager::new_without_load();
        manager.unlock(AchievementId::Armed, 1.0);
        manager.increment_progress(AchievementId::Armed, 50, 2.0);

        // Progress should not change after unlock
        let progress = manager.get_progress(AchievementId::Armed).unwrap();
        assert_eq!(progress.current, 0); // Stays at 0 because unlock skips update
    }

    #[test]
    fn test_manager_get_progress_returns_some() {
        let manager = AchievementManager::new_without_load();
        assert!(manager.get_progress(AchievementId::FirstFlight).is_some());
    }

    #[test]
    fn test_manager_is_unlocked() {
        let mut manager = AchievementManager::new_without_load();
        assert!(!manager.is_unlocked(AchievementId::Lucky));
        manager.unlock(AchievementId::Lucky, 1.0);
        assert!(manager.is_unlocked(AchievementId::Lucky));
    }

    #[test]
    fn test_manager_get_recent_unlocks_filters_by_age() {
        let mut manager = AchievementManager::new_without_load();
        manager.unlock(AchievementId::FirstFlight, 1.0);
        manager.unlock(AchievementId::FirstBlood, 5.0);
        manager.unlock(AchievementId::Marksman, 9.0);

        let recent = manager.get_recent_unlocks(3.0, 10.0);
        // At time 10.0 with max_age 3.0, only Marksman (9.0) should be recent
        assert_eq!(recent.len(), 1);
        assert!(recent.contains(&AchievementId::Marksman));
    }

    #[test]
    fn test_manager_cleanup_recent_unlocks() {
        let mut manager = AchievementManager::new_without_load();
        manager.unlock(AchievementId::FirstFlight, 1.0);
        manager.unlock(AchievementId::FirstBlood, 8.0);
        assert_eq!(manager.recently_unlocked.len(), 2);

        manager.cleanup_recent_unlocks(5.0, 10.0);
        // Only FirstBlood (8.0) should remain
        assert_eq!(manager.recently_unlocked.len(), 1);
        assert_eq!(manager.recently_unlocked[0].0, AchievementId::FirstBlood);
    }

    #[test]
    fn test_manager_get_stats() {
        let mut manager = AchievementManager::new_without_load();
        let (unlocked, total) = manager.get_stats();
        assert_eq!(unlocked, 0);
        assert_eq!(total, 41);

        manager.unlock(AchievementId::FirstFlight, 1.0);
        manager.unlock(AchievementId::FirstBlood, 2.0);
        let (unlocked, total) = manager.get_stats();
        assert_eq!(unlocked, 2);
        assert_eq!(total, 41);
    }

    #[test]
    fn test_manager_get_by_category_beginner() {
        let manager = AchievementManager::new_without_load();
        let beginner = manager.get_by_category(AchievementCategory::Beginner);
        assert_eq!(beginner.len(), 7);
        assert!(beginner.contains(&AchievementId::FirstFlight));
        assert!(beginner.contains(&AchievementId::FirstBlood));
        assert!(beginner.contains(&AchievementId::Lucky));
    }

    #[test]
    fn test_manager_get_by_category_hidden() {
        let manager = AchievementManager::new_without_load();
        let hidden = manager.get_by_category(AchievementCategory::Hidden);
        assert_eq!(hidden.len(), 3);
        assert!(hidden.contains(&AchievementId::ThePacifist));
        assert!(hidden.contains(&AchievementId::LuckySeven));
        assert!(hidden.contains(&AchievementId::MidnightWarrior));
    }

    #[test]
    fn test_manager_reset_clears_all() {
        let mut manager = AchievementManager::new_without_load();
        manager.unlock(AchievementId::FirstFlight, 1.0);
        manager.update_progress(AchievementId::Marksman, 5, 2.0);
        manager.stats.total_playtime = 100.0;
        manager.stats.total_kills = 50;

        manager.reset();

        // Check all progress reset
        for progress in manager.progress.values() {
            assert!(!progress.unlocked);
            assert_eq!(progress.current, 0);
            assert!(progress.unlock_time.is_none());
        }
        // Check stats reset
        assert_eq!(manager.stats.total_playtime, 0.0);
        assert_eq!(manager.stats.total_kills, 0);
        // Check recently_unlocked cleared
        assert!(manager.recently_unlocked.is_empty());
    }

    // ------------------------------------------------------------------------
    // parse_achievement_id Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_parse_achievement_id_known_ids() {
        let manager = AchievementManager::new_without_load();
        assert_eq!(
            manager.parse_achievement_id("FirstFlight"),
            Some(AchievementId::FirstFlight)
        );
        assert_eq!(
            manager.parse_achievement_id("Godlike"),
            Some(AchievementId::Godlike)
        );
        assert_eq!(
            manager.parse_achievement_id("CleanSweep"),
            Some(AchievementId::CleanSweep)
        );
    }

    #[test]
    fn test_parse_achievement_id_unknown_returns_none() {
        let manager = AchievementManager::new_without_load();
        assert!(manager.parse_achievement_id("Unknown").is_none());
        assert!(manager.parse_achievement_id("").is_none());
        assert!(manager.parse_achievement_id("firstflight").is_none()); // case sensitive
    }

    // ------------------------------------------------------------------------
    // Helper Method Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_extract_number() {
        let manager = AchievementManager::new_without_load();
        assert_eq!(manager.extract_number("kills: 42 total", "kills:"), 42);
        assert_eq!(manager.extract_number("no match here", "kills:"), 0);
        assert_eq!(manager.extract_number("score 100", "score"), 100);
    }

    #[test]
    fn test_extract_number_f64() {
        let manager = AchievementManager::new_without_load();
        assert_eq!(
            manager.extract_number_f64("time: 12.5 seconds", "time:"),
            12.5
        );
        assert_eq!(manager.extract_number_f64("no match", "time:"), 0.0);
    }

    // ------------------------------------------------------------------------
    // Additional Edge Case Tests (based on code review)
    // ------------------------------------------------------------------------

    #[test]
    fn test_manager_update_progress_on_unlocked_no_change() {
        let mut manager = AchievementManager::new_without_load();
        // First unlock the achievement
        manager.unlock(AchievementId::Marksman, 1.0);
        let original_time = manager
            .get_progress(AchievementId::Marksman)
            .unwrap()
            .unlock_time;

        // Try to update progress after unlock
        manager.update_progress(AchievementId::Marksman, 999, 5.0);

        let progress = manager.get_progress(AchievementId::Marksman).unwrap();
        // Progress should NOT be updated
        assert_eq!(progress.current, 0, "Progress should remain 0 after unlock");
        // unlock_time should NOT change
        assert_eq!(
            progress.unlock_time, original_time,
            "unlock_time should not change"
        );
    }

    #[test]
    fn test_manager_get_recent_unlocks_empty() {
        let manager = AchievementManager::new_without_load();
        let recent = manager.get_recent_unlocks(5.0, 10.0);
        assert!(recent.is_empty(), "No unlocks means empty recent list");
    }

    #[test]
    fn test_manager_get_recent_unlocks_all_too_old() {
        let mut manager = AchievementManager::new_without_load();
        manager.unlock(AchievementId::FirstFlight, 1.0);
        manager.unlock(AchievementId::FirstBlood, 2.0);

        // At time 100 with max_age 3, both are too old
        let recent = manager.get_recent_unlocks(3.0, 100.0);
        assert!(recent.is_empty());
    }

    #[test]
    fn test_manager_get_by_category_explorer() {
        let manager = AchievementManager::new_without_load();
        let explorer = manager.get_by_category(AchievementCategory::Explorer);
        // Explorer: Arsenal, Adventurer, Tinkerer, FirstContact (UFO)
        assert_eq!(explorer.len(), 4);
        assert!(explorer.contains(&AchievementId::Arsenal));
        assert!(explorer.contains(&AchievementId::Adventurer));
        assert!(explorer.contains(&AchievementId::FirstContact));
    }

    #[test]
    fn test_manager_get_by_category_veteran() {
        let manager = AchievementManager::new_without_load();
        let veteran = manager.get_by_category(AchievementCategory::Veteran);
        // Veteran: Veteran, Legend, SkyHunter
        assert_eq!(veteran.len(), 3);
        assert!(veteran.contains(&AchievementId::Veteran));
        assert!(veteran.contains(&AchievementId::Legend));
        assert!(veteran.contains(&AchievementId::SkyHunter));
    }

    #[test]
    fn test_manager_get_stats_repeated_unlock_no_double_count() {
        let mut manager = AchievementManager::new_without_load();
        manager.unlock(AchievementId::FirstFlight, 1.0);
        manager.unlock(AchievementId::FirstFlight, 2.0); // repeat

        let (unlocked, _) = manager.get_stats();
        assert_eq!(unlocked, 1, "Repeated unlock should not double count");
    }

    #[test]
    fn test_manager_get_stats_after_reset() {
        let mut manager = AchievementManager::new_without_load();
        manager.unlock(AchievementId::FirstFlight, 1.0);
        manager.unlock(AchievementId::FirstBlood, 2.0);
        assert_eq!(manager.get_stats().0, 2);

        manager.reset();
        let (unlocked, total) = manager.get_stats();
        assert_eq!(
            unlocked, 0,
            "After reset, no achievements should be unlocked"
        );
        assert_eq!(total, 41, "Total should still be 41");
    }

    #[test]
    fn test_manager_cleanup_with_no_recent_unlocks() {
        let mut manager = AchievementManager::new_without_load();
        // Should not panic on empty list
        manager.cleanup_recent_unlocks(5.0, 10.0);
        assert!(manager.recently_unlocked.is_empty());
    }

    #[test]
    fn test_all_achievements_have_valid_definition() {
        // Ensure Achievement::get() works for every AchievementId
        for id in AchievementId::all() {
            let achievement = Achievement::get(id);
            assert_eq!(achievement.id, id, "Achievement ID mismatch");
            assert!(
                !achievement.name.is_empty(),
                "Achievement name should not be empty"
            );
            assert!(
                !achievement.description.is_empty(),
                "Description should not be empty"
            );
        }
    }
}
