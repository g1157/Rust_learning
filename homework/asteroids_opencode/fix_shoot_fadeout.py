#!/usr/bin/env python3
"""
为音效文件添加更激进的淡出效果，彻底消除结尾杂音
针对不同音效使用不同的淡出时长
"""

import wave
import struct
import sys
import os

def add_fadeout(input_file, output_file, fadeout_duration_ms=50):
    """
    为 WAV 文件添加淡出效果
    
    Args:
        input_file: 输入 WAV 文件路径
        output_file: 输出 WAV 文件路径
        fadeout_duration_ms: 淡出时长（毫秒）
    """
    print(f"处理文件: {input_file}")
    
    # 打开输入文件
    with wave.open(input_file, 'rb') as wav_in:
        # 获取音频参数
        params = wav_in.getparams()
        nchannels = params.nchannels
        sampwidth = params.sampwidth
        framerate = params.framerate
        nframes = params.nframes
        
        print(f"  声道数: {nchannels}")
        print(f"  采样宽度: {sampwidth} 字节")
        print(f"  采样率: {framerate} Hz")
        print(f"  总帧数: {nframes}")
        print(f"  时长: {nframes / framerate:.2f} 秒")
        
        # 读取所有音频数据
        audio_data = wav_in.readframes(nframes)
    
    # 计算淡出帧数
    fadeout_frames = int(framerate * fadeout_duration_ms / 1000)
    print(f"  淡出时长: {fadeout_duration_ms} ms ({fadeout_frames} 帧)")
    
    # 将字节数据转换为采样点
    if sampwidth == 1:
        fmt = f'{nframes * nchannels}B'  # unsigned byte
    elif sampwidth == 2:
        fmt = f'{nframes * nchannels}h'  # signed short
    else:
        raise ValueError(f"不支持的采样宽度: {sampwidth}")
    
    samples = list(struct.unpack(fmt, audio_data))
    
    # 应用淡出效果到最后 fadeout_frames 帧
    start_fade = max(0, nframes - fadeout_frames)
    for i in range(start_fade, nframes):
        # 计算淡出系数 (1.0 到 0.0)
        fade_factor = 1.0 - (i - start_fade) / fadeout_frames
        
        # 对每个声道应用淡出
        for ch in range(nchannels):
            sample_index = i * nchannels + ch
            samples[sample_index] = int(samples[sample_index] * fade_factor)
    
    # 将采样点转换回字节数据
    faded_data = struct.pack(fmt, *samples)
    
    # 写入输出文件
    with wave.open(output_file, 'wb') as wav_out:
        wav_out.setparams(params)
        wav_out.writeframes(faded_data)
    
    print(f"  ✓ 已保存到: {output_file}\n")

def main():
    # 处理所有音效文件
    sounds_dir = "assets/sounds"
    
    if not os.path.exists(sounds_dir):
        print(f"错误: 目录不存在 {sounds_dir}")
        sys.exit(1)
    
    # 备份目录
    backup_dir = f"{sounds_dir}/original_backup"
    
    # 检查是否有备份
    if not os.path.exists(backup_dir):
        print(f"错误: 未找到备份目录 {backup_dir}")
        print("请先运行 add_fadeout.py 创建备份")
        sys.exit(1)
    
    # 从备份恢复文件
    print("从备份恢复原始文件...")
    import shutil
    for filename in os.listdir(backup_dir):
        if filename.endswith('.wav'):
            src = os.path.join(backup_dir, filename)
            dst = os.path.join(sounds_dir, filename)
            shutil.copy2(src, dst)
            print(f"  ✓ 恢复: {filename}")
    print()
    
    # 查找所有 WAV 文件
    wav_files = [f for f in os.listdir(sounds_dir) if f.endswith('.wav')]
    
    if not wav_files:
        print(f"在 {sounds_dir} 中未找到 WAV 文件")
        sys.exit(1)
    
    print(f"找到 {len(wav_files)} 个 WAV 文件\n")
    
    # 不同音效使用不同的淡出时长
    fadeout_settings = {
        'shoot.wav': 250,      # 射击音效：250ms 超长淡出（66%）
        'powerup.wav': 150,    # 道具音效：150ms
        'explosion.wav': 100,  # 爆炸音效：100ms
        'hit.wav': 150,        # 碰撞音效：150ms
        'thrust.wav': 80,      # 推进音效：80ms
    }
    
    for filename in wav_files:
        input_path = os.path.join(sounds_dir, filename)
        
        # 获取该文件的淡出时长
        fadeout_ms = fadeout_settings.get(filename, 100)  # 默认 100ms
        
        # 创建临时输出文件
        temp_output = input_path + ".temp.wav"
        
        try:
            # 添加淡出效果
            add_fadeout(input_path, temp_output, fadeout_duration_ms=fadeout_ms)
            
            # 替换原文件
            os.replace(temp_output, input_path)
            
        except Exception as e:
            print(f"  ✗ 处理失败: {e}\n")
            if os.path.exists(temp_output):
                os.remove(temp_output)
    
    print("=" * 50)
    print("处理完成！")
    print("射击音效使用了 250ms 超长淡出（66%淡出占比）")
    print("=" * 50)

if __name__ == "__main__":
    main()
