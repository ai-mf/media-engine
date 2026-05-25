import sys
import random

width = 320
height = 240
frames = 30  # 1 second at 30fps

for _ in range(frames):
    for _ in range(width * height):
        sys.stdout.buffer.write(bytes([
            random.randint(0,255),
            random.randint(0,255),
            random.randint(0,255),
        ]))