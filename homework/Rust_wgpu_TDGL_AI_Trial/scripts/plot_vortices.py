#!/usr/bin/env python3
"""绘制涡旋统计曲线 N_v(t)"""

import pandas as pd
import matplotlib.pyplot as plt

def main():
    # 读取数据
    df = pd.read_csv('vortices.csv')

    # 创建图表
    fig, ax = plt.subplots(figsize=(10, 6))

    ax.plot(df['time'], df['vortices'], 'b-', label='Vortices (+)', linewidth=2)
    ax.plot(df['time'], df['antivortices'], 'r--', label='Antivortices (-)', linewidth=2)

    ax.set_xlabel('Time (t)', fontsize=12)
    ax.set_ylabel('Vortex Count', fontsize=12)
    ax.set_title('Vortex Dynamics in TDGL Simulation (B=0.02)', fontsize=14)
    ax.legend()
    ax.grid(True, alpha=0.3)

    # 保存图片
    plt.savefig('vortices_plot.png', dpi=150, bbox_inches='tight')
    print('图表已保存: vortices_plot.png')
    plt.show()

if __name__ == '__main__':
    main()
