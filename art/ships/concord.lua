-- Free Concord: a free republic of luxury liners turned warships. Manta
-- curves in teal and chrome, warm lit windows, green plasma blasters,
-- drones flying escort, and a garden under glass.
-- Each entry builds one hull class; cmd = true adds the Liberty Star.

local px = dofile("art/lib/px.lua")
local P = dofile("art/lib/palette.lua")
local F = dofile("art/lib/forge.lua")
local M = dofile("art/lib/materials.lua")

local FL = M.flame.concord
local O = P.outline
local PLASMA, LEAF = M.plasma, M.leaf
local SAIL = { "#0f3a3a", "#1a6a62", "#2ea08a", "#5ad2b0", "#a8f4dc" }
local T = {}

local function new(name, w, h)
  local s = F.new(w or M.CLASS[name][1], h or M.CLASS[name][2])
  s.glowRamp = FL
  return s
end

local function windows(s, x0, x1, y, step, m) for x = x0, x1, step or 2 do s:px(x, y, M.window, m) end end

--- A drone flying escort beside the ship, drawn after the outline with its own glow.
local function orbitDrone(s, x, y)
  s:after(function(sh)
    local X, Y = x + sh.ox, y + sh.oy
    local shape = { ".abb.", "cdeef", ".abb." }
    local key = { a = M.chrome[6], b = M.chrome[5], c = M.chrome[3], d = M.chrome[4], e = M.azure[4], f = M.chrome[4] }
    for r, row in ipairs(shape) do
      for i = 1, #row do
        if row:sub(i, i) ~= "." then
          for _, q in ipairs({ { -1, 0 }, { 1, 0 }, { 0, -1 }, { 0, 1 } }) do
            local ox, oy = X + i - 3 + q[1], Y + r - 2 + q[2]
            if not px.opaque(sh.img, ox, oy) then px.set(sh.img, ox, oy, O) end
          end
        end
      end
    end
    for r, row in ipairs(shape) do
      for i = 1, #row do
        local ch = row:sub(i, i)
        if ch ~= "." then px.set(sh.img, X + i - 3, Y + r - 2, key[ch]) end
      end
    end
    px.set(sh.img, X - 3, Y, PLASMA[3]); px.set(sh.img, X - 4, Y, PLASMA[2])
  end)
  s:glow(x + 1, y, 0.7, M.azure)
end

--- Plasma orb (blaster focus) with its bloom.
local function orb(s, x, y, r)
  s:part(PLASMA, { round = true, shadow = 0, tone = 0.1 }, function(m) m:ellipse(x, y, r, r) end)
  s:glow(math.floor(x), math.floor(y), r, PLASMA)
end

--- Liberty Star: a gold star in a gold ring, the flagship's mark.
local function crown(s, x, y, r)
  r = r or 3
  s:part(M.gold, { bevel = 1 }, function(m) m:arc(x, y, r + 1.5, 1.3, -math.pi, math.pi) end)
  local pts = {}
  for i = 0, 9 do
    local a = -math.pi / 2 + i * math.pi / 5
    local rr = (i % 2 == 0) and r or r * 0.45
    pts[#pts + 1] = { x + math.cos(a) * rr, y + math.sin(a) * rr }
  end
  s:part(M.gold, { bevel = 1, shadow = 0, tone = 0.15 }, function(m) m:poly(pts) end)
  s:glow(math.floor(x), math.floor(y), r * 0.6, M.commandGold)
end

-- Interceptor "Dart": a palm-sized manta.
T.interceptor = function(cmd)
  local s = new("interceptor")
  s:part(M.teal, { bevel = 1 }, function(m) m:poly({ { 4, 6 }, { 10, 3 }, { 16, 0 }, { 19, 0 }, { 15, 3 }, { 12, 7 } }, true) end)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(12, 7.5, 10, 2.5) end)
  windows(s, 7, 15, 8, 3)
  orb(s, 22.5, 7.5, 1.5)
  s:glow(18, 0, 0.6, PLASMA, true)
  s:engine(2, 6, 3, 6, M.chrome)
  if cmd then crown(s, 10, 7.5, 2) end
  return s:finish()
end

-- Frigate "Swift": a little manta with a plasma coil for a nose.
T.frigate = function(cmd)
  local s = new("frigate", 38, 23)
  s:part(M.teal, { bevel = 1 }, function(m) m:poly({ { 6, 10 }, { 14, 5 }, { 24, 1 }, { 30, 1 }, { 26, 4 }, { 20, 8 }, { 18, 11 } }, true) end)
  s:part(M.chrome, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 24, 1 }, { 30, 1 }, { 27, 3 }, { 22, 3 } }, true) end)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(19, 11.5, 15, 3.5) end)
  s:part(M.teal, { round = true, shadow = 0 }, function(m) m:ellipse(16, 10.5, 8, 1.5) end)
  windows(s, 10, 22, 12, 3)
  orb(s, 33, 11.5, 2.2)
  s:glow(29, 1, 0.8, PLASMA, true)
  s:engine(4, 10, 3, 9, M.chrome)
  if cmd then crown(s, 14, 11.5, 2.5) end
  return s:finish()
end

-- Destroyer "Heron": a slim hull under long curved wings, one drone in attendance.
T.destroyer = function(cmd)
  local s = new("destroyer")
  s:part(M.teal, { bevel = 1 }, function(m)
    m:arc(26, 12.5, 12, function(t) return 1 + 2.5 * math.sin(math.pi * t) end, 1.6, 2.9, true)
  end)
  s:part(M.chrome, { bevel = 1, shadow = 0 }, function(m) m:arc(26, 12.5, 12, 1, 1.65, 2.85, true) end)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(22, 12.5, 20, 3.2) end)
  s:part(M.teal, { round = true, shadow = 0 }, function(m) m:ellipse(20, 11.5, 12, 1.4) end)
  windows(s, 8, 34, 13, 2)
  orb(s, 42.5, 12.5, 2)
  orbitDrone(s, 40, 3)
  s:engine(3, 11, 3, 9, M.chrome)
  if cmd then crown(s, 22, 12.5, 2.5) end
  return s:finish()
end

-- Cruiser "Lyre": lyre arms cradling a plasma orb, drones in attendance.
T.cruiser = function(cmd)
  local s = new("cruiser", 60, 33)
  s:part(M.teal, { bevel = 2 }, function(m) m:arc(30, 16.5, 16, function(t) return 1.5 + 3.5 * math.sin(math.pi * (0.15 + t * 0.85)) end, 0.4, 2.75, true) end)
  s:part(M.chrome, { bevel = 1, shadow = 0 }, function(m) m:arc(30, 16.5, 16, 1, 0.45, 2.7, true) end)
  windows(s, 24, 36, 3, 3, true)
  s:part(M.chrome, { bevel = 1 }, function(m) m:bar(24, 13, 22, 3, 1.3, true); m:bar(36, 13, 38, 4, 1.1, true) end)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(26, 16.5, 20, 5) end)
  s:part(M.teal, { round = true, shadow = 0 }, function(m) m:ellipse(24, 15.5, 13, 2.2) end)
  windows(s, 10, 38, 18, 2)
  orb(s, 49, 16.5, 3.5)
  s:glow(43, 7, 1, PLASMA, true)
  orbitDrone(s, 56, 4); orbitDrone(s, 12, 2); orbitDrone(s, 54, 29)
  s:engine(6, 14, 5, 12, M.chrome)
  if cmd then crown(s, 22, 16.5, 3) end
  return s:finish()
end

-- Battlecruiser "Regatta": a racing yacht under glowing solar sails.
T.battlecruiser = function(cmd)
  local s = new("battlecruiser")
  s:part(SAIL, { bevel = 1, tone = 0.05 }, function(m) m:poly({ { 12, 14 }, { 22, 2 }, { 44, 0 }, { 40, 13 } }, true) end)
  for _, x in ipairs({ 22, 30, 38 }) do s:line(x - 6, 13, x + 1, 1, M.chrome[4], true) end
  s:part(M.chrome, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 22, 2 }, { 44, 0 }, { 44, 1.5 }, { 23, 3.5 } }, true) end)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(34, 17.5, 30, 5) end)
  s:part(M.teal, { round = true, shadow = 0 }, function(m) m:ellipse(30, 16.5, 20, 2.2) end)
  windows(s, 10, 54, 19, 2); windows(s, 14, 50, 20, 4)
  s:turret("blaster", 48, 12, M.chrome, 1, true, PLASMA)
  orb(s, 65, 17.5, 2.5)
  orbitDrone(s, 60, 5); orbitDrone(s, 64, 30)
  s:engine(4, 15, 5, 13, M.chrome)
  if cmd then crown(s, 26, 17.5, 3) end
  return s:finish()
end

-- Battleship "Aurora": a great manta with aurora-lit wings and plasma wingtips.
T.battleship = function(cmd)
  local s = new("battleship", 88, 45)
  s:part(M.teal, { bevel = 2 }, function(m) m:poly({ { 8, 19 }, { 24, 10 }, { 42, 2 }, { 56, 0 }, { 62, 2 }, { 52, 8 }, { 44, 16 }, { 40, 20 } }, true) end)
  s:part(M.chrome, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 8, 19 }, { 24, 10 }, { 42, 2 }, { 44, 3.5 }, { 26, 11.5 }, { 10, 20 } }, true) end)
  local AUR = { PLASMA[3], M.cyan[3], M.azure[3], "#b38aff" }
  for i = 0, 22 do s:px(22 + i, 14 - math.floor(i * 0.45), AUR[(i // 6) % 4 + 1], true) end
  windows(s, 30, 48, 9, 3, true)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(44, 22.5, 40, 6) end)
  s:part(M.teal, { round = true, shadow = 0 }, function(m) m:ellipse(40, 21.5, 26, 2.5) end)
  windows(s, 12, 70, 24, 2); windows(s, 18, 64, 26, 3)
  s:turret("blaster", 54, 3, M.chrome, 2, true, PLASMA)
  orb(s, 82, 22.5, 2.8)
  orbitDrone(s, 72, 9); orbitDrone(s, 76, 36); orbitDrone(s, 66, 40)
  s:engine(6, 19, 7, 15, M.chrome)
  if cmd then crown(s, 36, 22.5, 3.5) end
  return s:finish()
end

-- Carrier "Grove": a carrier built around a garden under glass.
T.carrier = function(cmd)
  local s = new("carrier", 126, 57)
  s:part(M.teal, { bevel = 2 }, function(m) m:arc(62, 28.5, 28.5, function(t) return 2 + 5 * math.sin(math.pi * t) end, 1.15, 2.85, true) end)
  s:part(M.chrome, { bevel = 1, shadow = 0 }, function(m) m:arc(62, 28.5, 28.5, 1, 1.2, 2.8, true) end)
  s:part(M.teal, { round = true, radius = 9 }, function(m) m:ellipse(60, 28.5, 50, 16) end)
  windows(s, 20, 100, 16, 3); windows(s, 22, 98, 41, 3)
  for _, y in ipairs({ 18, 37 }) do
    s:part(M.chrome, { bevel = 1, shadow = 0 }, function(m) m:rect(76, y, 26, 3) end)
    s:dots(78, 100, y + 1, M.azure[3], 4)
  end
  s:part(M.chrome, { round = true }, function(m) m:ellipse(54, 28.5, 21, 12.5) end)
  s:part(LEAF, { round = true, shadow = 0, tone = -0.15 }, function(m) m:ellipse(54, 28.5, 19, 10.5) end)
  local rnd = px.rng(5)
  for _ = 1, 22 do
    local tx, ty = rnd(38, 70), rnd(21, 36)
    local dx, dy = (tx - 54) / 17, (ty - 28.5) / 9
    if dx * dx + dy * dy < 1 then
      s:part(LEAF, { round = true, shadow = 1 }, function(m) m:ellipse(tx, ty, rnd(15, 30) / 10, rnd(12, 24) / 10) end)
      if rnd(1, 4) == 1 then s:px(tx, ty, "#ffb0d0") end
    end
  end
  s:hline(40, 68, 28, "#5a8acc"); s:hline(42, 66, 29, "#3a6aac")
  s:line(42, 21, 48, 18, "#e8fbff"); s:line(44, 23, 47, 21, "#a6dcff")
  s:part(M.chrome, { round = true }, function(m) m:ellipse(102, 28.5, 22, 5) end)
  windows(s, 88, 114, 29, 2)
  orb(s, 120, 28.5, 2.8)
  s:turret("blaster", 94, 9, M.chrome, 2, true, PLASMA)
  orbitDrone(s, 112, 15); orbitDrone(s, 118, 40); orbitDrone(s, 106, 46); orbitDrone(s, 122, 10)
  s:engine(10, 20, 6, 15, M.chrome, true); s:engine(10, 26, 5, 13, M.chrome)
  if cmd then crown(s, 96, 28.5, 3.5) end
  return s:finish()
end

-- Dreadnought "Liberty": twin liner hulls carrying a plasma accelerator between them.
T.dreadnought = function(cmd)
  local s = new("dreadnought")
  -- the deck joining the hulls
  s:part(M.teal, { round = true, radius = 6 }, function(m) m:poly({ { 12, 14 }, { 80, 13 }, { 92, 25.5 }, { 80, 38 }, { 12, 37 } }) end)
  local AUR = { PLASMA[3], M.cyan[3], M.azure[3], "#b38aff" }
  for i = 0, 40 do s:px(20 + i, 17, AUR[(i // 8) % 4 + 1], true) end
  -- twin hulls
  for _, y in ipairs({ 9.5, 41.5 }) do
    s:part(M.chrome, { round = true }, function(m) m:ellipse(54, y, 50, 5.5) end)
    s:part(M.teal, { round = true, shadow = 0 }, function(m) m:ellipse(50, y - 1, 34, 1.8) end)
    windows(s, 12, 94, math.floor(y) + 1, 2)
    orb(s, 104, y, 2.2)
  end
  -- accelerator spine with coils
  s:part(M.chrome, { round = true, radius = 2 }, function(m) m:rect(18, 23.5, 86, 4) end)
  for x = 36, 88, 13 do
    s:part(M.chrome, { round = true }, function(m) m:ellipse(x, 25.5, 2, 6) end)
    s:px(x, 25, PLASMA[4]); s:glow(x, 25, 1.5, PLASMA)
  end
  orb(s, 107, 25.5, 4.5)
  s:turret("blaster", 30, 9.5, M.chrome, 2, true, PLASMA)
  orbitDrone(s, 98, 18); orbitDrone(s, 98, 33); orbitDrone(s, 84, 2)
  s:engine(4, 7, 5, 14, M.chrome, true); s:engine(12, 23, 5, 14, M.chrome)
  if cmd then crown(s, 24, 25.5, 4) end
  return s:finish()
end

-- Titan "Harmony": a lit ring around a plasma heart, pierced by a liner hull.
T.titan = function(cmd)
  local s = new("titan")
  local cx, cy = 58, 27.5
  -- the ring
  s:part(M.teal, { round = true, radius = 3 }, function(m)
    m:ellipse(cx, cy, 20, 27.5):ellipse(cx, cy, 15, 22.5, false, true)
  end)
  for i = 0, 15 do
    local a = i * math.pi / 8
    s:px(math.floor(cx + math.cos(a) * 17.5), math.floor(cy + math.sin(a) * 25), M.window)
  end
  -- spokes
  s:part(M.chrome, { bevel = 1 }, function(m) m:bar(cx, cy - 6, cx, 4, 1.4, true); m:bar(cx - 6, cy - 4, cx - 14, 8, 1.1, true); m:bar(cx + 6, cy - 4, cx + 14, 8, 1.1, true) end)
  -- the liner hull through it
  s:part(M.teal, { bevel = 2 }, function(m) m:poly({ { 6, 22 }, { 22, 14 }, { 40, 12 }, { 36, 22 } }, true) end)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(64, cy, 58, 7) end)
  s:part(M.teal, { round = true, shadow = 0 }, function(m) m:ellipse(60, cy - 1, 42, 2.5) end)
  windows(s, 12, 112, 29, 2); windows(s, 16, 108, 31, 3)
  -- plasma heart
  s:part(PLASMA, { round = true, shadow = 0, tone = 0.15 }, function(m) m:ellipse(cx, cy, 8, 8) end)
  s:part(M.chrome, { bevel = 1, shadow = 0 }, function(m) m:arc(cx, cy, 9.5, 1.5, -math.pi, math.pi) end)
  s:glow(cx, math.floor(cy), 6, PLASMA)
  orb(s, 120, cy, 3.5)
  for _, x in ipairs({ 84, 98 }) do s:turret("blaster", x, cy - 9, M.chrome, 2, true, PLASMA) end
  orbitDrone(s, 92, 3); orbitDrone(s, 108, 48); orbitDrone(s, 30, 4); orbitDrone(s, 116, 12)
  s:engine(7, 22, 11, 20, M.chrome)
  if cmd then crown(s, 88, cy, 4) end
  return s:finish()
end

-- ---------------------------------------------------------------------------
-- Logistics: nanite clouds, repair drones and a lotus in bloom.
-- ---------------------------------------------------------------------------

--- A drift of glowing nanites around a point (drawn after the outline).
local function nanites(s, cx, cy, r, n, seed)
  local rnd = px.rng(seed)
  for _ = 1, n do
    local a = rnd() * math.pi * 2
    local d = r * (0.6 + rnd() * 0.5)
    s:glow(math.floor(cx + math.cos(a) * d), math.floor(cy + math.sin(a) * d * 0.7), 0.6, M.mend)
  end
end

--- Mend orb: the green core every Concord healer carries.
local function mendOrb(s, x, y, r)
  s:part(M.mend, { round = true, shadow = 0, tone = 0.1 }, function(m) m:ellipse(x, y, r, r) end)
  s:glow(math.floor(x), math.floor(y), r, M.mend)
end

-- Logi Frigate "Grace": a manta trailing a cloud of nanites.
T.logi_frigate = function()
  local s = new("logi_frigate")
  s:part(M.teal, { bevel = 1 }, function(m) m:poly({ { 6, 9 }, { 12, 5 }, { 20, 1 }, { 25, 1 }, { 21, 4 }, { 17, 7 }, { 16, 10 } }, true) end)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(18, 10.5, 15, 3.5) end)
  s:part(M.teal, { round = true, shadow = 0 }, function(m) m:ellipse(15, 9.5, 7, 1.4) end)
  windows(s, 9, 21, 11, 3)
  mendOrb(s, 32.5, 10.5, 2.2)
  s:part(M.mend, { bevel = 1, shadow = 0 }, function(m) m:arc(18, 10.5, 7, 1, 1.8, 2.9, true) end)
  nanites(s, 22, 10.5, 10, 6, 3)
  s:engine(3, 9, 3, 9, M.chrome)
  return s:finish()
end

-- Logi Cruiser "Mercy": a teardrop with a ring of repair drones about it.
T.logi_cruiser = function()
  local s = new("logi_cruiser")
  s:part(M.teal, { bevel = 2 }, function(m) m:arc(28, 15.5, 15, function(t) return 1.5 + 3 * math.sin(math.pi * t) end, 1.0, 2.8, true) end)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(28, 15.5, 24, 5.5) end)
  s:part(M.teal, { round = true, shadow = 0 }, function(m) m:ellipse(26, 14.5, 15, 2.2) end)
  windows(s, 10, 44, 17, 2)
  s:part(M.chrome, { bevel = 1 }, function(m) m:ellipse(32, 15.5, 5, 8):ellipse(32, 15.5, 3, 6, false, true) end)
  mendOrb(s, 32, 15.5, 2.5)
  mendOrb(s, 52.5, 15.5, 2.4)
  orbitDrone(s, 46, 3); orbitDrone(s, 14, 2); orbitDrone(s, 48, 28); orbitDrone(s, 18, 29)
  s:engine(4, 13, 5, 11, M.chrome)
  return s:finish()
end

-- Force Auxiliary "Sanctuary": a lotus of chrome petals opening around a mend core.
T.fax = function()
  local s = new("fax")
  local cx, cy = 58, 25.5
  -- outer then inner petals
  for _, p in ipairs({ { 25, 0.25, 1.15 }, { 25, 1.3, 2.2 }, { 25, 2.3, 3.0 } }) do
    s:part(M.teal, { bevel = 2 }, function(m) m:arc(cx, cy, p[1], function(t) return 1 + 6 * math.sin(math.pi * t) end, p[2], p[3], true) end)
  end
  for _, p in ipairs({ { 16, 0.6, 1.5 }, { 16, 1.65, 2.6 } }) do
    s:part(M.chrome, { bevel = 1 }, function(m) m:arc(cx, cy, p[1], function(t) return 1 + 4 * math.sin(math.pi * t) end, p[2], p[3], true) end)
  end
  s:part(M.chrome, { round = true }, function(m) m:ellipse(48, cy, 42, 6) end)
  s:part(M.teal, { round = true, shadow = 0 }, function(m) m:ellipse(44, cy - 1, 28, 2.5) end)
  windows(s, 12, 86, 27, 2)
  s:part(M.chrome, { bevel = 1, shadow = 0 }, function(m) m:arc(cx, cy, 10, 1.5, -math.pi, math.pi) end)
  mendOrb(s, cx, cy, 7.5)
  mendOrb(s, 92, cy, 3)
  nanites(s, cx, cy, 30, 12, 9)
  orbitDrone(s, 96, 8); orbitDrone(s, 100, 42); orbitDrone(s, 30, 4); orbitDrone(s, 34, 47)
  s:engine(6, 21, 9, 18, M.chrome)
  return s:finish()
end

return T
