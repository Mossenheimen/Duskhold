"""The two hot spots a real tick adds: field of view (recursive shadowcasting, radius 20) and A*
(8 directions, no corner cutting) on the same 96x96 map with 12% blocked tiles."""
import heapq, random, time

W = H = 96
rnd = random.Random(1)
BLOCKED = [[rnd.random() < 0.12 for _ in range(W)] for _ in range(H)]


def blk(x, y):
    return not (0 <= x < W and 0 <= y < H) or BLOCKED[y][x]


MULT = ((1, 0, 0, -1, -1, 0, 0, 1), (0, 1, -1, 0, 0, -1, 1, 0), (0, 1, 1, 0, 0, -1, -1, 0), (1, 0, 0, 1, -1, 0, 0, -1))


def cast(cx, cy, row, start, end, radius, xx, xy, yx, yy, seen):
    if start < end:
        return
    r2 = radius * radius
    new_start = start
    for j in range(row, radius + 1):
        dx, dy = -j - 1, -j
        blocked = False
        while dx <= 0:
            dx += 1
            X, Y = cx + dx * xx + dy * xy, cy + dx * yx + dy * yy
            l_slope, r_slope = (dx - 0.5) / (dy + 0.5), (dx + 0.5) / (dy - 0.5)
            if start < r_slope:
                continue
            if end > l_slope:
                break
            if dx * dx + dy * dy < r2 and 0 <= X < W and 0 <= Y < H:
                seen[Y][X] = True
            if blocked:
                if blk(X, Y):
                    new_start = r_slope
                else:
                    blocked = False
                    start = new_start
            elif blk(X, Y) and j < radius:
                blocked = True
                cast(cx, cy, j + 1, start, l_slope, radius, xx, xy, yx, yy, seen)
                new_start = r_slope
        if blocked:
            break


def fov(cx, cy, radius=20):
    seen = [[False] * W for _ in range(H)]
    seen[cy][cx] = True
    for o in range(8):
        cast(cx, cy, 1, 1.0, 0.0, radius, MULT[0][o], MULT[1][o], MULT[2][o], MULT[3][o], seen)
    return seen


STEPS = [(dx, dy, 1.0 if dx == 0 or dy == 0 else 1.4142) for dx in (-1, 0, 1) for dy in (-1, 0, 1) if dx or dy]


def astar(sx, sy, gx, gy):
    def h(x, y):
        dx, dy = abs(x - gx), abs(y - gy)
        return max(dx, dy) + 0.4142 * min(dx, dy)

    best = {(sx, sy): 0.0}
    frontier = [(h(sx, sy), 0.0, sx, sy)]
    while frontier:
        _, g, x, y = heapq.heappop(frontier)
        if (x, y) == (gx, gy):
            return g
        if g > best.get((x, y), 1e9):
            continue
        for dx, dy, c in STEPS:
            nx, ny = x + dx, y + dy
            if blk(nx, ny) or (dx and dy and (blk(x + dx, y) or blk(x, y + dy))):
                continue
            ng = g + c
            if ng < best.get((nx, ny), 1e9):
                best[(nx, ny)] = ng
                heapq.heappush(frontier, (ng + h(nx, ny), ng, nx, ny))
    return None


def free_point():
    while True:
        x, y = rnd.randrange(W), rnd.randrange(H)
        if not BLOCKED[y][x]:
            return x, y


if __name__ == "__main__":
    pts = [free_point() for _ in range(200)]
    t0 = time.perf_counter()
    for x, y in pts:
        fov(x, y)
    fov_ms = (time.perf_counter() - t0) / len(pts) * 1000
    pairs = []
    while len(pairs) < 100:
        (ax, ay), (bx, by) = free_point(), free_point()
        if 30 <= max(abs(ax - bx), abs(ay - by)) <= 50:
            pairs.append((ax, ay, bx, by))
    t0 = time.perf_counter()
    found = sum(astar(*p) is not None for p in pairs)
    astar_ms = (time.perf_counter() - t0) / len(pairs) * 1000
    print(f"field of view, radius 20: {fov_ms:.2f} ms each")
    print(f"A*, 30-50 tiles apart:    {astar_ms:.2f} ms each ({found}/{len(pairs)} paths found)")
