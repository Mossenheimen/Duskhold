"""How fast can a pure-Python Duskhold tick be? A stripped-down tick: the dead hear and see the
player (Bresenham sight lines through blocked tiles) and step greedily toward it, one per tile, no
corner cutting. A real tick also does fields, bodies, needs, items, AI states and pathfinding."""
import os, random, sys, time

W = H = 96
TICK_HZ = 10


def run(n_dead, ticks, seed=1):
    rnd = random.Random(seed)
    blocked = [[rnd.random() < 0.12 for _ in range(W)] for _ in range(H)]
    occ = [[-1] * W for _ in range(H)]
    px, py = W // 2, H // 2
    blocked[py][px] = False
    dead = []  # [x, y, target_x, target_y, progress]
    while len(dead) < n_dead:
        x, y = rnd.randrange(W), rnd.randrange(H)
        if not blocked[y][x] and occ[y][x] < 0 and (x, y) != (px, py):
            occ[y][x] = len(dead)
            dead.append([x, y, -1, -1, 0.0])

    def sees(x0, y0, x1, y1):
        dx, dy = abs(x1 - x0), -abs(y1 - y0)
        sx, sy = (1 if x0 < x1 else -1), (1 if y0 < y1 else -1)
        err = dx + dy
        while (x0, y0) != (x1, y1):
            e2 = 2 * err
            if e2 >= dy:
                err += dy; x0 += sx
            if e2 <= dx:
                err += dx; y0 += sy
            if (x0, y0) != (x1, y1) and blocked[y0][x0]:
                return False
        return True

    def free(x, y):
        return 0 <= x < W and 0 <= y < H and not blocked[y][x] and occ[y][x] < 0 and (x, y) != (px, py)

    t0 = time.perf_counter()
    for tick in range(ticks):
        if tick % 5 == 0:  # the player wanders a tile every half second
            nx, ny = px + rnd.choice((-1, 0, 1)), py + rnd.choice((-1, 0, 1))
            if 0 <= nx < W and 0 <= ny < H and not blocked[ny][nx] and occ[ny][nx] < 0:
                px, py = nx, ny
        noise = tick % 10 == 0  # a noise every second, heard 20 tiles away
        for i, z in enumerate(dead):
            x, y = z[0], z[1]
            dx, dy = px - x, py - y
            d2 = dx * dx + dy * dy
            if (d2 <= 225 and sees(x, y, px, py)) or (noise and d2 <= 400):
                z[2], z[3] = px, py
            if z[2] < 0:
                continue
            z[4] += 0.08  # 0.8 tiles a second
            if z[4] < 1.0:
                continue
            z[4] -= 1.0
            sx = (z[2] > x) - (z[2] < x)
            sy = (z[3] > y) - (z[3] < y)
            for mx, my in ((sx, sy), (sx, 0), (0, sy)):
                if (mx or my) and free(x + mx, y + my) and not (mx and my and (blocked[y][x + mx] or blocked[y + my][x])):
                    occ[y][x] = -1
                    z[0], z[1] = x + mx, y + my
                    occ[z[1]][z[0]] = i
                    break
    return ticks / (time.perf_counter() - t0)


if __name__ == "__main__":
    print(f"Python {sys.version.split()[0]}, {os.cpu_count()} logical CPUs")
    for n in (100, 500):
        run(n, 200)  # warm-up
        tps = max(run(n, 3000, seed=s) for s in (1, 2, 3))
        print(f"{n:4d} dead: {tps:8.0f} ticks/s = {tps / TICK_HZ:6.0f}x real time at {TICK_HZ} ticks per game second")
