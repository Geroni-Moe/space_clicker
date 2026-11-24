-- Tidewall State: a megacorp navy. Knife-edged "tuning fork" prows with a
-- particle projector in the throat, navy hulls in white chevron livery with
-- orange safety bands, hull numbers, armed missile silos, hex shield walls.
-- Each entry builds one hull class; cmd = true adds the command array.

local px = dofile("art/lib/px.lua")
local P = dofile("art/lib/palette.lua")
local F = dofile("art/lib/forge.lua")
local M = dofile("art/lib/materials.lua")

local FL = M.flame.tide
local O = P.outline
local ORANGE, WARHEAD = M.orange, M.warhead
local T = {}

local function new(name, w)
  local s = F.new(w or M.CLASS[name][1], M.CLASS[name][2])
  s.glowRamp = FL
  return s
end

--- Hex-patterned shield arc projected ahead of the ship (drawn after outline).
local function hexShield(s, cx, cy, r, a0, a1)
  s:after(function(sh)
    for y = 0, sh.H - 1 do
      for x = 0, sh.W - 1 do
        local dx, dy = x + 0.5 - (cx + sh.ox), -(y + 0.5 - (cy + sh.oy))
        local d = math.sqrt(dx * dx + dy * dy)
        local a = math.atan(dy, dx)
        if a >= a0 and a <= a1 and not px.opaque(sh.img, x, y) and d <= r and d > r - 4 then
          local u, v = math.floor(a * r / 3), math.floor(r - d)
          if d > r - 1 then px.set(sh.img, x, y, "#8af0ffd0")
          elseif math.floor(a * r) % 3 == 0 then px.set(sh.img, x, y, "#40d0e6a0")
          elseif (u + v % 2) % 2 == 0 and (x + y) % 2 == 0 then px.set(sh.img, x, y, "#1db2cc50") end
        end
      end
    end
  end)
end

--- Silo field: tubes in a grid, some loaded with glowing warheads.
local function silos(s, x, y, cols, rows, seed)
  local rnd = px.rng(seed)
  for r = 0, rows - 1 do
    for c = 0, cols - 1 do
      local xx, yy = x + c * 3, y + r * 3
      s:rect(xx, yy, 2, 2, O)
      if rnd(1, 3) == 1 then
        s:px(xx, yy, WARHEAD[2]); s:px(xx + 1, yy, WARHEAD[1]); s:px(xx, yy + 1, WARHEAD[1])
      else
        s:px(xx + 1, yy + 1, M.cyan[2])
      end
    end
  end
end

local function band(s, x0, x1, y, h)
  for yy = y, y + h - 1 do s:hline(x0, x1, yy, ORANGE[3]) end
  s:hline(x0, x1, y, ORANGE[4])
end

--- Projector in the fork's throat, with sparks running down the gap.
local function projector(s, x, y, r, x1)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(x, y, r, r + 0.5) end)
  s:glow(x + 1, math.floor(y), r, M.cyan)
  for xx = x + r + 3, x1, 3 do s:px(xx, math.floor(y), M.cyan[4]); s:px(xx + 1, math.floor(y), M.cyan[3]) end
end

--- Command array: a gold-rimmed radar dish with two antenna masts.
local function crown(s, x, y, r)
  r = r or 3
  s:line(x - 1, y, x - 5, y - r - 4, M.white[4]); s:line(x + 1, y, x - 1, y - r - 6, M.white[4])
  s:glow(x - 5, y - r - 4, 1, M.commandGold); s:glow(x - 1, y - r - 6, 1, M.commandGold)
  s:part(M.gold, { round = true }, function(m) m:ellipse(x, y, r + 1, r + 1) end)
  s:part(M.white, { round = true, shadow = 0 }, function(m) m:ellipse(x - 0.5, y - 0.5, r, r) end)
  s:px(math.floor(x), math.floor(y), M.gold[6])
end

--- Parked strike fighter for the carrier rails.
local function fighter(s, x, y)
  s:map(x, y, { "nn...", "wnnnc", "nn..." }, { n = M.navy[5], w = M.white[5], c = M.cyan[4] })
end

-- Interceptor "Skua": the smallest fork, all fin and throat.
T.interceptor = function(cmd)
  local s = new("interceptor")
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 3, 4 }, { 10, 4 }, { 7, 0 }, { 3, 0 } }) end)
  s:part(ORANGE, { bevel = 1, shadow = 0 }, function(m) m:rect(3, 0, 4, 1.2) end)
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 10, 4 }, { 20, 4.5 }, { 23, 6 }, { 10, 6.5 } }) end)
  s:part(M.navy, { bevel = 1 }, function(m) m:poly({ { 8, 8.5 }, { 20, 8.5 }, { 26, 9.5 }, { 20, 10.5 }, { 8, 10.5 } }) end)
  s:part(M.navy, { bevel = 1 }, function(m) m:poly({ { 2, 4 }, { 12, 3.5 }, { 14, 5 }, { 14, 10 }, { 12, 11.5 }, { 2, 11 } }) end)
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 6, 3.8 }, { 8, 3.7 }, { 11, 11.3 }, { 9, 11.3 } }) end)
  s:glow(15, 7, 1, M.cyan)
  s:engine(2, 5, 5, 7, M.navy)
  if cmd then crown(s, 5, 7, 1.5) end
  return s:finish()
end

-- Frigate "Petrel": a forked knife with a raked fin.
T.frigate = function(cmd)
  local s = new("frigate", 38)
  s:part(M.navy, { bevel = 1, tone = -0.06 }, function(m) m:poly({ { 6, 15 }, { 14, 15 }, { 12, 20 }, { 5, 20 } }) end)
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 3, 6 }, { 13, 6 }, { 9, 0 }, { 3, 0 } }) end)
  s:part(ORANGE, { bevel = 1, shadow = 0 }, function(m) m:rect(3, 0, 6, 1.5) end)
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 14, 6 }, { 28, 6.5 }, { 32, 8 }, { 14, 9 } }) end)
  s:part(M.navy, { bevel = 1 }, function(m) m:poly({ { 12, 11 }, { 30, 11 }, { 38, 12.5 }, { 30, 14 }, { 12, 14 } }) end)
  s:part(M.navy, { bevel = 1 }, function(m) m:poly({ { 2, 6 }, { 16, 5 }, { 20, 7 }, { 20, 14 }, { 16, 16 }, { 2, 15 } }) end)
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 8, 5.5 }, { 12, 5.3 }, { 16, 15.5 }, { 12, 15.5 } }) end)
  band(s, 3, 7, 12, 1)
  s:glow(21, 10, 1.5, M.cyan)
  s:px(18, 8, M.cyan[4])
  silos(s, 7, 17, 2, 1, 3)
  s:engine(2, 7, 7, 10, M.navy)
  if cmd then crown(s, 6, 10, 2) end
  return s:finish()
end

-- Destroyer "Pike": a long fork with a launcher rail on an outrigger.
T.destroyer = function(cmd)
  local s = new("destroyer")
  s:part(M.navy, { bevel = 1, tone = -0.1 }, function(m) m:bar(16, 4, 18, 7, 1.1) end)
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 8, 0 }, { 28, 0 }, { 31, 2 }, { 28, 4 }, { 8, 4 } }) end)
  silos(s, 10, 1, 5, 1, 13)
  band(s, 26, 27, 0, 4)
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 22, 7 }, { 38, 8 }, { 42, 10 }, { 22, 11 } }) end)
  s:part(M.navy, { bevel = 1 }, function(m) m:poly({ { 22, 14 }, { 40, 14 }, { 46, 16 }, { 40, 18 }, { 22, 18.5 } }) end)
  s:hline(26, 40, 16, ORANGE[3])
  s:part(M.navy, { bevel = 2 }, function(m) m:poly({ { 2, 6 }, { 22, 5 }, { 26, 8 }, { 26, 18 }, { 22, 20 }, { 2, 19 } }) end)
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 8, 5.6 }, { 12, 5.4 }, { 18, 19.6 }, { 14, 19.6 } }) end)
  s:digits(6, 14, "24", M.white[4])
  projector(s, 27, 12.5, 1.5, 40)
  s:engine(2, 7, 5, 11, M.navy); s:engine(2, 13, 5, 11, M.navy)
  if cmd then crown(s, 20, 12, 2) end
  return s:finish()
end

-- Cruiser "Bulwark": forked prow, a launcher on an outrigger arm, an antenna mast.
T.cruiser = function(cmd)
  local s = new("cruiser", 60)
  s:line(9, 7, 6, 1, M.white[4]); s:line(10, 7, 7, 1, M.white[2]); s:glow(6, 1, 1, M.cyan)
  s:part(M.navy, { bevel = 1, tone = -0.1 }, function(m) m:bar(22, 8, 25, 3, 1.2) end)
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 16, 0 }, { 36, 0 }, { 39, 2 }, { 36, 4.5 }, { 16, 4.5 } }) end)
  silos(s, 18, 1, 6, 1, 7)
  band(s, 34, 35, 0, 4)
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 30, 8 }, { 50, 9 }, { 56, 11 }, { 30, 13 } }) end)
  s:part(M.navy, { bevel = 2 }, function(m) m:poly({ { 30, 18 }, { 54, 18 }, { 60, 20 }, { 54, 22.5 }, { 30, 23 } }) end)
  s:hline(34, 54, 20, ORANGE[3])
  s:part(M.navy, { bevel = 2 }, function(m) m:poly({ { 4, 7 }, { 32, 7 }, { 37, 10 }, { 37, 22 }, { 32, 25 }, { 4, 25 } }) end)
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 12, 7.3 }, { 18, 7.3 }, { 27, 24.7 }, { 21, 24.7 } }) end)
  s:digits(9, 18, "07", M.white[4])
  band(s, 28, 36, 12, 1)
  projector(s, 39, 15.5, 2.5, 52)
  hexShield(s, 30, 15.5, 31, -0.75, 0.75)
  s:turret("missile", 26, 18, M.white, 1, false, WARHEAD)
  s:engine(4, 8, 7, 12, M.navy); s:engine(4, 17, 7, 12, M.navy)
  if cmd then crown(s, 24, 14, 3) end
  return s:finish()
end

-- Battlecruiser "Breaker": a fork with lopsided shoulder pods full of silos.
T.battlecruiser = function(cmd)
  local s = new("battlecruiser")
  s:part(M.navy, { bevel = 1, tone = -0.1 }, function(m) m:bar(24, 6, 26, 9, 1.4); m:bar(26, 26, 26, 29, 1.2) end)
  s:part(M.white, { bevel = 2 }, function(m) m:poly({ { 12, 0 }, { 40, 0 }, { 45, 3.5 }, { 40, 7 }, { 12, 7 } }) end)
  silos(s, 14, 2, 8, 1, 17)
  band(s, 37, 39, 0, 7)
  s:part(M.navy, { bevel = 1 }, function(m) m:poly({ { 18, 28 }, { 34, 28 }, { 37, 30.5 }, { 34, 33 }, { 18, 33 } }) end)
  silos(s, 20, 29, 4, 1, 5)
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 38, 9 }, { 58, 10 }, { 64, 12.5 }, { 38, 15 } }) end)
  s:part(M.navy, { bevel = 2 }, function(m) m:poly({ { 38, 20 }, { 62, 20 }, { 70, 22.5 }, { 62, 25 }, { 38, 25.5 } }) end)
  band(s, 44, 62, 22, 1)
  s:part(M.navy, { bevel = 2 }, function(m) m:poly({ { 4, 7 }, { 36, 6 }, { 42, 10 }, { 42, 26 }, { 36, 29 }, { 4, 29 } }) end)
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m)
    m:poly({ { 26, 6.3 }, { 31, 6.2 }, { 22, 28.7 }, { 17, 28.7 } })
  end)
  s:digits(9, 21, "48", M.white[4])
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 6, 10 }, { 18, 10 }, { 21, 13 }, { 18, 16 }, { 6, 16 } }) end)
  s:dots(8, 18, 13, M.cyan[4], 2)
  projector(s, 44, 17.5, 3, 62)
  hexShield(s, 34, 17.5, 38, -0.7, 0.7)
  s:engine(4, 8, 6, 13, M.navy); s:engine(4, 15, 6, 13, M.navy); s:engine(4, 22, 6, 13, M.navy)
  if cmd then crown(s, 30, 17, 3) end
  return s:finish()
end

-- Battleship "Tsunami": a great fork with raked fins and banks of armed silos.
T.battleship = function(cmd)
  local s = new("battleship", 88)
  s:part(M.white, { bevel = 2 }, function(m) m:poly({ { 8, 9 }, { 32, 9 }, { 24, 0 }, { 10, 0 } }) end)
  s:part(M.navy, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 10, 0 }, { 24, 0 }, { 25, 1.5 }, { 11, 1.5 } }) end)
  band(s, 12, 18, 5, 2)
  s:part(M.navy, { bevel = 2, tone = -0.06 }, function(m) m:poly({ { 12, 35 }, { 30, 35 }, { 24, 43 }, { 12, 43 } }) end)
  s:part(M.white, { bevel = 2 }, function(m) m:poly({ { 46, 10 }, { 76, 11 }, { 84, 14 }, { 46, 17 } }) end)
  s:part(M.navy, { bevel = 2 }, function(m) m:poly({ { 46, 25 }, { 80, 25 }, { 88, 28 }, { 80, 31.5 }, { 46, 32 } }) end)
  band(s, 52, 80, 28, 1)
  s:part(M.navy, { bevel = 2 }, function(m) m:poly({ { 4, 8 }, { 48, 7 }, { 54, 12 }, { 54, 31 }, { 48, 36 }, { 4, 36 } }) end)
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 34, 7.3 }, { 42, 7.3 }, { 30, 35.7 }, { 22, 35.7 } }) end)
  silos(s, 7, 11, 5, 3, 11); silos(s, 7, 26, 5, 2, 19); silos(s, 38, 20, 3, 3, 23)
  s:digits(41, 29, "112", M.white[4])
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 24, 18 }, { 36, 18 }, { 40, 21.5 }, { 36, 25 }, { 24, 25 } }) end)
  s:dots(26, 36, 21, M.cyan[4], 2)
  projector(s, 56, 21.5, 3.5, 78)
  hexShield(s, 48, 21.5, 42, -0.6, 0.6)
  s:turret("missile", 64, 13.5, M.navy, 1, false, WARHEAD)
  s:turret("missile", 66, 28.5, M.white, 1, false, WARHEAD)
  s:engine(4, 9, 8, 14, M.navy); s:engine(4, 18, 7, 13, M.navy); s:engine(4, 27, 8, 14, M.navy)
  if cmd then crown(s, 30, 21, 3.5) end
  return s:finish()
end

-- Carrier "Hive": a numbered block with launch rails full of parked fighters.
T.carrier = function(cmd)
  local s = new("carrier")
  -- launch rails reaching past the bow
  for _, y in ipairs({ 3, 44 }) do
    s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 26, y }, { 100, y }, { 104, y + 2 }, { 100, y + 4 }, { 26, y + 4 } }) end)
    s:dots(28, 100, y + 2, M.cyan[3], 4)
    band(s, 96, 98, y, 4)
  end
  s:part(M.navy, { bevel = 3 }, function(m) m:poly({ { 6, 7 }, { 84, 7 }, { 96, 15 }, { 96, 36 }, { 84, 44 }, { 6, 44 } }) end)
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 50, 7.3 }, { 58, 7.3 }, { 44, 43.7 }, { 36, 43.7 } }) end)
  for x = 30, 92, 9 do fighter(s, x, 3); fighter(s, x, 45) end
  silos(s, 10, 11, 7, 2, 29); silos(s, 10, 34, 7, 2, 37)
  s:digits(64, 32, "220", M.white[4])
  band(s, 8, 30, 25, 1)
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 60, 15 }, { 80, 15 }, { 86, 20 }, { 80, 25 }, { 60, 25 } }) end)
  s:dots(62, 82, 18, M.cyan[4], 2); s:dots(62, 80, 22, M.cyan[3], 2)
  projector(s, 98, 25.5, 2.5, 104)
  s:turret("missile", 30, 25.5, M.white, 2, false, WARHEAD)
  s:engine(6, 9, 8, 15, M.navy); s:engine(6, 21, 9, 16, M.navy); s:engine(6, 34, 8, 15, M.navy)
  if cmd then crown(s, 46, 20, 4) end
  return s:finish()
end

-- Dreadnought "Bastion": a siege slab with its torpedo doors open.
T.dreadnought = function(cmd)
  local s = new("dreadnought")
  for _, y in ipairs({ 6, 45 }) do
    s:part(M.chrome, { bevel = 1 }, function(m) m:poly({ { 88, y - 2 }, { 104, y - 1 }, { 104, y + 1 }, { 88, y + 2 } }) end)
    s:glow(104, y, 1.5, M.cyan)
  end
  s:part(M.navy, { bevel = 3 }, function(m) m:poly({ { 4, 6 }, { 86, 5 }, { 96, 12 }, { 96, 39 }, { 86, 46 }, { 4, 45 } }) end)
  s:part(M.white, { bevel = 2 }, function(m) m:poly({ { 84, 9 }, { 112, 19 }, { 112, 32 }, { 84, 42 }, { 90, 25.5 } }) end)
  -- open torpedo tubes with warheads showing
  for _, y in ipairs({ 16, 22, 28, 34 }) do
    s:rect(100, y - 1, 6, 3, O)
    s:px(104, y, M.white[6]); s:px(105, y, WARHEAD[1]); s:px(103, y, M.white[4])
  end
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m)
    m:poly({ { 30, 5.4 }, { 38, 5.3 }, { 24, 45.6 }, { 16, 45.6 } })
    m:poly({ { 42, 5.3 }, { 46, 5.3 }, { 32, 45.6 }, { 28, 45.6 } })
  end)
  silos(s, 50, 9, 10, 3, 61); silos(s, 50, 33, 10, 3, 67)
  s:digits(9, 10, "077", M.white[4])
  band(s, 6, 84, 25, 1)
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 54, 19 }, { 76, 19 }, { 82, 25.5 }, { 76, 32 }, { 54, 32 } }) end)
  s:dots(56, 78, 23, M.cyan[4], 2); s:dots(56, 76, 28, M.cyan[3], 2)
  hexShield(s, 80, 25.5, 40, -0.7, 0.7)
  s:engine(4, 8, 8, 16, M.navy); s:engine(4, 18, 7, 15, M.navy); s:engine(4, 27, 7, 15, M.navy); s:engine(4, 36, 8, 16, M.navy)
  if cmd then crown(s, 40, 25, 4) end
  return s:finish()
end

-- Titan "Citadel": a fortress fork charging a particle lance behind a hex wall.
T.titan = function(cmd)
  local s = new("titan", 126)
  s:part(M.white, { bevel = 2 }, function(m) m:poly({ { 10, 7 }, { 40, 7 }, { 32, 0 }, { 14, 0 } }) end)
  band(s, 14, 26, 3, 2)
  s:part(M.navy, { bevel = 2, tone = -0.06 }, function(m) m:poly({ { 14, 48 }, { 40, 48 }, { 32, 55 }, { 14, 55 } }) end)
  s:part(M.white, { bevel = 2 }, function(m) m:poly({ { 62, 8 }, { 108, 10 }, { 122, 15 }, { 62, 21 } }) end)
  s:part(M.navy, { bevel = 2 }, function(m) m:poly({ { 62, 34 }, { 112, 34 }, { 126, 39.5 }, { 112, 45 }, { 62, 46 } }) end)
  band(s, 70, 110, 39, 2)
  silos(s, 72, 12, 10, 2, 41)
  s:part(M.navy, { bevel = 3 }, function(m) m:poly({ { 4, 6 }, { 64, 5 }, { 72, 12 }, { 72, 43 }, { 64, 49 }, { 4, 49 } }) end)
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m)
    m:poly({ { 44, 5.3 }, { 52, 5.3 }, { 38, 48.7 }, { 30, 48.7 } })
    m:poly({ { 56, 5.3 }, { 60, 5.3 }, { 46, 48.7 }, { 42, 48.7 } })
  end)
  silos(s, 8, 10, 8, 3, 29); silos(s, 8, 36, 8, 3, 31)
  s:digits(10, 22, "001", M.white[4])
  band(s, 6, 26, 30, 1)
  s:part(M.white, { bevel = 1 }, function(m) m:poly({ { 48, 20 }, { 64, 20 }, { 68, 27.5 }, { 64, 35 }, { 48, 35 } }) end)
  s:dots(50, 64, 24, M.cyan[4], 2); s:dots(50, 64, 31, M.cyan[3], 2)
  s:part(M.chrome, { round = true }, function(m) m:ellipse(75, 27.5, 4.5, 7) end)
  s:glow(76, 27, 5, M.cyan)
  for x = 82, 110, 4 do
    local j = (x // 4) % 3 - 1
    s:vline(x, 22 + j, 33 - j, M.cyan[3]); s:px(x, 27, M.cyan[5])
  end
  hexShield(s, 70, 27.5, 60, -0.55, 0.55)
  for _, x in ipairs({ 84, 98 }) do s:turret("missile", x, 15, M.navy, 2, false, WARHEAD) end
  s:engine(4, 8, 8, 18, M.navy); s:engine(4, 18, 8, 18, M.navy); s:engine(4, 29, 8, 18, M.navy); s:engine(4, 39, 8, 18, M.navy)
  if cmd then crown(s, 56, 27, 4.5) end
  return s:finish()
end

-- ---------------------------------------------------------------------------
-- Logistics: shield-transfer dishes and hex walls, marked with a mint cross.
-- ---------------------------------------------------------------------------

--- Mint repair cross on a white panel: the Tidewall logi marking.
local function cross(s, x, y)
  s:map(x, y, { "wwwww", "ww+ww", "w+++w", "ww+ww", "wwwww" }, { w = M.white[5], ["+"] = M.mend[3] })
end

--- Shield-transfer dish on a strut, its emitter glowing.
local function dish(s, x0, y0, x, y, r, m)
  s:part(M.navy, { bevel = 1, tone = -0.1 }, function(mk) mk:bar(x0, y0, x, y, 1.1, m) end)
  s:part(M.white, { round = true }, function(mk) mk:ellipse(x, y, r + 0.5, r) end)
  if m then s:part(M.white, { round = true }, function(mk) mk:ellipse(x, s.h - y, r + 0.5, r) end) end
  s:part(M.cyan, { round = true, shadow = 0, tone = 0.1 }, function(mk) mk:ellipse(x + 0.5, y, r * 0.45, r * 0.45, m) end)
  s:glow(math.floor(x + 0.5), math.floor(y), r * 0.6, M.mend, m)
end

-- Logi Frigate "Buoy": a short fork lifting one big transfer dish.
T.logi_frigate = function()
  local s = new("logi_frigate")
  dish(s, 14, 7, 17, 4, 4, false)
  s:part(M.navy, { bevel = 1 }, function(m) m:poly({ { 2, 6 }, { 16, 5 }, { 22, 8 }, { 36, 10.5 }, { 22, 13 }, { 16, 16 }, { 2, 15 } }) end)
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 6, 5.6 }, { 10, 5.4 }, { 14, 15.6 }, { 10, 15.6 } }) end)
  cross(s, 17, 8)
  band(s, 24, 30, 10, 1)
  s:engine(2, 7, 7, 10, M.navy)
  return s:finish()
end

-- Logi Cruiser "Aegis": a numbered slab between two dishes, inside its own hex wall.
T.logi_cruiser = function()
  local s = new("logi_cruiser")
  dish(s, 24, 8, 28, 4.5, 4.5, true)
  s:part(M.navy, { bevel = 2 }, function(m) m:poly({ { 4, 7 }, { 40, 6 }, { 52, 11 }, { 58, 15.5 }, { 52, 20 }, { 40, 25 }, { 4, 24 } }) end)
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 14, 6.4 }, { 20, 6.3 }, { 30, 24.6 }, { 24, 24.6 } }) end)
  cross(s, 36, 13)
  s:digits(8, 9, "33", M.white[4])
  band(s, 8, 20, 20, 1)
  s:dots(42, 52, 15, M.cyan[4], 2)
  hexShield(s, 30, 15.5, 30, -0.8, 0.8)
  s:engine(4, 8, 7, 12, M.navy); s:engine(4, 17, 7, 12, M.navy)
  return s:finish()
end

-- Force Auxiliary "Lighthouse": a fortress with a ringed beacon and four dishes.
T.fax = function()
  local s = new("fax")
  dish(s, 30, 9, 28, 5, 5, true)
  dish(s, 70, 9, 72, 5, 5, true)
  s:part(M.navy, { bevel = 3 }, function(m) m:poly({ { 6, 8 }, { 80, 7 }, { 94, 14 }, { 104, 25.5 }, { 94, 37 }, { 80, 44 }, { 6, 43 } }) end)
  s:part(M.white, { bevel = 1, shadow = 0 }, function(m)
    m:poly({ { 16, 7.4 }, { 24, 7.3 }, { 12, 43.6 }, { 4, 43.6 } })
  end)
  silos(s, 76, 14, 4, 2, 71); silos(s, 76, 33, 4, 2, 73)
  s:digits(28, 37, "100", M.white[4])
  band(s, 30, 88, 25, 1)
  -- the beacon: rings around a bright core
  s:part(M.chrome, { round = true }, function(m) m:ellipse(54, 25.5, 13, 13) end)
  s:part(M.cyan, { bevel = 1, shadow = 0 }, function(m) m:arc(54, 25.5, 10.5, 1.5, -math.pi, math.pi) end)
  s:part(M.mend, { round = true, shadow = 0, tone = 0.15 }, function(m) m:ellipse(54, 25.5, 6, 6) end)
  s:glow(54, 25, 4.5, M.mend)
  cross(s, 84, 23)
  hexShield(s, 50, 25.5, 48, -0.75, 0.75)
  s:engine(6, 10, 8, 15, M.navy); s:engine(6, 22, 8, 15, M.navy); s:engine(6, 34, 8, 15, M.navy)
  return s:finish()
end

return T
