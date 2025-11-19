//! 字体管理模块
//!
//! 提供自定义字体加载和回退机制。

use crate::FontChoice;
use macroquad::text::{Font, load_ttf_font};

/// 字体系统
pub struct FontSystem {
    dejavu_sans: Option<Font>,
    ubuntu: Option<Font>,
    custom: Option<Font>,
    chinese: Option<Font>, // 中文字体
}

impl FontSystem {
    /// 加载所有可用字体
    pub async fn new() -> Self {
        let dejavu_sans = Self::try_load("assets/fonts/DejaVuSans.ttf").await;
        let ubuntu = Self::try_load("assets/fonts/ubuntu.ttf").await;
        let custom = Self::try_load("assets/fonts/font.ttf").await;
        let chinese = Self::try_load("assets/fonts/wqy-microhei.ttc").await;

        println!("字体加载状态:");
        println!(
            "  DejaVu Sans: {}",
            if dejavu_sans.is_some() { "OK" } else { "FAIL" }
        );
        println!(
            "  Ubuntu:      {}",
            if ubuntu.is_some() { "OK" } else { "FAIL" }
        );
        println!(
            "  Custom:      {}",
            if custom.is_some() { "OK" } else { "FAIL" }
        );
        println!(
            "  Chinese:     {}",
            if chinese.is_some() { "OK" } else { "FAIL" }
        );

        Self {
            dejavu_sans,
            ubuntu,
            custom,
            chinese,
        }
    }

    /// 尝试加载单个字体文件
    async fn try_load(path: &str) -> Option<Font> {
        load_ttf_font(path).await.ok()
    }

    /// 根据设置获取字体引用
    pub fn get(&self, choice: FontChoice) -> Option<&Font> {
        match choice {
            FontChoice::Default => None, // 使用 Macroquad 默认字体
            FontChoice::DejaVuSans => self.dejavu_sans.as_ref(),
            FontChoice::Ubuntu => self.ubuntu.as_ref(),
            FontChoice::Custom => self.custom.as_ref(),
        }
    }

    /// 获取中文字体（用于显示中文文本）
    #[allow(dead_code)]
    pub fn get_chinese(&self) -> Option<&Font> {
        self.chinese.as_ref()
    }

    /// 获取最佳字体（优先使用中文字体，回退到选择的字体）
    pub fn get_best(&self, choice: FontChoice) -> Option<&Font> {
        // 如果有中文字体，优先使用
        if self.chinese.is_some() {
            return self.chinese.as_ref();
        }
        // 否则使用选择的字体
        self.get(choice)
    }

    /// 检查某个字体是否可用
    #[allow(dead_code)]
    pub fn is_available(&self, choice: FontChoice) -> bool {
        match choice {
            FontChoice::Default => true, // 默认字体总是可用
            FontChoice::DejaVuSans => self.dejavu_sans.is_some(),
            FontChoice::Ubuntu => self.ubuntu.is_some(),
            FontChoice::Custom => self.custom.is_some(),
        }
    }
}
