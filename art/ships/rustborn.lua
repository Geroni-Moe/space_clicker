-- Rustborn Clans: scrap-built raiders. War paint, painted shark jaws,
-- mismatched salvage, too many engines, guns welded wherever they fit.
-- Each entry builds one hull class; cmd = true adds the warlord's totem.

local px = dofile("art/lib/px.lua")
local P = dofile("art/lib/palette.lua")
local F = dofile("art/lib/forge.lua")
local M = dofile("art/lib/materials.lua")

local FL = M.flame.rust
local O = P.outline
local T = {}

local function new(name)
  local s = F.new(M.CLASS[name][1], M.CLASS[name][2])
  s.glowRamp = FL
  return s
end

--- Painted shark jaw: mouth line at row y from x0 to x1, teeth above and below.
local function jaw(s, x0, x1, y, big)
  s:hline(x0, x1, y, O)
  for x = x0, x1 - 1 do
    if big then
      local k = (x - x0) % 3
      if k == 0 then s:px(x, y - 1, M.bone[5]); s:px(x, y - 2, M.bone[4]) elseif k == 1 then s:px(x, y - 1, M.bone[4]) end
      if k == 1 then s:px(x, y + 1, M.bone[4]); s:px(x, y + 2, M.bone[3]) elseif k == 2 then s:px(x, y + 1, M.bone[3]) end
    else
      if (x - x0) % 2 == 0 then s:px(x, y - 1, M.bone[5]) else s:px(x, y + 1, M.bone[4]) end
    end
  end
end

local function hazard(s, x, y, w, h)
  for yy = 0, h - 1 do
    for xx = 0, w - 1 do
      s:px(x + xx, y + yy, math.floor((xx + yy) / 2) % 2 == 0 and M.hazard or "#1c1610")
    end
  end
end

local function rivets(s, x0, x1, y, col, step)
  for x = x0, x1, step or 3 do s:px(x, y, col) end
end

--- Warlord's totem: a bone pole topped with a skull, red banners streaming back.
local function crown(s, x, y)
  s:part(M.warred, { bevel = 1 }, function(m)
    m:poly({ { x - 1, y - 8 }, { x - 13, y - 10 }, { x - 9, y - 6.5 }, { x - 14, y - 3 }, { x - 1, y - 3 } })
  end)
  s:line(x - 11, y - 9, x - 3, y - 7, M.bone[4])
  s:part(M.bone, { bevel = 1 }, function(m) m:rect(x - 0.5, y - 9, 2, 11) end)
  s:part(M.bone, { round = true }, function(m) m:ellipse(x + 0.5, y - 10, 3.2, 2.8) end)
  s:rect(x - 1, y - 11, 1, 2, O); s:rect(x + 1, y - 11, 1, 2, O)
  s:px(x, y - 8, O)
  s:glow(x - 1, y - 10, 0.6, M.ember); s:glow(x + 1, y - 10, 0.6, M.ember)
end

--- A docked raider for the carrier's cradle.
local function raider(s, x, y)
  s:part(M.rust, { bevel = 1 }, function(m) m:poly({ { x + 4, y + 1 }, { x + 8, y + 1 }, { x + 12, y - 3 }, { x + 9, y - 3 } }) end)
  s:part(M.scrap, { round = true }, function(m) m:rect(x, y + 0.5, 5, 6) end)
  s:part(M.rust, { bevel = 1 }, function(m) m:poly({ { x + 3, y + 1 }, { x + 12, y + 1 }, { x + 17, y + 3.5 }, { x + 12, y + 6 }, { x + 3, y + 6 } }) end)
  s:part(M.warred, { bevel = 1, shadow = 0 }, function(m) m:poly({ { x + 12, y + 1.2 }, { x + 17, y + 3.5 }, { x + 12, y + 5.8 }, { x + 11, y + 3.5 } }) end)
  s:hline(x + 12, x + 16, y + 3, O)
  s:px(x + 12, y + 2, M.bone[5]); s:px(x + 14, y + 2, M.bone[5]); s:px(x + 13, y + 4, M.bone[4]); s:px(x + 15, y + 4, M.bone[4])
  s:px(x - 1, y + 3, M.ember[4])
end

-- Interceptor "Spark": a scrap dart, one wing, all teeth.
T.interceptor = function(cmd)
  local s = new("interceptor")
  s:part(M.rust, { bevel = 1 }, function(m) m:poly({ { 6, 6 }, { 11, 6 }, { 18, 0 }, { 14, 0 } }) end)
  s:part(M.warred, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 14, 0 }, { 18, 0 }, { 16, 1.5 }, { 13, 1.2 } }) end)
  s:part(M.scrap, { bevel = 1 }, function(m) m:poly({ { 6, 9 }, { 10, 9 }, { 12, 14 }, { 8, 15 } }) end)
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 4.5, 7, 6) end)
  s:part(M.rust, { bevel = 1 }, function(m) m:poly({ { 5, 5 }, { 17, 5 }, { 26, 7.5 }, { 17, 10 }, { 5, 10 } }) end)
  s:part(M.warred, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 17, 5.2 }, { 26, 7.5 }, { 17, 9.8 }, { 15, 7.5 } }) end)
  jaw(s, 17, 24, 7)
  s:px(20, 5, M.ember[5])
  s:line(10, 5, 11, 9, M.bone[4])
  s:hline(15, 20, 2, M.scrap[6])
  s:engine(0, 6, 3, 7, M.scrap)
  if cmd then crown(s, 11, 6) end
  return s:finish()
end

-- Frigate "Gnasher": a forward-swept dart with a painted shark mouth.
T.frigate = function(cmd)
  local s = new("frigate")
  s:part(M.rust, { bevel = 1 }, function(m) m:poly({ { 4, 9 }, { 15, 9 }, { 27, 0 }, { 19, 0 } }) end)
  s:part(M.warred, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 21, 0 }, { 26, 0 }, { 23, 2.5 }, { 19, 1.5 } }) end)
  s:part(M.scrap, { bevel = 1 }, function(m) m:poly({ { 5, 12 }, { 12, 12 }, { 19, 20 }, { 14, 21 } }) end)
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 6.5, 10, 8) end)
  s:hline(2, 8, 8, M.scrap[1]); s:hline(2, 8, 12, M.scrap[1])
  s:part(M.rust, { bevel = 1 }, function(m) m:poly({ { 7, 7.5 }, { 24, 7 }, { 36, 10.5 }, { 24, 14 }, { 7, 13.5 } }) end)
  s:part(M.warred, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 25, 7.2 }, { 36, 10.5 }, { 25, 13.8 }, { 23, 10.5 } }) end)
  s:hline(25, 34, 10, O)
  s:map(25, 8, { "#.#.#.#.", ".#.#.#.." }, { ["#"] = M.bone[5] })
  s:map(25, 11, { ".#.#.#..", "#.#.#.#." }, { ["#"] = M.bone[4] })
  s:px(30, 8, M.ember[5]); s:glow(30, 8, 1, M.ember)
  s:line(13, 7, 15, 13, M.bone[4]); s:line(14, 7, 16, 13, M.bone[3])
  s:dots(8, 12, 8, M.rust[7], 2); s:dots(18, 22, 13, M.rust[2], 2)
  s:part(M.scrap, { bevel = 1 }, function(m) m:rect(17, 2, 4, 3) end)
  s:hline(21, 28, 2, M.scrap[6]); s:hline(21, 27, 4, M.scrap[5])
  s:seams({ 11, 19 }, 8, 12, M.rust[2], M.rust[6])
  s:engine(0, 8, 5, 9, M.scrap)
  if cmd then crown(s, 20, 9) end
  return s:finish()
end

-- Destroyer "Ripper": a saw-edged ram on a girder, engine pods bolted on.
T.destroyer = function(cmd)
  local s = new("destroyer")
  s:part(M.scrap, { bevel = 1, tone = -0.15 }, function(m) m:bar(10, 7, 14, 11, 0.9); m:bar(10, 18, 13, 14, 0.9) end)
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 3, 13, 7) end)
  s:part(M.rust, { round = true }, function(m) m:rect(2, 17, 9, 6) end)
  s:part(M.rust, { bevel = 2 }, function(m) m:poly({ { 8, 9 }, { 32, 8 }, { 46, 12.5 }, { 32, 17 }, { 8, 16 } }) end)
  s:part(M.warred, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 31, 8.2 }, { 46, 12.5 }, { 31, 16.8 }, { 28, 12.5 } }) end)
  -- saw edge along the prow
  for x = 30, 42, 2 do
    local t = (x - 30) / 16
    s:px(x, math.floor(8 + t * 4) - 1, M.bone[5]); s:px(x, math.floor(17 - t * 4), M.bone[4])
  end
  jaw(s, 32, 44, 12)
  s:px(36, 10, M.ember[5]); s:glow(36, 10, 1, M.ember)
  hazard(s, 24, 9, 3, 7)
  s:line(14, 9, 17, 15, M.bone[4])
  rivets(s, 10, 22, 10, M.rust[7], 3)
  s:seams({ 12, 20 }, 9, 15, M.rust[2], M.rust[6])
  s:turret("auto", 7, 6, M.scrap, 1)
  s:engine(0, 4, 5, 10, M.scrap); s:engine(2, 18, 4, 8, M.rust)
  if cmd then crown(s, 18, 9) end
  return s:finish()
end

-- Cruiser "Scrapjaw": two salvaged hulls lashed together with girders.
T.cruiser = function(cmd)
  local s = new("cruiser")
  s:part(M.rust, { bevel = 1 }, function(m) m:poly({ { 12, 7 }, { 26, 7 }, { 18, 0 }, { 11, 0 } }) end)
  s:line(13, 1, 18, 6, M.bone[4]); s:line(15, 1, 20, 6, M.bone[3])
  s:part(M.scrap, { bevel = 1, tone = -0.15 }, function(m)
    m:bar(14, 13, 18, 18, 0.9); m:bar(24, 13, 20, 18, 0.9); m:bar(30, 13, 34, 18, 0.9)
    m:rect(10, 14.5, 26, 1.2)
  end)
  s:part(M.scrap, { bevel = 2 }, function(m) m:poly({ { 6, 17 }, { 34, 17 }, { 44, 20.5 }, { 34, 25 }, { 6, 25 } }) end)
  hazard(s, 34, 18, 4, 6)
  s:line(14, 18, 17, 24, M.warred[3]); s:line(16, 18, 19, 24, M.warred[3])
  for x = 9, 30, 4 do s:px(x, 23, M.ember[4]) end
  s:part(M.rust, { bevel = 2 }, function(m) m:poly({ { 8, 6 }, { 42, 5 }, { 56, 10 }, { 42, 14 }, { 8, 13 } }) end)
  s:part(M.warred, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 41, 5.2 }, { 56, 10 }, { 41, 13.8 }, { 38, 9.5 } }) end)
  jaw(s, 42, 53, 9)
  s:px(47, 6, M.ember[5]); s:glow(47, 6, 1, M.ember)
  s:part(M.scrap, { bevel = 1 }, function(m) m:poly({ { 16, 7 }, { 24, 6.5 }, { 25, 12 }, { 15, 12.5 } }) end)
  rivets(s, 17, 23, 7, M.scrap[6], 2); rivets(s, 16, 24, 11, M.scrap[1], 2)
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 4, 10, 11) end)
  s:part(M.rust, { round = true }, function(m) m:rect(1, 17, 7, 8) end)
  s:engine(0, 5, 4, 11, M.scrap); s:engine(0, 10, 3, 9, M.scrap); s:engine(1, 18, 5, 8, M.rust)
  s:seams({ 12, 28, 36 }, 6, 12, M.rust[2], M.rust[6])
  s:seams({ 12, 26 }, 18, 24, M.scrap[1], M.scrap[5])
  s:turret("auto", 31, 9.5, M.scrap, 2)
  s:turret("auto", 26, 21, M.scrap, 1)
  if cmd then crown(s, 22, 9) end
  return s:finish()
end

-- Battlecruiser "Hellbore": a squat brawler under one enormous twin autocannon.
T.battlecruiser = function(cmd)
  local s = new("battlecruiser")
  -- strapped-on engine cans
  for _, y in ipairs({ 3, 26 }) do
    s:part(M.scrap, { round = true }, function(m) m:rect(4, y, 18, 6) end)
    s:vline(10, y + 1, y + 4, M.warred[3]); s:vline(16, y + 1, y + 4, M.warred[3])
    s:engine(4, y + 1, 4, 9, M.scrap)
  end
  s:part(M.scrap, { bevel = 1, tone = -0.15 }, function(m) m:bar(14, 8, 18, 12, 1); m:bar(14, 27, 18, 23, 1) end)
  s:part(M.rust, { bevel = 2 }, function(m) m:poly({ { 10, 10 }, { 48, 9 }, { 62, 13 }, { 70, 17.5 }, { 62, 22 }, { 48, 26 }, { 10, 25 } }) end)
  s:part(M.warred, { bevel = 2, shadow = 0 }, function(m) m:poly({ { 52, 10.5 }, { 70, 17.5 }, { 52, 24.5 }, { 48, 17.5 } }) end)
  jaw(s, 52, 67, 17, true)
  s:px(58, 14, M.ember[5]); s:glow(58, 14, 1, M.ember)
  -- bone war-paint chevrons
  for i = 0, 2 do s:line(16 + i * 6, 11, 19 + i * 6, 17, M.bone[4]); s:line(19 + i * 6, 17, 16 + i * 6, 23, M.bone[3]) end
  hazard(s, 44, 21, 6, 3)
  rivets(s, 12, 46, 24, M.rust[2], 3)
  s:seams({ 14, 34, 44 }, 11, 24, M.rust[2], M.rust[6])
  -- the big gun: two barrels from a heavy turret
  s:part(M.scrap, { round = true, radius = 1.5 }, function(m) m:rect(36, 10, 30, 2); m:rect(36, 13.5, 28, 2) end)
  s:part(M.rust, { bevel = 1, shadow = 0 }, function(m) m:rect(54, 9.5, 2, 6.5) end)
  s:part(M.scrap, { round = true }, function(m) m:ellipse(36, 13, 6.5, 5.5) end)
  s:px(34, 11, M.scrap[6])
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 11, 11, 13) end)
  s:engine(0, 12, 5, 12, M.scrap); s:engine(0, 18, 5, 12, M.scrap)
  if cmd then crown(s, 24, 17) end
  return s:finish()
end

-- Battleship "Warpyre": a towering forward-swept sail over a ramming spine.
T.battleship = function(cmd)
  local s = new("battleship")
  s:part(M.rust, { bevel = 2 }, function(m) m:poly({ { 22, 17 }, { 42, 17 }, { 66, 0 }, { 52, 0 } }) end)
  for i = 0, 3 do s:line(28 + i * 5, 16, 54 + i * 3, 1, M.rust[2]) end
  s:line(36, 13, 41, 8, M.bone[4]); s:line(41, 8, 44, 11, M.bone[4]); s:line(44, 11, 49, 5, M.bone[4])
  s:part(M.warred, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 52, 0 }, { 66, 0 }, { 62, 3 }, { 49, 2.5 } }) end)
  s:part(M.scrap, { bevel = 1 }, function(m) m:poly({ { 18, 25 }, { 32, 25 }, { 22, 40 }, { 12, 41 } }) end)
  s:part(M.rust, { round = true }, function(m) m:rect(8, 34, 8, 6) end)
  s:engine(8, 35, 4, 7, M.rust)
  s:part(M.rust, { bevel = 2 }, function(m) m:poly({ { 10, 15 }, { 62, 14 }, { 84, 20.5 }, { 62, 27 }, { 10, 26 } }) end)
  s:part(M.warred, { bevel = 2, shadow = 0 }, function(m) m:poly({ { 62, 14.2 }, { 84, 20.5 }, { 62, 26.8 }, { 58, 20.5 } }) end)
  jaw(s, 62, 80, 20, true)
  s:px(70, 16, M.ember[5]); s:glow(70, 16, 1.5, M.ember)
  hazard(s, 56, 15, 4, 11)
  s:part(M.scrap, { bevel = 1 }, function(m) m:poly({ { 40, 21 }, { 52, 20.5 }, { 53, 25.5 }, { 41, 26 } }) end)
  rivets(s, 41, 52, 21, M.scrap[6], 2)
  for x = 14, 36, 4 do s:px(x, 24, M.ember[4]); s:px(x + 1, 24, M.ember[3]) end
  rivets(s, 12, 54, 16, M.rust[7], 3)
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 12, 14, 17) end)
  s:hline(2, 12, 16, M.scrap[1]); s:hline(2, 12, 24, M.scrap[1])
  s:engine(0, 13, 3, 9, M.scrap); s:engine(0, 17, 6, 14, M.scrap); s:engine(0, 24, 3, 9, M.scrap)
  s:seams({ 18, 26, 36, 54 }, 15, 26, M.rust[2], M.rust[6])
  s:turret("auto", 30, 21, M.scrap, 2); s:turret("auto", 46, 17.5, M.scrap, 2)
  s:turret("auto", 50, 7, M.scrap, 1); s:turret("auto", 42, 12, M.scrap, 1)
  s:turret("auto", 23, 31, M.scrap, 1)
  if cmd then crown(s, 36, 21) end
  return s:finish()
end

-- Carrier "Nest": an open scaffold cradle with raiders docked inside.
T.carrier = function(cmd)
  local s = new("carrier")
  -- the cradle: two rails joined by girders
  s:part(M.scrap, { bevel = 1, tone = -0.15 }, function(m)
    for _, x in ipairs({ 18, 36, 54, 72 }) do m:bar(x, 13, x + 3, 38, 1.1) end
  end)
  s:part(M.rust, { bevel = 2 }, function(m) m:poly({ { 10, 6 }, { 74, 5 }, { 98, 10 }, { 74, 15 }, { 10, 14 } }) end)
  s:part(M.warred, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 76, 5.3 }, { 98, 10 }, { 76, 14.7 }, { 72, 10 } }) end)
  jaw(s, 76, 95, 10)
  s:part(M.scrap, { bevel = 2 }, function(m) m:poly({ { 10, 37 }, { 78, 37 }, { 92, 41 }, { 78, 46 }, { 10, 46 } }) end)
  hazard(s, 74, 38, 5, 7)
  s:line(30, 6, 34, 13, M.bone[4]); s:line(34, 6, 38, 13, M.bone[4])
  rivets(s, 12, 70, 13, M.rust[2], 3); rivets(s, 12, 74, 38, M.scrap[6], 3)
  for x = 14, 70, 6 do s:px(x, 44, M.ember[4]) end
  -- docked raiders
  for _, p in ipairs({ { 22, 20 }, { 44, 20 }, { 32, 30 }, { 54, 30 } }) do raider(s, p[1], p[2]) end
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 4, 12, 13); m:rect(0, 35, 12, 13) end)
  s:engine(0, 5, 5, 12, M.scrap); s:engine(0, 11, 5, 12, M.scrap)
  s:engine(0, 36, 5, 11, M.scrap); s:engine(0, 42, 5, 11, M.scrap)
  s:turret("auto", 50, 10, M.scrap, 2); s:turret("auto", 40, 41.5, M.scrap, 2)
  if cmd then crown(s, 24, 10) end
  return s:finish()
end

-- Dreadnought "Anvil": a slab of layered armour behind a hazard-striped ram.
T.dreadnought = function(cmd)
  local s = new("dreadnought")
  s:part(M.rust, { bevel = 3 }, function(m) m:poly({ { 12, 8 }, { 80, 6 }, { 94, 13 }, { 94, 38 }, { 80, 45 }, { 12, 43 } }) end)
  -- armour layers
  s:part(M.scrap, { bevel = 2 }, function(m) m:poly({ { 30, 7 }, { 60, 6.5 }, { 62, 16 }, { 28, 17 } }) end)
  s:part(M.warred, { bevel = 2 }, function(m) m:poly({ { 48, 30 }, { 78, 31 }, { 76, 42 }, { 46, 41 } }) end)
  s:part(M.rust, { bevel = 2, tone = 0.06 }, function(m) m:poly({ { 18, 20 }, { 70, 19 }, { 72, 30 }, { 18, 31 } }) end)
  rivets(s, 31, 59, 8, M.scrap[6], 2); rivets(s, 49, 75, 31, M.warred[5], 2)
  rivets(s, 20, 68, 20, M.rust[7], 3)
  -- the ram
  s:part(M.scrap, { bevel = 2 }, function(m) m:poly({ { 88, 10 }, { 112, 25.5 }, { 88, 41 }, { 92, 25.5 } }) end)
  for i = 0, 5 do s:line(92 + i * 3, 13 + i * 2, 92 + i * 3, 38 - i * 2, i % 2 == 0 and M.hazard or "#1c1610") end
  jaw(s, 92, 108, 25, true)
  -- clan glyph
  s:line(24, 26, 30, 22, M.bone[4]); s:line(30, 22, 36, 28, M.bone[4]); s:line(36, 28, 42, 22, M.bone[4])
  for x = 16, 84, 5 do s:px(x, 42, M.ember[4]) end
  s:seams({ 22, 44, 66, 82 }, 9, 42, M.rust[2], M.rust[6])
  for _, p in ipairs({ { 38, 11 }, { 52, 11 }, { 60, 25 }, { 30, 36 }, { 60, 36 }, { 80, 18 } }) do
    s:turret("auto", p[1], p[2], M.scrap, 2)
  end
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 9, 16, 33) end)
  s:engine(0, 10, 6, 14, M.scrap); s:engine(0, 17, 7, 16, M.scrap); s:engine(0, 26, 7, 16, M.scrap); s:engine(0, 34, 6, 14, M.scrap)
  if cmd then crown(s, 46, 25) end
  return s:finish()
end

-- Titan "Clanhold": a moving fortress built around one enormous gun.
T.titan = function(cmd)
  local s = new("titan")
  s:part(M.scrap, { bevel = 2 }, function(m) m:poly({ { 30, 40 }, { 54, 40 }, { 44, 52 }, { 24, 53 } }) end)
  s:part(M.rust, { round = true }, function(m) m:rect(20, 46, 9, 6) end)
  s:engine(20, 47, 4, 8, M.rust)
  s:part(M.scrap, { bevel = 1, tone = -0.2 }, function(m)
    for _, x in ipairs({ 22, 40, 58, 74 }) do m:bar(x, 22, x + 4, 29, 1.2) end
  end)
  s:part(M.rust, { bevel = 2, tone = -0.08 }, function(m) m:poly({ { 8, 27 }, { 80, 27 }, { 104, 33.5 }, { 80, 41 }, { 8, 41 } }) end)
  s:part(M.warred, { bevel = 2, shadow = 0 }, function(m) m:poly({ { 80, 27.2 }, { 104, 33.5 }, { 80, 40.8 }, { 75, 33.5 } }) end)
  jaw(s, 80, 100, 33, true)
  s:px(88, 29, M.ember[5]); s:glow(88, 29, 1.5, M.ember)
  for x = 14, 70, 5 do s:rect(x, 38, 2, 1, M.ember[4]) end
  s:part(M.scrap, { bevel = 1 }, function(m) m:poly({ { 50, 28 }, { 66, 27.5 }, { 67, 33 }, { 49, 33.5 } }) end)
  rivets(s, 51, 65, 28, M.scrap[6], 2)
  s:part(M.rust, { bevel = 2 }, function(m) m:poly({ { 14, 8 }, { 72, 6 }, { 92, 12 }, { 92, 21 }, { 14, 24 } }) end)
  hazard(s, 84, 12, 6, 8)
  s:line(26, 20, 32, 12, M.bone[4]); s:line(32, 12, 36, 18, M.bone[4]); s:line(36, 18, 42, 10, M.bone[4]); s:line(42, 10, 46, 16, M.bone[4])
  s:line(27, 20, 33, 12, M.bone[3])
  s:part(M.scrap, { round = true, radius = 3 }, function(m) m:rect(36, 7, 80, 6) end)
  for x = 44, 108, 12 do s:part(M.rust, { bevel = 1, shadow = 0 }, function(m) m:rect(x, 6, 3, 8) end) end
  s:part(M.scrap, { bevel = 1 }, function(m) m:rect(112, 4, 8, 12) end)
  s:vline(115, 5, 14, O); s:vline(117, 5, 14, O)
  s:part(M.scrap, { bevel = 2 }, function(m) m:poly({ { 30, 3 }, { 46, 3 }, { 50, 16 }, { 28, 16 } }) end)
  s:part(M.ember, { flat = 3, shadow = 0 }, function(m) m:rect(34, 6, 2, 6) end)
  s:glow(35, 9, 2, M.ember)
  s:seams({ 20, 30, 62, 74 }, 9, 23, M.rust[2], M.rust[6])
  s:seams({ 18, 28, 40, 70 }, 28, 40, M.rust[2], M.rust[6])
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 6, 16, 20) end)
  s:part(M.rust, { round = true }, function(m) m:rect(2, 28, 9, 12) end)
  s:engine(0, 8, 4, 12, M.scrap); s:engine(0, 13, 6, 17, M.scrap); s:engine(0, 20, 4, 12, M.scrap)
  s:engine(2, 29, 4, 10, M.rust); s:engine(2, 35, 4, 10, M.rust)
  for _, x in ipairs({ 30, 44, 60 }) do s:turret("auto", x, 34, M.scrap, 2) end
  s:turret("auto", 58, 16, M.scrap, 2); s:turret("auto", 72, 15, M.scrap, 1)
  if cmd then crown(s, 22, 16); crown(s, 64, 34) end
  return s:finish()
end

-- ---------------------------------------------------------------------------
-- Logistics: crane arms, welding torches and a forge that never cools.
-- ---------------------------------------------------------------------------

--- Painted mint cross over a scrap plate: the clans' "don't shoot the medic".
local function cross(s, x, y)
  s:map(x, y, { ".sss.", "ss+ss", "s+++s", "ss+ss", ".sss." }, { s = M.bone[4], ["+"] = M.mend[3] })
end

--- Crane arm ending in a welding torch that throws sparks.
local function torch(s, x0, y0, x1, y1, m)
  s:part(M.scrap, { bevel = 1, tone = -0.05 }, function(mk) mk:bar(x0, y0, x1, y1, 1, m) end)
  s:part(M.scrap, { bevel = 1 }, function(mk) mk:rect(x1 - 1.5, y1 - 1.5, 3, 3, m) end)
  s:glow(x1 + 1, y1, 1.4, M.mend, m)
  s:glow(x1 + 3, y1 - 1, 0.6, M.ember, m)
  s:glow(x1 + 2, y1 + 1, 0.6, M.ember, m)
end

-- Logi Frigate "Tinker": a scrap dart with a welding crane over its nose.
T.logi_frigate = function()
  local s = new("logi_frigate")
  torch(s, 12, 8, 26, 2, false)
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 6.5, 10, 8) end)
  s:part(M.rust, { bevel = 1 }, function(m) m:poly({ { 7, 7.5 }, { 24, 7 }, { 34, 10.5 }, { 24, 14 }, { 7, 13.5 } }) end)
  s:part(M.scrap, { bevel = 1 }, function(m) m:poly({ { 5, 13 }, { 12, 13 }, { 17, 20 }, { 12, 21 } }) end)
  cross(s, 16, 8)
  s:seams({ 12, 23 }, 8, 12, M.rust[2], M.rust[6])
  s:engine(0, 8, 5, 9, M.scrap)
  return s:finish()
end

-- Logi Cruiser "Mender": a tug with two crane arms and spare plates lashed on.
T.logi_cruiser = function()
  local s = new("logi_cruiser")
  torch(s, 36, 10, 52, 3, true)
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 7, 12, 17) end)
  s:part(M.rust, { bevel = 2 }, function(m) m:poly({ { 8, 7 }, { 40, 6 }, { 50, 11 }, { 54, 15.5 }, { 50, 20 }, { 40, 25 }, { 8, 24 } }) end)
  -- spare plates stacked on deck
  for i = 0, 2 do
    s:part(i == 1 and M.warred or M.scrap, { bevel = 1 }, function(m) m:rect(14 + i * 2, 9 + i * 2, 12, 4) end)
  end
  cross(s, 32, 13)
  hazard(s, 40, 20, 6, 3)
  rivets(s, 10, 38, 23, M.rust[2], 3)
  s:seams({ 12, 30 }, 8, 23, M.rust[2], M.rust[6])
  s:engine(0, 8, 6, 12, M.scrap); s:engine(0, 17, 6, 12, M.scrap)
  return s:finish()
end

-- Force Auxiliary "Forgeship": a roaring furnace under two gantry cranes.
T.fax = function()
  local s = new("fax")
  -- gantries above and below
  s:part(M.scrap, { bevel = 1, tone = -0.1 }, function(m)
    m:rect(20, 4, 64, 2.5, true)
    for x = 24, 80, 14 do m:bar(x, 6, x + 4, 14, 0.9, true) end
  end)
  torch(s, 78, 6, 96, 4, true)
  s:part(M.rust, { bevel = 3 }, function(m) m:poly({ { 8, 12 }, { 76, 11 }, { 94, 18 }, { 100, 25.5 }, { 94, 33 }, { 76, 40 }, { 8, 39 } }) end)
  -- the furnace mouth
  s:part(M.scrap, { round = true }, function(m) m:ellipse(48, 25.5, 14, 11) end)
  s:part(M.ember, { round = true, shadow = 0, tone = 0.1 }, function(m) m:ellipse(49, 25.5, 10, 8) end)
  for x = 41, 57, 4 do s:vline(x, 19, 32, M.scrap[2]) end
  s:glow(49, 25, 6, M.ember)
  cross(s, 76, 23)
  s:part(M.mend, { round = true, shadow = 0 }, function(m) m:ellipse(90, 25.5, 2.5, 2.5) end)
  s:glow(90, 25, 2.5, M.mend)
  hazard(s, 64, 34, 10, 3)
  rivets(s, 12, 74, 13, M.rust[7], 3); rivets(s, 12, 74, 38, M.rust[2], 3)
  s:seams({ 20, 30, 66 }, 13, 38, M.rust[2], M.rust[6])
  s:part(M.scrap, { round = true }, function(m) m:rect(0, 12, 12, 27) end)
  s:engine(0, 13, 6, 14, M.scrap); s:engine(0, 22, 7, 16, M.scrap); s:engine(0, 32, 6, 14, M.scrap)
  return s:finish()
end

return T
