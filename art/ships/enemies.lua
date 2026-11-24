-- Enemy hulls, five tiers per faction (1 skirmisher ... 5 boss). Drawn nose
-- right like the player's ships; the game mirrors them to face left.
--   Crimson Raiders  - pirates: blackened iron, blood paint, spikes, skulls.
--   Verdant Swarm    - living ships: carapace, legs, glowing sacs.
--   Void Cult        - crescent blades around burning magenta eyes.
--   Iron Synod       - gothic machines: chrome, red optics, cables.

local px = dofile("art/lib/px.lua")
local P = dofile("art/lib/palette.lua")
local F = dofile("art/lib/forge.lua")
local M = dofile("art/lib/materials.lua")

local O = P.outline
local SIZE = { { 22, 13 }, { 30, 17 }, { 42, 23 }, { 58, 31 }, { 100, 49 } }

local BLOOD = { "#1a0a0c", "#3a1016", "#64161e", "#962028", "#c8363a", "#ff6a5a" }
local IRON = { "#0e0d12", "#1c1a22", "#2e2a36", "#46404e", "#6a6270", "#948a98" }
local FIRE = { "#5a1406", "#c2401a", "#ff8a1c", "#ffd27a", "#ffffff" }
local SHELL = { "#0f1a0c", "#1e3216", "#33521f", "#4f7a2a", "#78a83a", "#b2d86a" }
local SAC = { "#3f5f0c", "#93c51d", "#d8ff5c", "#f6ffd0", "#ffffff" }
local BELLY = { "#2a2010", "#4a3a1c", "#6e5a2c", "#9a8448", "#c8b478" }
local WING = { "#1a3020", "#2a5a40", "#4a8a6a", "#8ac8a8" }
local VOID = { "#120c1e", "#22163a", "#3a2660", "#5a3c8c", "#8460b8", "#b9a0e0" }
local OBSID = { "#08060c", "#141020", "#221a32", "#342848", "#4a3a64" }
local EYE = { "#6a0f5a", "#c4248c", "#ff6ad8", "#ffd0f4", "#ffffff" }
local GUN = { "#141820", "#232a36", "#384252", "#56627a", "#8090a8", "#c0cad8" }
local OPTIC = { "#4a0b14", "#a3172b", "#ff4a5a", "#ffc2c2", "#ffffff" }

local FLAMES = {
  crimson = { "#c2401a", "#ff8a3c", "#ffd27a", "#ffffff" },
  verdant = { "#3f8a1a", "#93e04a", "#e0ffa0", "#ffffff" },
  void = { "#7a1aa0", "#c43dff", "#ff86f0", "#ffffff" },
  synod = { "#a3172b", "#ff4a5a", "#ffc2c2", "#ffffff" },
}

local function new(f, t)
  local s = F.new(SIZE[t][1], SIZE[t][2])
  s.glowRamp = FLAMES[f]
  return s
end

local E = { crimson = {}, verdant = {}, void = {}, synod = {} }

-- ---------------------------------------------------------------------------
-- Crimson Raiders
-- ---------------------------------------------------------------------------

local function spikes(s, x0, x1, y, len, step, m, ramp)
  for x = x0, x1, step do
    s:part(ramp or IRON, { bevel = 1, shadow = 0 }, function(mk) mk:poly({ { x, y }, { x + 3, y }, { x - 2, y - len } }, m) end)
  end
end

local function skull(s, x, y)
  s:map(x, y, { ".###.", "#o#o#", "#####", ".#.#." }, { ["#"] = M.bone[4], o = O })
end

E.crimson[1] = function()
  local s = new("crimson", 1)
  s:part(IRON, { bevel = 1 }, function(m) m:poly({ { 2, 3 }, { 12, 4 }, { 22, 6.5 }, { 12, 9 }, { 2, 10 }, { 5, 6.5 } }) end)
  s:part(BLOOD, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 10, 4.6 }, { 22, 6.5 }, { 10, 8.4 }, { 12, 6.5 } }) end)
  spikes(s, 6, 6, 4, 4, 4)
  s:px(15, 6, FIRE[4]); s:glow(15, 6, 0.8, FIRE)
  s:engine(3, 5, 3, 5, IRON)
  return s:finish()
end

E.crimson[2] = function()
  local s = new("crimson", 2)
  -- hooked mandibles
  s:part(IRON, { bevel = 1 }, function(m) m:poly({ { 14, 5 }, { 24, 2 }, { 30, 4 }, { 26, 5 }, { 28, 7 }, { 22, 6 }, { 16, 7.5 } }, true) end)
  s:part(BLOOD, { bevel = 1 }, function(m) m:poly({ { 2, 4 }, { 16, 5 }, { 24, 8.5 }, { 16, 12 }, { 2, 13 }, { 6, 8.5 } }) end)
  spikes(s, 5, 11, 5, 4, 6)
  s:line(8, 6, 12, 11, IRON[2])
  s:px(18, 8, FIRE[4]); s:glow(18, 8, 1, FIRE)
  s:engine(4, 7, 3, 6, IRON)
  return s:finish()
end

E.crimson[3] = function()
  local s = new("crimson", 3)
  -- cleaver prow
  s:part(IRON, { bevel = 2 }, function(m) m:poly({ { 26, 3 }, { 36, 4 }, { 42, 11.5 }, { 36, 14 }, { 28, 13 } }) end)
  s:line(30, 5, 40, 11, IRON[6])
  s:part(BLOOD, { bevel = 2 }, function(m) m:poly({ { 2, 5 }, { 28, 6 }, { 34, 11.5 }, { 28, 17 }, { 2, 18 }, { 7, 11.5 } }) end)
  spikes(s, 6, 22, 6, 5, 5)
  spikes(s, 8, 20, 17, -4, 6, false, BLOOD)
  skull(s, 14, 9)
  s:turret("auto", 26, 17, IRON, 1)
  s:px(31, 11, FIRE[4]); s:glow(31, 11, 1, FIRE)
  s:engine(4, 8, 3, 7, IRON, true)
  return s:finish()
end

E.crimson[4] = function()
  local s = new("crimson", 4)
  s:part(IRON, { bevel = 2 }, function(m) m:poly({ { 14, 2 }, { 34, 4 }, { 26, 10 } }, true) end)
  s:part(BLOOD, { bevel = 2 }, function(m) m:poly({ { 2, 8 }, { 36, 7 }, { 50, 11 }, { 58, 15.5 }, { 50, 20 }, { 36, 24 }, { 2, 23 }, { 8, 15.5 } }) end)
  s:part(IRON, { bevel = 2, shadow = 0 }, function(m) m:poly({ { 40, 9 }, { 58, 15.5 }, { 40, 22 }, { 44, 15.5 } }) end)
  spikes(s, 6, 34, 8, 5, 5)
  spikes(s, 8, 32, 23, -5, 6, false, BLOOD)
  skull(s, 22, 13)
  for x = 10, 34, 6 do s:px(x, 19, FIRE[3]); s:glow(x, 19, 0.6, FIRE) end
  s:turret("auto", 36, 12, IRON, 2); s:turret("auto", 30, 19, IRON, 1)
  s:px(50, 15, FIRE[5]); s:glow(50, 15, 1.5, FIRE)
  s:engine(4, 10, 4, 9, IRON, true)
  return s:finish()
end

E.crimson[5] = function()
  local s = new("crimson", 5)
  -- crown of spikes
  for i = 0, 6 do
    local x = 16 + i * 8
    s:part(IRON, { bevel = 1 }, function(m) m:poly({ { x, 14 }, { x + 6, 14 }, { x - 4 + i, 1 + math.abs(3 - i) * 2 } }, true) end)
  end
  s:part(BLOOD, { bevel = 3 }, function(m) m:poly({ { 4, 12 }, { 60, 11 }, { 84, 18 }, { 100, 24.5 }, { 84, 31 }, { 60, 38 }, { 4, 37 }, { 14, 24.5 } }) end)
  s:part(IRON, { bevel = 2, shadow = 0 }, function(m) m:poly({ { 20, 17 }, { 60, 16 }, { 72, 24.5 }, { 60, 33 }, { 20, 32 } }) end)
  -- skull figurehead
  s:part(M.bone, { round = true }, function(m) m:ellipse(88, 24.5, 7, 6) end)
  s:rect(85, 22, 2, 2, O); s:rect(89, 22, 2, 2, O); s:hline(85, 91, 27, O); s:dots(85, 91, 28, M.bone[3], 2)
  s:glow(85, 22, 0.8, FIRE); s:glow(89, 22, 0.8, FIRE)
  for x = 24, 56, 8 do s:px(x, 24, FIRE[3]); s:glow(x, 24, 1.2, FIRE) end
  for _, p in ipairs({ { 30, 19 }, { 46, 19 }, { 62, 22 } }) do s:turret("auto", p[1], p[2], IRON, 2, true) end
  s:engine(6, 15, 6, 14, IRON, true); s:engine(12, 22, 5, 12, IRON)
  return s:finish()
end

-- ---------------------------------------------------------------------------
-- Verdant Swarm
-- ---------------------------------------------------------------------------

local function legs(s, xs, y0, y1, m)
  for _, x in ipairs(xs) do
    s:part(SHELL, { bevel = 1, tone = -0.2, shadow = 0 }, function(mk) mk:bar(x, y0, x - 3, y1, 0.8, m) end)
  end
end

local function segment(s, cx, cy, rx, ry, tone)
  s:part(SHELL, { round = true, tone = tone or 0 }, function(m) m:ellipse(cx, cy, rx, ry) end)
end

local function sac(s, x, y, r)
  s:part(SAC, { round = true, shadow = 0 }, function(m) m:ellipse(x, y, r, r * 0.8) end)
  s:glow(math.floor(x), math.floor(y), r, SAC)
end

E.verdant[1] = function()
  local s = new("verdant", 1)
  legs(s, { 9, 13 }, 3, 0, true)
  segment(s, 9, 6.5, 7, 4.5)
  segment(s, 17, 6.5, 4.5, 3.5, 0.08)
  s:px(20, 5, SAC[3]); s:px(20, 8, SAC[3])
  sac(s, 8, 6.5, 1.5)
  s:engine(2, 5, 3, 4, BELLY)
  return s:finish()
end

E.verdant[2] = function()
  local s = new("verdant", 2)
  -- translucent wings
  s:part(WING, { bevel = 1, tone = 0.1 }, function(m) m:ellipse(12, 3.5, 7, 3, true) end)
  s:line(7, 3, 17, 4, WING[4], true)
  segment(s, 7, 8.5, 6, 4)
  segment(s, 15, 8.5, 4, 3.5, 0.06)
  segment(s, 22, 8.5, 3.5, 3, 0.1)
  -- forward stinger
  s:part(BELLY, { bevel = 1 }, function(m) m:poly({ { 24, 7.5 }, { 30, 8.5 }, { 24, 9.5 } }) end)
  for x = 4, 10, 3 do s:px(x, 7, SHELL[1]) end
  sac(s, 7, 9, 1.8)
  s:engine(1, 7, 3, 5, BELLY)
  return s:finish()
end

E.verdant[3] = function()
  local s = new("verdant", 3)
  legs(s, { 12, 18, 24 }, 6, 0, true)
  -- the shell, split down the middle
  s:part(SHELL, { round = true }, function(m) m:ellipse(18, 11.5, 15, 9) end)
  s:hline(5, 31, 11, SHELL[1])
  for x = 9, 27, 6 do s:px(x, 7, SHELL[6]); s:px(x + 1, 15, SHELL[5]) end
  segment(s, 33, 11.5, 5, 5, 0.06)
  -- horn
  s:part(BELLY, { bevel = 1 }, function(m) m:poly({ { 34, 9 }, { 42, 5 }, { 40, 8.5 }, { 37, 12 } }) end)
  s:px(36, 10, SAC[4]); s:px(36, 13, SAC[4])
  sac(s, 12, 11.5, 2); sac(s, 22, 11.5, 2)
  s:engine(3, 10, 3, 6, BELLY)
  return s:finish()
end

E.verdant[4] = function()
  local s = new("verdant", 4)
  legs(s, { 16, 22, 28 }, 10, 2, true)
  s:part(WING, { bevel = 1, tone = 0.1 }, function(m) m:ellipse(20, 5, 12, 3.5, true) end)
  s:line(10, 5, 30, 6, WING[4], true)
  for i = 0, 4 do segment(s, 6 + i * 8, 15.5, 6 - i * 0.5, 6 - i * 0.6, i * 0.03) end
  -- head and raptorial forelimbs
  segment(s, 44, 15.5, 5, 4.5, 0.08)
  s:part(SHELL, { bevel = 1 }, function(m) m:poly({ { 40, 12 }, { 50, 6 }, { 58, 9 }, { 52, 9 }, { 46, 13 } }, true) end)
  s:map(52, 8, { "#.#" }, { ["#"] = M.bone[4] }, true)
  s:px(47, 13, SAC[4]); s:px(47, 18, SAC[4]); s:glow(47, 13, 0.8, SAC); s:glow(47, 18, 0.8, SAC)
  sac(s, 8, 15.5, 2.5); sac(s, 22, 15.5, 2)
  s:engine(1, 13, 5, 7, BELLY)
  return s:finish()
end

E.verdant[5] = function()
  local s = new("verdant", 5)
  legs(s, { 30, 40, 50, 60 }, 14, 1, true)
  -- bloated abdomen full of eggs
  s:part(BELLY, { round = true, radius = 10 }, function(m) m:ellipse(30, 24.5, 28, 18) end)
  for _, p in ipairs({ { 16, 18 }, { 26, 14 }, { 36, 18 }, { 20, 28 }, { 32, 30 }, { 42, 25 }, { 12, 26 }, { 26, 22 } }) do
    sac(s, p[1], p[2], 2.6)
  end
  for i = 0, 3 do s:hline(8, 52, 12 + i * 9, BELLY[2]) end
  -- thorax and head
  segment(s, 62, 24.5, 12, 11)
  segment(s, 80, 24.5, 9, 8, 0.06)
  -- horn crown
  for i = 0, 2 do
    s:part(BELLY, { bevel = 1 }, function(m) m:poly({ { 78 + i * 4, 18 - i }, { 82 + i * 4, 18 - i }, { 86 + i * 5, 8 - i * 2 } }, true) end)
  end
  s:part(SHELL, { bevel = 1 }, function(m) m:poly({ { 86, 22 }, { 100, 20 }, { 96, 24.5 } }, true) end)
  s:px(84, 21, SAC[5]); s:px(84, 27, SAC[5]); s:glow(84, 21, 1.2, SAC); s:glow(84, 27, 1.2, SAC)
  s:engine(3, 18, 13, 10, BELLY)
  return s:finish()
end

-- ---------------------------------------------------------------------------
-- Void Cult
-- ---------------------------------------------------------------------------

local function eye(s, x, y, r)
  s:part(OBSID, { round = true }, function(m) m:ellipse(x, y, r + 1, r + 1) end)
  s:part(EYE, { round = true, shadow = 0, tone = 0.15 }, function(m) m:ellipse(x, y, r, r * 0.75) end)
  s:part(OBSID, { flat = 1, shadow = 0 }, function(m) m:ellipse(x, y, math.max(0.6, r * 0.25), r * 0.6) end)
  s:glow(math.floor(x), math.floor(y), r + 0.5, EYE)
end

local function blade(s, cx, cy, r, th, a0, a1, ramp)
  s:part(ramp or VOID, { bevel = 1 }, function(m)
    m:arc(cx, cy, r, function(t) return 0.8 + th * math.sin(math.pi * t) end, a0, a1, true)
  end)
end

local function runes(s, cx, cy, r, n, a0, a1)
  for i = 0, n - 1 do
    local a = a0 + (a1 - a0) * (i + 0.5) / n
    s:px(math.floor(cx + math.cos(a) * r), math.floor(cy - math.sin(a) * r), EYE[3], true)
  end
end

E.void[1] = function()
  local s = new("void", 1)
  blade(s, 10, 6.5, 6.5, 2, 0.6, 3.1)
  eye(s, 13, 6.5, 2)
  s:engine(4, 6, 1, 4, OBSID)
  return s:finish()
end

E.void[2] = function()
  local s = new("void", 2)
  blade(s, 12, 8.5, 8.5, 3, 0.4, 2.9)
  s:part(OBSID, { bevel = 1 }, function(m) m:poly({ { 2, 7.5 }, { 26, 7.5 }, { 30, 8.5 }, { 26, 9.5 }, { 2, 9.5 } }) end)
  runes(s, 12, 8.5, 7, 3, 0.8, 2.6)
  eye(s, 18, 8.5, 2.2)
  s:engine(2, 8, 1, 5, OBSID)
  return s:finish()
end

E.void[3] = function()
  local s = new("void", 3)
  blade(s, 20, 11.5, 11.5, 3.5, 0.35, 2.6)
  blade(s, 26, 11.5, 7, 2, 0.3, 2.2, OBSID)
  s:part(OBSID, { bevel = 1 }, function(m) m:poly({ { 2, 10 }, { 34, 10 }, { 42, 11.5 }, { 34, 13 }, { 2, 13 } }) end)
  runes(s, 20, 11.5, 9.5, 5, 0.6, 2.4)
  eye(s, 24, 11.5, 3)
  s:engine(2, 10, 3, 6, OBSID)
  return s:finish()
end

E.void[4] = function()
  local s = new("void", 4)
  -- thorned ring
  s:part(VOID, { bevel = 2 }, function(m) m:ellipse(30, 15.5, 15, 15):ellipse(30, 15.5, 11, 11, false, true) end)
  for i = 0, 7 do
    local a = i * math.pi / 4 + 0.4
    local x, y = 30 + math.cos(a) * 14, 15.5 - math.sin(a) * 14
    s:part(OBSID, { bevel = 1, shadow = 0 }, function(m) m:poly({ { x - 1.5, y }, { x + 1.5, y }, { 30 + math.cos(a) * 18, 15.5 - math.sin(a) * 17 } }) end)
  end
  runes(s, 30, 15.5, 13, 10, 0, math.pi * 2 - 0.1)
  blade(s, 40, 15.5, 10, 2.5, 0.2, 1.4)
  s:part(OBSID, { bevel = 1 }, function(m) m:poly({ { 4, 14 }, { 50, 14 }, { 58, 15.5 }, { 50, 17 }, { 4, 17 } }) end)
  eye(s, 30, 15.5, 5)
  s:engine(4, 14, 3, 8, OBSID)
  return s:finish()
end

E.void[5] = function()
  local s = new("void", 5)
  -- orbiting blades
  blade(s, 44, 24.5, 24, 5, 0.3, 2.7)
  blade(s, 52, 24.5, 17, 3, 0.2, 2.2, OBSID)
  -- the eclipse: a dark sphere with a burning corona
  s:part(EYE, { round = true, shadow = 0 }, function(m) m:ellipse(48, 24.5, 17, 17) end)
  s:part(OBSID, { round = true, shadow = 0 }, function(m) m:ellipse(49, 24.5, 15, 15) end)
  runes(s, 48, 24.5, 16, 14, 0, math.pi * 2 - 0.1)
  s:part(OBSID, { bevel = 1 }, function(m) m:poly({ { 4, 22 }, { 30, 21 }, { 30, 28 }, { 4, 27 } }) end)
  s:part(VOID, { bevel = 1 }, function(m) m:poly({ { 64, 23 }, { 96, 23.5 }, { 100, 24.5 }, { 96, 25.5 }, { 64, 26 } }) end)
  eye(s, 52, 24.5, 8)
  s:engine(4, 22, 5, 12, OBSID)
  return s:finish()
end

-- ---------------------------------------------------------------------------
-- Iron Synod
-- ---------------------------------------------------------------------------

local function optic(s, x, y, r)
  s:part(GUN, { round = true }, function(m) m:ellipse(x, y, r + 1, r + 1) end)
  s:part(OPTIC, { round = true, shadow = 0, tone = 0.1 }, function(m) m:ellipse(x, y, r, r) end)
  s:glow(math.floor(x), math.floor(y), r + 0.5, OPTIC)
end

local function slits(s, x0, x1, y, step, m)
  for x = x0, x1, step or 3 do s:px(x, y, OPTIC[3], m); s:px(x, y + 1, OPTIC[2], m) end
end

local function cables(s, x, y, n, len)
  for i = 0, n - 1 do
    local yy = y + (i - (n - 1) / 2) * 3
    s:part(GUN, { round = true, shadow = 0 }, function(m)
      for k = 0, len, 2 do m:ellipse(x - k, yy + math.sin(k * 0.35 + i) * 2, 1, 1) end
    end)
  end
end

E.synod[1] = function()
  local s = new("synod", 1)
  s:part(M.chrome, { bevel = 1 }, function(m) m:poly({ { 2, 6.5 }, { 11, 1 }, { 20, 6.5 }, { 11, 12 } }) end)
  s:part(GUN, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 6, 6.5 }, { 11, 3.5 }, { 16, 6.5 }, { 11, 9.5 } }) end)
  optic(s, 12, 6.5, 1.5)
  s:engine(3, 6, 1, 4, GUN)
  return s:finish()
end

E.synod[2] = function()
  local s = new("synod", 2)
  s:part(GUN, { bevel = 1 }, function(m) m:poly({ { 4, 7 }, { 12, 7 }, { 8, 1 }, { 4, 1 } }, true) end)
  s:part(M.chrome, { bevel = 1 }, function(m) m:poly({ { 2, 6 }, { 18, 6 }, { 30, 8.5 }, { 18, 11 }, { 2, 11 } }) end)
  slits(s, 6, 14, 7, 3)
  optic(s, 20, 8.5, 1.8)
  s:engine(2, 7, 3, 6, GUN)
  return s:finish()
end

E.synod[3] = function()
  local s = new("synod", 3)
  -- gothic shield with a cross of optics
  s:part(M.chrome, { bevel = 2 }, function(m) m:poly({ { 6, 2 }, { 30, 2 }, { 42, 11.5 }, { 30, 21 }, { 6, 21 }, { 10, 11.5 } }) end)
  s:part(GUN, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 12, 5 }, { 28, 5 }, { 36, 11.5 }, { 28, 18 }, { 12, 18 }, { 15, 11.5 } }) end)
  s:part(M.chrome, { bevel = 1, shadow = 0 }, function(m) m:rect(16, 10.5, 16, 2); m:rect(22, 6, 2, 11) end)
  optic(s, 23, 11.5, 2.2)
  slits(s, 8, 30, 3, 4, true)
  s:engine(6, 6, 4, 6, GUN, true)
  return s:finish()
end

E.synod[4] = function()
  local s = new("synod", 4)
  cables(s, 8, 15.5, 3, 8)
  -- stepped cathedral hull
  s:part(GUN, { bevel = 2 }, function(m) m:poly({ { 8, 4 }, { 30, 4 }, { 34, 0 }, { 40, 0 }, { 44, 6 }, { 22, 9 } }, true) end)
  s:part(M.chrome, { bevel = 2 }, function(m) m:poly({ { 6, 8 }, { 40, 7 }, { 52, 11 }, { 58, 15.5 }, { 52, 20 }, { 40, 24 }, { 6, 23 } }) end)
  s:part(GUN, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 12, 11 }, { 40, 10.5 }, { 46, 15.5 }, { 40, 20.5 }, { 12, 20 } }) end)
  slits(s, 14, 38, 13, 4); slits(s, 14, 38, 17, 4)
  slits(s, 12, 28, 5, 4, true)
  optic(s, 50, 15.5, 2.5)
  s:engine(6, 10, 4, 7, GUN, true)
  return s:finish()
end

E.synod[5] = function()
  local s = new("synod", 5)
  cables(s, 14, 24.5, 5, 12)
  -- the face: a chrome mask around one great eye
  s:part(GUN, { bevel = 3 }, function(m) m:poly({ { 12, 10 }, { 36, 2 }, { 60, 0 }, { 76, 6 }, { 60, 16 }, { 30, 18 } }, true) end)
  s:part(M.chrome, { bevel = 3 }, function(m) m:poly({ { 10, 14 }, { 50, 9 }, { 80, 14 }, { 100, 24.5 }, { 80, 35 }, { 50, 40 }, { 10, 35 } }) end)
  s:part(GUN, { bevel = 2, shadow = 0 }, function(m) m:poly({ { 20, 18 }, { 52, 15 }, { 74, 20 }, { 82, 24.5 }, { 74, 29 }, { 52, 34 }, { 20, 31 } }) end)
  for x = 26, 46, 5 do s:vline(x, 18, 31, M.chrome[2]) end
  slits(s, 24, 70, 6, 5, true)
  optic(s, 64, 24.5, 7)
  s:part(OPTIC, { flat = 1, shadow = 0 }, function(m) m:ellipse(65, 24.5, 1.5, 4) end)
  for _, p in ipairs({ { 36, 12 }, { 50, 10 } }) do s:turret("laser", p[1], p[2], M.chrome, 2, true, OPTIC) end
  s:engine(10, 18, 6, 12, GUN, true)
  return s:finish()
end

-- ---------------------------------------------------------------------------
-- Logistics ships: every faction's healer carries the same lime glow, so the
-- player learns to spot (and kill) it first.
-- ---------------------------------------------------------------------------
local LIME = { "#2a4a06", "#5a9a10", "#b4f03a", "#e8ffb0", "#ffffff" }
local LOGI_SIZE = { 34, 19 }

local function limeCore(s, x, y, r)
  s:part(LIME, { round = true, shadow = 0, tone = 0.1 }, function(m) m:ellipse(x, y, r, r) end)
  s:glow(math.floor(x), math.floor(y), r + 0.5, LIME)
end

local function newLogi(f)
  local s = F.new(LOGI_SIZE[1], LOGI_SIZE[2])
  s.glowRamp = FLAMES[f]
  return s
end

-- Crimson "Leech": claw arms clutching a stolen repair core.
E.crimson.logi = function()
  local s = newLogi("crimson")
  s:part(IRON, { bevel = 1 }, function(m) m:poly({ { 16, 6 }, { 28, 2 }, { 33, 4 }, { 28, 4.5 }, { 22, 7.5 } }, true) end)
  s:part(BLOOD, { bevel = 1 }, function(m) m:poly({ { 2, 5 }, { 18, 6 }, { 24, 9.5 }, { 18, 13 }, { 2, 14 }, { 6, 9.5 } }) end)
  spikes(s, 6, 12, 6, 3, 6)
  limeCore(s, 27, 9.5, 2.6)
  s:engine(4, 8, 3, 6, IRON)
  return s:finish()
end

-- Verdant "Nurse": a swollen sac of healing ichor on spindly legs.
E.verdant.logi = function()
  local s = newLogi("verdant")
  legs(s, { 12, 18 }, 5, 0, true)
  segment(s, 10, 9.5, 8, 6, -0.05)
  s:part(LIME, { round = true, shadow = 0 }, function(m) m:ellipse(12, 9.5, 5.5, 4.2) end)
  s:glow(12, 9, 3.5, LIME)
  segment(s, 24, 9.5, 5, 4, 0.06)
  s:px(27, 8, SAC[4]); s:px(27, 11, SAC[4])
  s:engine(2, 8, 3, 5, BELLY)
  return s:finish()
end

-- Void "Mender of Shades": twin rings around a lime flame.
E.void.logi = function()
  local s = newLogi("void")
  s:part(VOID, { bevel = 1 }, function(m) m:ellipse(17, 9.5, 8, 8):ellipse(17, 9.5, 5.5, 5.5, false, true) end)
  s:part(OBSID, { bevel = 1 }, function(m) m:poly({ { 2, 8.5 }, { 28, 8.5 }, { 34, 9.5 }, { 28, 10.5 }, { 2, 10.5 } }) end)
  runes(s, 17, 9.5, 7, 6, 0, math.pi * 2 - 0.1)
  limeCore(s, 17, 9.5, 3)
  s:engine(2, 9, 1, 5, OBSID)
  return s:finish()
end

-- Synod "Restorer": a chrome drone with a lime optic and a dish.
E.synod.logi = function()
  local s = newLogi("synod")
  s:part(GUN, { bevel = 1 }, function(m) m:bar(14, 6, 18, 2, 1, true) end)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(19, 2.5, 3, 2.2, true) end)
  s:part(M.chrome, { bevel = 1 }, function(m) m:poly({ { 2, 5 }, { 20, 5 }, { 32, 9.5 }, { 20, 14 }, { 2, 14 } }) end)
  slits(s, 6, 16, 7, 3)
  limeCore(s, 22, 9.5, 2.4)
  s:engine(2, 7, 5, 6, GUN)
  return s:finish()
end

-- adapter: E[faction](tier)
local out = {}
for k, v in pairs(E) do out[k] = function(t) return v[t]() end end
-- E.logi[faction]() builds that faction's logistics ship
out.logi = {}
for k, v in pairs(E) do out.logi[k] = v.logi end
return out
