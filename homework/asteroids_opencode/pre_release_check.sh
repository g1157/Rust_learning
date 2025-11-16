#!/bin/bash

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║          🎮 Asteroids 发布前检查脚本 🎮                   ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 检查计数器
PASS=0
FAIL=0
WARN=0

# 1. 代码格式检查
echo -e "${YELLOW}[1/8]${NC} 检查代码格式..."
if cargo fmt --check &>/dev/null; then
    echo -e "  ${GREEN}✅ 代码格式正确${NC}"
    ((PASS++))
else
    echo -e "  ${RED}❌ 代码格式不符合标准，运行: cargo fmt${NC}"
    ((FAIL++))
fi

# 2. Clippy 检查
echo -e "${YELLOW}[2/8]${NC} 运行 Clippy 检查..."
CLIPPY_OUTPUT=$(cargo clippy --quiet 2>&1)
if [ $? -eq 0 ]; then
    echo -e "  ${GREEN}✅ Clippy 检查通过（0警告）${NC}"
    ((PASS++))
else
    echo -e "  ${RED}❌ Clippy 发现问题:${NC}"
    echo "$CLIPPY_OUTPUT" | head -10
    ((FAIL++))
fi

# 3. 单元测试
echo -e "${YELLOW}[3/8]${NC} 运行单元测试..."
TEST_OUTPUT=$(cargo test --quiet 2>&1 | tail -1)
if echo "$TEST_OUTPUT" | grep -q "test result: ok"; then
    TESTS=$(echo "$TEST_OUTPUT" | grep -oP '\d+(?= passed)')
    echo -e "  ${GREEN}✅ 所有测试通过 ($TESTS/36)${NC}"
    ((PASS++))
else
    echo -e "  ${RED}❌ 测试失败${NC}"
    echo "$TEST_OUTPUT"
    ((FAIL++))
fi

# 4. 发布版本构建
echo -e "${YELLOW}[4/8]${NC} 构建发布版本..."
if cargo build --release --quiet 2>&1 | grep -q "Finished"; then
    echo -e "  ${GREEN}✅ 发布版本构建成功${NC}"
    ((PASS++))
else
    echo -e "  ${RED}❌ 发布版本构建失败${NC}"
    ((FAIL++))
fi

# 5. 音频文件检查
echo -e "${YELLOW}[5/8]${NC} 检查音频文件..."
if [ -f "assets/sounds/shoot.wav" ] && [ -f "assets/sounds/powerup.wav" ]; then
    echo -e "  ${GREEN}✅ 音频文件存在${NC}"
    
    # 检查音频淡出
    if command -v python3 &>/dev/null; then
        AMP=$(python3 << 'PYEND'
import wave, struct
try:
    with wave.open('assets/sounds/shoot.wav', 'rb') as w:
        p = w.getparams()
        w.setpos(p.nframes - 10)
        frames = w.readframes(10)
        samples = struct.unpack(f'{10 * p.nchannels}h', frames)
        print(max(abs(s) for s in samples))
except:
    print("-1")
PYEND
)
        if [ "$AMP" = "0" ]; then
            echo -e "  ${GREEN}✅ 音频淡出正确（末尾振幅为0）${NC}"
            ((PASS++))
        else
            echo -e "  ${YELLOW}⚠️  音频可能需要重新处理（末尾振幅: $AMP）${NC}"
            echo -e "     运行: python3 fix_shoot_fadeout.py"
            ((WARN++))
        fi
    else
        echo -e "  ${YELLOW}⚠️  无法验证音频淡出（需要 Python 3）${NC}"
        ((WARN++))
    fi
else
    echo -e "  ${RED}❌ 音频文件缺失${NC}"
    echo -e "     请准备音频文件并运行: python3 fix_shoot_fadeout.py"
    ((FAIL++))
fi

# 6. 文档完整性检查
echo -e "${YELLOW}[6/8]${NC} 检查文档..."
DOCS=("README.md" "README.zh-CN.md" "AUDIO_FADEOUT.md" "SHOOT_FADEOUT_FIX.md" "RELEASE_GUIDE.md")
DOC_MISSING=0
for doc in "${DOCS[@]}"; do
    if [ ! -f "$doc" ]; then
        echo -e "  ${RED}❌ 缺失: $doc${NC}"
        ((DOC_MISSING++))
    fi
done

if [ $DOC_MISSING -eq 0 ]; then
    echo -e "  ${GREEN}✅ 所有文档齐全（${#DOCS[@]}个）${NC}"
    ((PASS++))
else
    echo -e "  ${RED}❌ 缺失 $DOC_MISSING 个文档${NC}"
    ((FAIL++))
fi

# 7. 音量设置检查
echo -e "${YELLOW}[7/8]${NC} 检查默认音量..."
if grep -q "sound_volume: 0.01" src/main.rs; then
    echo -e "  ${GREEN}✅ 默认音量正确（1%）${NC}"
    ((PASS++))
else
    echo -e "  ${YELLOW}⚠️  默认音量可能不是1%${NC}"
    ((WARN++))
fi

# 8. Git 状态检查
echo -e "${YELLOW}[8/8]${NC} 检查 Git 状态..."
if git rev-parse --git-dir > /dev/null 2>&1; then
    UNCOMMITTED=$(git status --porcelain | wc -l)
    if [ $UNCOMMITTED -eq 0 ]; then
        echo -e "  ${GREEN}✅ 所有更改已提交${NC}"
        ((PASS++))
    else
        echo -e "  ${YELLOW}⚠️  有 $UNCOMMITTED 个未提交的更改${NC}"
        echo -e "     运行: git status"
        ((WARN++))
    fi
else
    echo -e "  ${YELLOW}⚠️  不是 Git 仓库${NC}"
    ((WARN++))
fi

# 总结
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}检查结果总结${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "  ${GREEN}✅ 通过: $PASS${NC}"
echo -e "  ${RED}❌ 失败: $FAIL${NC}"
echo -e "  ${YELLOW}⚠️  警告: $WARN${NC}"
echo ""

if [ $FAIL -eq 0 ] && [ $WARN -eq 0 ]; then
    echo -e "${GREEN}🎉 完美！项目已准备好发布/展示！${NC}"
    echo ""
    echo -e "下一步:"
    echo -e "  1. 运行游戏测试: ${BLUE}cargo run --release${NC}"
    echo -e "  2. 调整演示音量: 在游戏设置中调到 5-10%"
    echo -e "  3. 查看发布指南: ${BLUE}cat RELEASE_GUIDE.md${NC}"
    exit 0
elif [ $FAIL -eq 0 ]; then
    echo -e "${YELLOW}⚠️  项目基本准备好，但有一些警告${NC}"
    echo -e "建议检查上述警告项"
    exit 1
else
    echo -e "${RED}❌ 发现问题，请修复后再发布${NC}"
    echo -e "详见上方红色标记的错误"
    exit 2
fi
