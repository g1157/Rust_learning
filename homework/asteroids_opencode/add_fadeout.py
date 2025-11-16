#!/usr/bin/env python3
"""
为 WAV 音频文件添加淡出效果，避免结尾电流声
使用 Python 标准库 wave 和 struct
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
        fadeout_duration_ms: 淡出时长（毫秒），默认 50ms
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
    
    # 备份原始文件
    backup_dir = f"{sounds_dir}/original_backup"
    os.makedirs(backup_dir, exist_ok=True)
    
    # 查找所有 WAV 文件
    wav_files = [f for f in os.listdir(sounds_dir) if f.endswith('.wav')]
    
    if not wav_files:
        print(f"在 {sounds_dir} 中未找到 WAV 文件")
        sys.exit(1)
    
    print(f"找到 {len(wav_files)} 个 WAV 文件\n")
    
    for filename in wav_files:
        input_path = os.path.join(sounds_dir, filename)
        backup_path = os.path.join(backup_dir, filename)
        
        # 备份原始文件
        if not os.path.exists(backup_path):
            import shutil
            shutil.copy2(input_path, backup_path)
            print(f"已备份: {backup_path}")
        
        # 创建临时输出文件
        temp_output = input_path + ".temp.wav"
        
        try:
            # 添加淡出效果
            add_fadeout(input_path, temp_output, fadeout_duration_ms=50)
            
            # 替换原文件
            os.replace(temp_output, input_path)
            
        except Exception as e:
            print(f"  ✗ 处理失败: {e}\n")
            if os.path.exists(temp_output):
                os.remove(temp_output)
    
    print("=" * 50)
    print("处理完成！")
    print(f"原始文件已备份到: {backup_dir}")
    print("=" * 50)

if __name__ == "__main__":
    main()
