"""Synthesise the game's sound effects into assets/sfx/*.wav.

    python3 art/sfx.py

Pure standard library: oscillators, noise, envelopes and a one-pole filter.
"""

import math
import os
import random
import struct
import wave

RATE = 44100
OUT = "assets/sfx"


def write(name, samples, gain=0.8):
    peak = max(1e-9, max(abs(s) for s in samples))
    k = gain / peak
    os.makedirs(OUT, exist_ok=True)
    with wave.open(f"{OUT}/{name}.wav", "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(b"".join(struct.pack("<h", int(max(-1, min(1, s * k)) * 32767)) for s in samples))
    print(f"[sfx] {name:14} {len(samples) / RATE:.2f}s")


def n(sec):
    return int(sec * RATE)


def env(i, total, attack=0.005, curve=2.0):
    t = i / RATE
    a = min(1.0, t / attack) if attack > 0 else 1.0
    return a * (1 - i / total) ** curve


def lowpass(samples, cutoff):
    out, y = [], 0.0
    a = 1 - math.exp(-2 * math.pi * cutoff / RATE)
    for s in samples:
        y += a * (s - y)
        out.append(y)
    return out


def sweep(f0, f1, sec, shape="square", curve=2.0, attack=0.002):
    total = n(sec)
    out, ph = [], 0.0
    for i in range(total):
        t = i / total
        f = f0 * (f1 / f0) ** t
        ph += f / RATE
        if shape == "square":
            v = 1.0 if (ph % 1) < 0.5 else -1.0
        elif shape == "saw":
            v = 2 * (ph % 1) - 1
        elif shape == "tri":
            v = 4 * abs((ph % 1) - 0.5) - 1
        else:
            v = math.sin(2 * math.pi * ph)
        out.append(v * env(i, total, attack, curve))
    return out


def noise(sec, cutoff, curve=1.5, attack=0.002, seed=1):
    rnd = random.Random(seed)
    total = n(sec)
    raw = [rnd.uniform(-1, 1) * env(i, total, attack, curve) for i in range(total)]
    return lowpass(raw, cutoff)


def mix(*tracks):
    length = max(len(t) for t, _ in tracks)
    out = [0.0] * length
    for t, g in tracks:
        for i, s in enumerate(t):
            out[i] += s * g
    return out


def seq(*parts):
    out = []
    for p in parts:
        out.extend(p)
    return out


def silence(sec):
    return [0.0] * n(sec)


# Click laser: bright descending zap.
write("laser", mix((sweep(1800, 300, 0.12, "square", 2.5), 0.5), (sweep(3600, 900, 0.06, "sine", 3), 0.4)), 0.5)
# Fleet weapons
write("blaster", sweep(1200, 200, 0.08, "saw", 2.5), 0.35)
write("cannon", mix((noise(0.12, 2500, 2.5, seed=3), 0.8), (sweep(180, 60, 0.12, "sine", 2), 0.8)), 0.4)
write("missile", mix((noise(0.35, 1400, 1.2, attack=0.05, seed=4), 0.8), (sweep(300, 900, 0.35, "saw", 1.5, 0.05), 0.15)), 0.35)
write("beam", mix((sweep(220, 180, 0.4, "saw", 1.2, 0.02), 0.5), (sweep(880, 700, 0.4, "square", 1.2, 0.02), 0.2)), 0.3)
# Impacts and kills
write("hit", mix((noise(0.05, 4000, 3, seed=5), 0.7), (sweep(500, 200, 0.05, "square", 3), 0.3)), 0.35)
write("crit", mix((noise(0.1, 6000, 2.5, seed=6), 0.6), (sweep(2400, 600, 0.12, "square", 2), 0.4)), 0.55)
write("explode", mix((noise(0.6, 900, 1.8, seed=7), 1.0), (sweep(120, 35, 0.5, "sine", 1.5), 0.9)), 0.7)
write("explode_big", mix((noise(1.4, 600, 1.4, seed=8), 1.0), (sweep(90, 25, 1.2, "sine", 1.3), 1.0), (noise(0.3, 5000, 3, seed=9), 0.3)), 0.85)
# Economy and UI
write("coin", seq(sweep(1320, 1320, 0.05, "square", 0.5), sweep(1980, 1980, 0.12, "square", 2.5)), 0.22)
write("buy", seq(sweep(660, 660, 0.05, "tri", 0.5), sweep(990, 990, 0.05, "tri", 0.5), sweep(1320, 1320, 0.12, "tri", 2)), 0.4)
write("deny", mix((sweep(160, 120, 0.15, "square", 1), 0.5),), 0.3)
write("tab", sweep(900, 1100, 0.03, "tri", 1), 0.25)
# Events
write("boss", seq(*[seq(sweep(440, 330, 0.22, "square", 0.6), silence(0.06)) for _ in range(3)]), 0.4)
write("warp", mix((sweep(80, 1600, 1.0, "saw", 0.6, 0.3), 0.4), (noise(1.0, 3000, 1, attack=0.4, seed=10), 0.5)), 0.5)
write("ability", seq(sweep(440, 880, 0.12, "square", 0.3), sweep(880, 1760, 0.2, "tri", 2)), 0.4)
write("sector", seq(*[sweep(f, f, d, "tri", 1.5) for f, d in [(523, 0.1), (659, 0.1), (784, 0.1), (1046, 0.3)]]), 0.45)
write("levelup", seq(*[sweep(f, f, 0.07, "square", 1.0) for f in [784, 988, 1175, 1568]]), 0.3)

# survival
write("alarm", seq(sweep(880, 660, 0.18, "square", 0.4), silence(0.05), sweep(880, 660, 0.18, "square", 0.4)), 0.3)
write("ship_lost", mix((noise(0.5, 1200, 1.6, seed=11), 0.9), (sweep(260, 50, 0.45, "saw", 1.4), 0.5)), 0.55)
write("death", seq(mix((noise(1.8, 500, 1.2, seed=12), 1.0), (sweep(70, 20, 1.8, "sine", 1.1), 1.0), (sweep(440, 55, 1.6, "saw", 1.2), 0.35))), 0.9)
