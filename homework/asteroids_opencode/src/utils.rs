//! 工具函数模块
//!
//! 提供碰撞检测和几何计算辅助函数。
//!
//! ## 功能
//! - 点在三角形内判定
//! - 圆与三角形相交检测
//! - 点到线段距离计算
//! - 屏幕边界环绕

use macroquad::prelude::*;

/// 屏幕边界环绕：当物体超出屏幕边界时，从对面重新进入
pub fn wrap_around(v: &Vec2) -> Vec2 {
    let mut result = Vec2::new(v.x, v.y);
    if result.x > screen_width() {
        result.x = 0.;
    }
    if result.x < 0. {
        result.x = screen_width();
    }
    if result.y > screen_height() {
        result.y = 0.;
    }
    if result.y < 0. {
        result.y = screen_height();
    }
    result
}

/// 检测圆形是否与三角形相交
/// 用于检测子弹/小行星与飞船的碰撞
pub fn circle_intersects_triangle(center: Vec2, radius: f32, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let radius_sq = radius * radius;
    // 检查圆心是否在三角形顶点半径内
    if (a - center).length_squared() <= radius_sq
        || (b - center).length_squared() <= radius_sq
        || (c - center).length_squared() <= radius_sq
    {
        return true;
    }

    // 检查圆心是否在三角形内部
    if point_in_triangle(center, a, b, c) {
        return true;
    }

    // 检查圆是否与三角形的任一边相交
    segment_distance_sq(center, a, b) <= radius_sq
        || segment_distance_sq(center, b, c) <= radius_sq
        || segment_distance_sq(center, c, a) <= radius_sq
}

/// 使用重心坐标法检测点是否在三角形内
fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let v0 = c - a;
    let v1 = b - a;
    let v2 = p - a;

    let dot00 = v0.dot(v0);
    let dot01 = v0.dot(v1);
    let dot02 = v0.dot(v2);
    let dot11 = v1.dot(v1);
    let dot12 = v1.dot(v2);

    let denom = dot00 * dot11 - dot01 * dot01;
    if denom.abs() < f32::EPSILON {
        return false;
    }

    let inv_denom = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
    u >= 0.0 && v >= 0.0 && (u + v) <= 1.0
}

/// 计算点到线段的最短距离的平方
fn segment_distance_sq(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();

    // 处理退化情况：线段两端点重合
    if len_sq < f32::EPSILON {
        return (p - a).length_squared();
    }

    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let projection = a + ab * t;
    (projection - p).length_squared()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_in_triangle_inside() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        let c = Vec2::new(5.0, 10.0);
        let p = Vec2::new(5.0, 3.0);
        assert!(point_in_triangle(p, a, b, c));
    }

    #[test]
    fn test_point_in_triangle_outside() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        let c = Vec2::new(5.0, 10.0);
        let p = Vec2::new(15.0, 3.0);
        assert!(!point_in_triangle(p, a, b, c));
    }

    #[test]
    fn test_point_in_triangle_on_edge() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        let c = Vec2::new(5.0, 10.0);
        let p = Vec2::new(5.0, 0.0); // 在 ab 边上
        assert!(point_in_triangle(p, a, b, c));
    }

    #[test]
    fn test_segment_distance_sq_on_segment() {
        let p = Vec2::new(5.0, 0.0);
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        assert!((segment_distance_sq(p, a, b) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_segment_distance_sq_perpendicular() {
        let p = Vec2::new(5.0, 5.0);
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        assert!((segment_distance_sq(p, a, b) - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_segment_distance_sq_beyond_endpoint() {
        let p = Vec2::new(15.0, 0.0);
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        // 最近点应该是 b (10, 0)，距离的平方是 25
        assert!((segment_distance_sq(p, a, b) - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_circle_intersects_triangle_vertex() {
        let center = Vec2::new(0.0, 0.0);
        let radius = 2.0;
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        let c = Vec2::new(5.0, 10.0);
        assert!(circle_intersects_triangle(center, radius, a, b, c));
    }

    #[test]
    fn test_circle_intersects_triangle_center_inside() {
        let center = Vec2::new(5.0, 3.0);
        let radius = 1.0;
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        let c = Vec2::new(5.0, 10.0);
        assert!(circle_intersects_triangle(center, radius, a, b, c));
    }

    #[test]
    fn test_circle_intersects_triangle_edge() {
        let center = Vec2::new(5.0, -2.0);
        let radius = 2.5;
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        let c = Vec2::new(5.0, 10.0);
        assert!(circle_intersects_triangle(center, radius, a, b, c));
    }

    #[test]
    fn test_circle_no_intersection() {
        let center = Vec2::new(50.0, 50.0);
        let radius = 5.0;
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        let c = Vec2::new(5.0, 10.0);
        assert!(!circle_intersects_triangle(center, radius, a, b, c));
    }
}
