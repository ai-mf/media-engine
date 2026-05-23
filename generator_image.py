import sys
import random

width = 256
height = 256

for _ in range(width * height):
    r = random.randint(0, 255)
    g = random.randint(0, 255)
    b = random.randint(0, 255)
    sys.stdout.buffer.write(bytes([r, g, b]))