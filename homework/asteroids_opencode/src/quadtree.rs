//! QuadTree 空间分区模块
//!
//! 使用四叉树数据结构优化碰撞检测性能。
//!
//! ## 性能优化
//! - 将碰撞检测从 O(n²) 降低到 O(n log n)
//! - 递归分割 2D 空间为四个象限
//! - 最大深度 5 层，每节点最多 4 个对象
//!
//! ## 应用场景
//! - 玩家飞船与小行星碰撞
//! - 子弹与小行星碰撞
//! - 适用于大量动态对象的场景

use macroquad::prelude::*;

const MAX_OBJECTS: usize = 4; // 每个节点最多存储的对象数
const MAX_DEPTH: usize = 5; // 最大递归深度

/// QuadTree 中存储的对象索引
#[derive(Clone, Copy, Debug)]
pub struct ObjectIndex {
    pub index: usize,
    pub pos: Vec2,
    pub radius: f32,
}

/// QuadTree 边界矩形
#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Bounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// 检查点是否在边界内
    #[allow(dead_code)]
    pub fn contains_point(&self, pos: Vec2) -> bool {
        pos.x >= self.x
            && pos.x <= self.x + self.width
            && pos.y >= self.y
            && pos.y <= self.y + self.height
    }

    /// 检查圆是否与边界相交
    pub fn intersects_circle(&self, pos: Vec2, radius: f32) -> bool {
        // 找到矩形中离圆心最近的点
        let closest_x = pos.x.clamp(self.x, self.x + self.width);
        let closest_y = pos.y.clamp(self.y, self.y + self.height);

        // 计算距离
        let distance = ((pos.x - closest_x).powi(2) + (pos.y - closest_y).powi(2)).sqrt();
        distance <= radius
    }
}

/// QuadTree 节点
pub struct QuadTree {
    depth: usize,
    bounds: Bounds,
    objects: Vec<ObjectIndex>,
    children: Option<Box<[QuadTree; 4]>>,
}

impl QuadTree {
    /// 创建新的 QuadTree
    pub fn new(bounds: Bounds) -> Self {
        Self::with_depth(bounds, 0)
    }

    fn with_depth(bounds: Bounds, depth: usize) -> Self {
        Self {
            depth,
            bounds,
            objects: Vec::new(),
            children: None,
        }
    }

    /// 清空 QuadTree
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.objects.clear();
        self.children = None;
    }

    /// 分裂节点为 4 个子节点
    fn split(&mut self) {
        let half_width = self.bounds.width / 2.0;
        let half_height = self.bounds.height / 2.0;
        let x = self.bounds.x;
        let y = self.bounds.y;
        let next_depth = self.depth + 1;

        self.children = Some(Box::new([
            // 右上
            QuadTree::with_depth(
                Bounds::new(x + half_width, y, half_width, half_height),
                next_depth,
            ),
            // 左上
            QuadTree::with_depth(Bounds::new(x, y, half_width, half_height), next_depth),
            // 左下
            QuadTree::with_depth(
                Bounds::new(x, y + half_height, half_width, half_height),
                next_depth,
            ),
            // 右下
            QuadTree::with_depth(
                Bounds::new(x + half_width, y + half_height, half_width, half_height),
                next_depth,
            ),
        ]));
    }

    /// 获取对象应该插入的子节点索引（如果有）
    fn get_child_index(&self, obj: &ObjectIndex) -> Option<usize> {
        self.children.as_ref()?;

        let mid_x = self.bounds.x + self.bounds.width / 2.0;
        let mid_y = self.bounds.y + self.bounds.height / 2.0;

        // 检查对象是否完全在某个子节点内
        let top = obj.pos.y - obj.radius < mid_y;
        let bottom = obj.pos.y + obj.radius > mid_y;
        let left = obj.pos.x - obj.radius < mid_x;
        let right = obj.pos.x + obj.radius > mid_x;

        // 对象必须完全在一个象限内才能插入子节点
        if top && !bottom {
            if right && !left {
                Some(0) // 右上
            } else if left && !right {
                Some(1) // 左上
            } else {
                None
            }
        } else if bottom && !top {
            if left && !right {
                Some(2) // 左下
            } else if right && !left {
                Some(3) // 右下
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 插入对象
    pub fn insert(&mut self, obj: ObjectIndex) {
        // 如果有子节点，尝试插入子节点
        if self.children.is_some() {
            let index = self.get_child_index(&obj);
            if let Some(idx) = index
                && let Some(ref mut children) = self.children
            {
                children[idx].insert(obj);
                return;
            }
        }

        // 添加到当前节点
        self.objects.push(obj);

        // 如果超过容量且未达到最大深度，分裂节点
        if self.objects.len() > MAX_OBJECTS && self.depth < MAX_DEPTH && self.children.is_none() {
            self.split();

            // 重新分配现有对象
            let objects_to_redistribute: Vec<_> = self.objects.drain(..).collect();
            let mut remaining = Vec::new();

            for obj in objects_to_redistribute {
                let index = self.get_child_index(&obj);
                if let Some(idx) = index {
                    if let Some(ref mut children) = self.children {
                        children[idx].insert(obj);
                    }
                } else {
                    remaining.push(obj);
                }
            }
            self.objects = remaining;
        }
    }

    /// 查询与给定圆相交的所有对象
    pub fn query(&self, pos: Vec2, radius: f32, result: &mut Vec<ObjectIndex>) {
        // 检查边界是否相交
        if !self.bounds.intersects_circle(pos, radius) {
            return;
        }

        // 添加当前节点的对象
        for &obj in &self.objects {
            let dist_sq = (obj.pos - pos).length_squared();
            let radius_sum = obj.radius + radius;
            if dist_sq <= radius_sum * radius_sum {
                result.push(obj);
            }
        }

        // 递归查询子节点
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query(pos, radius, result);
            }
        }
    }

    /// 获取所有对象（用于调试）
    #[allow(dead_code)]
    pub fn get_all_objects(&self, result: &mut Vec<ObjectIndex>) {
        result.extend_from_slice(&self.objects);
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.get_all_objects(result);
            }
        }
    }

    /// 获取树的最大深度（用于性能监控）
    pub fn max_depth(&self) -> usize {
        if let Some(ref children) = self.children {
            let mut max = self.depth;
            for child in children.iter() {
                max = max.max(child.max_depth());
            }
            max
        } else {
            self.depth
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_contains_point() {
        let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);
        assert!(bounds.contains_point(Vec2::new(50.0, 50.0)));
        assert!(bounds.contains_point(Vec2::new(0.0, 0.0)));
        assert!(bounds.contains_point(Vec2::new(100.0, 100.0)));
        assert!(!bounds.contains_point(Vec2::new(101.0, 50.0)));
        assert!(!bounds.contains_point(Vec2::new(-1.0, 50.0)));
    }

    #[test]
    fn test_bounds_intersects_circle() {
        let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);
        assert!(bounds.intersects_circle(Vec2::new(50.0, 50.0), 10.0));
        assert!(bounds.intersects_circle(Vec2::new(0.0, 0.0), 1.0));
        assert!(bounds.intersects_circle(Vec2::new(110.0, 50.0), 15.0)); // 边缘相交
        assert!(!bounds.intersects_circle(Vec2::new(200.0, 200.0), 10.0));
    }

    #[test]
    fn test_quadtree_insert_and_query() {
        let mut tree = QuadTree::new(Bounds::new(0.0, 0.0, 800.0, 600.0));

        // 插入一些对象
        tree.insert(ObjectIndex {
            index: 0,
            pos: Vec2::new(100.0, 100.0),
            radius: 10.0,
        });
        tree.insert(ObjectIndex {
            index: 1,
            pos: Vec2::new(200.0, 200.0),
            radius: 10.0,
        });
        tree.insert(ObjectIndex {
            index: 2,
            pos: Vec2::new(700.0, 500.0),
            radius: 10.0,
        });

        // 查询靠近第一个对象的区域
        let mut result = Vec::new();
        tree.query(Vec2::new(105.0, 105.0), 20.0, &mut result);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index, 0);

        // 查询大范围
        result.clear();
        tree.query(Vec2::new(400.0, 300.0), 500.0, &mut result);
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_quadtree_split() {
        let mut tree = QuadTree::new(Bounds::new(0.0, 0.0, 800.0, 600.0));

        // 插入超过 MAX_OBJECTS 个对象
        for i in 0..10 {
            tree.insert(ObjectIndex {
                index: i,
                pos: Vec2::new(100.0 + i as f32 * 10.0, 100.0),
                radius: 5.0,
            });
        }

        // 验证树已经分裂
        assert!(tree.children.is_some());
    }
}
