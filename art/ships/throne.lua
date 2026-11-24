-- Solar Throne: a crusading theocracy. Gold over white marble, crescents and
-- haloes, ruby stained glass, beam lances that burn at the prow.
-- Each entry builds one hull class; cmd = true adds the flagship's halo.

local F = dofile("art/lib/forge.lua")
local M = dofile("art/lib/materials.lua")

local FL = M.flame.throne
local T = {}

local function stainedGlass(s, x0, x1, y, m)
  for x = x0, x1, 2 do s:px(x, y, ((x - x0) / 2) % 2 == 0 and M.ruby[3] or M.gold[6], m) end
end

--- Flagship crown: a gilded halo hovering over the bridge with a sun at its heart.
local function crown(s, x, y, r)
  s:part(M.gold, { bevel = 1 }, function(m) m:arc(x, y, r, 1.4, -math.pi, math.pi) end)
  s:part(M.sun, { round = true, shadow = 0 }, function(m) m:ellipse(x, y, r * 0.4, r * 0.4) end)
  s:glow(math.floor(x), math.floor(y), r * 0.5, M.sun)
end

local function size(name) return M.CLASS[name][1], M.CLASS[name][2] end

-- Interceptor "Seraph": a gold dart between two forward-swept blades.
T.interceptor = function(cmd)
  local s = F.new(size("interceptor"))
  s.glowRamp = FL
  s:part(M.marble, { bevel = 1 }, function(m) m:poly({ { 4, 6 }, { 10, 6 }, { 17, 1 }, { 13, 0 } }, true) end)
  s:part(M.gold, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 13, 0 }, { 17, 1 }, { 15, 2 }, { 12, 1.2 } }, true) end)
  s:part(M.gold, { bevel = 1 }, function(m) m:poly({ { 2, 5 }, { 16, 5 }, { 26, 7.5 }, { 16, 10 }, { 2, 10 } }) end)
  s:part(M.marble, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 5, 6.4 }, { 14, 6.2 }, { 20, 7.5 }, { 14, 8.8 }, { 5, 8.6 } }) end)
  s:px(16, 7, M.ruby[4]); s:glow(16, 7, 0.8, M.ruby)
  s:px(25, 7, M.sun[5]); s:glow(25, 7, 1, M.sun)
  s:engine(2, 6, 3, 6, M.gold)
  if cmd then crown(s, 9, 7.5, 3) end
  return s:finish()
end

-- Frigate "Acolyte": a gilded lance with a halo and twin blade wings.
T.frigate = function(cmd)
  local s = F.new(size("frigate"))
  s.glowRamp = FL
  s:part(M.marble, { bevel = 1 }, function(m) m:poly({ { 6, 8 }, { 12, 8 }, { 22, 1 }, { 18, 0 } }, true) end)
  s:part(M.gold, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 18, 0 }, { 22, 1 }, { 20, 2.5 }, { 16, 1.5 } }, true) end)
  s:part(M.gold, { bevel = 1 }, function(m) m:poly({ { 2, 8 }, { 22, 7.5 }, { 36, 10.5 }, { 22, 13.5 }, { 2, 13 } }) end)
  s:part(M.marble, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 6, 9 }, { 18, 8.6 }, { 26, 10.5 }, { 18, 12.4 }, { 6, 12 } }) end)
  s:part(M.gold, { bevel = 1 }, function(m) m:arc(15, 10.5, 6.5, 1.5, -math.pi, math.pi) end)
  s:px(22, 10, M.ruby[4]); s:px(23, 10, M.ruby[3]); s:glow(22, 10, 1, M.ruby)
  s:px(35, 10, M.sun[5]); s:glow(35, 10, 1.5, M.sun)
  s:engine(2, 9, 3, 8, M.gold)
  if cmd then
    s:part(M.sun, { round = true, shadow = 0 }, function(m) m:ellipse(15, 10.5, 2.5, 2.5) end)
    s:glow(15, 10, 2, M.sun)
  end
  return s:finish()
end

-- Destroyer "Zealot": a lance with feathered wings, three blades a side.
T.destroyer = function(cmd)
  local s = F.new(size("destroyer"))
  s.glowRamp = FL
  for i = 2, 0, -1 do
    s:part(i % 2 == 0 and M.marble or M.gold, { bevel = 1 }, function(m)
      m:poly({ { 10 + i * 7, 10 }, { 17 + i * 7, 10 }, { 6 + i * 8, 1 + i * 2 }, { 2 + i * 8, 1 + i * 2 } }, true)
    end)
  end
  s:part(M.gold, { bevel = 2 }, function(m) m:poly({ { 2, 8 }, { 30, 7.5 }, { 46, 12.5 }, { 30, 17.5 }, { 2, 17 } }) end)
  s:part(M.marble, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 6, 10 }, { 28, 9.8 }, { 37, 12.5 }, { 28, 15.2 }, { 6, 15 } }) end)
  stainedGlass(s, 8, 26, 12)
  s:turret("laser", 30, 12.5, M.gold, 1, false, M.sun)
  s:px(45, 12, M.sun[5]); s:glow(45, 12, 1.5, M.sun)
  s:engine(2, 9, 3, 9, M.gold, true)
  if cmd then crown(s, 16, 12.5, 4) end
  return s:finish()
end

-- Cruiser "Censer": crescent wings around a domed nave, a forked beam prow.
T.cruiser = function(cmd)
  local s = F.new(size("cruiser"))
  s.glowRamp = FL
  s:part(M.gold, { bevel = 2 }, function(m)
    m:arc(26, 15.5, 15.5, function(t) return 1.5 + 3.5 * math.sin(math.pi * math.min(1, t * 1.2)) end, 0.9, math.pi - 0.2, true)
  end)
  stainedGlass(s, 18, 30, 2, true)
  s:part(M.gold, { bevel = 1 }, function(m) m:poly({ { 38, 9 }, { 50, 10 }, { 58, 12.5 }, { 48, 13.5 }, { 38, 13 } }, true) end)
  s:part(M.ruby, { flat = 3, shadow = 0 }, function(m) m:rect(48, 14.5, 9, 2) end)
  s:glow(56, 15, 2, M.sun)
  s:part(M.marble, { bevel = 2 }, function(m) m:poly({ { 4, 10 }, { 36, 9 }, { 48, 15.5 }, { 36, 22 }, { 4, 21 } }) end)
  s:part(M.gold, { bevel = 1, shadow = 0 }, function(m) m:rect(4, 9.5, 32, 1.4); m:rect(4, 20.2, 32, 1.4) end)
  for i, x in ipairs({ 14, 26 }) do
    s:part(M.gold, { round = true }, function(m) m:ellipse(x, 15.5, 4 - i * 0.5 + 1, 4 - i * 0.5) end)
    s:px(x - 1, 14, M.ruby[4]); s:px(x, 14, M.ruby[3]); s:glow(x - 1, 14, 0.8, M.ruby)
  end
  s:turret("laser", 36, 15.5, M.gold, 2, false, M.sun)
  s:engine(4, 11, 4, 10, M.gold, true)
  if cmd then crown(s, 20, 15.5, 5) end
  return s:finish()
end

-- Battlecruiser "Paladin": a nave advancing behind a gilded kite shield.
T.battlecruiser = function(cmd)
  local s = F.new(size("battlecruiser"))
  s.glowRamp = FL
  s:part(M.marble, { bevel = 2 }, function(m) m:poly({ { 6, 14 }, { 20, 4 }, { 32, 2 }, { 30, 14 } }, true) end)
  s:part(M.gold, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 20, 4 }, { 32, 2 }, { 32, 4.5 }, { 21, 6 } }, true) end)
  stainedGlass(s, 18, 28, 9, true)
  s:part(M.gold, { bevel = 2 }, function(m) m:poly({ { 2, 11 }, { 46, 10 }, { 56, 17.5 }, { 46, 25 }, { 2, 24 } }) end)
  s:part(M.marble, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 6, 13 }, { 42, 12.5 }, { 48, 17.5 }, { 42, 22.5 }, { 6, 22 } }) end)
  -- the kite shield
  s:part(M.gold, { bevel = 2 }, function(m) m:poly({ { 46, 4 }, { 60, 10 }, { 70, 17.5 }, { 60, 25 }, { 46, 31 }, { 53, 17.5 } }) end)
  s:part(M.marble, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 50, 9 }, { 59, 12.5 }, { 65, 17.5 }, { 59, 22.5 }, { 50, 26 }, { 55, 17.5 } }) end)
  s:part(M.sun, { round = true, shadow = 0 }, function(m) m:ellipse(59, 17.5, 2.5, 2.5) end)
  s:glow(59, 17, 2.5, M.sun)
  s:part(M.gold, { round = true }, function(m) m:ellipse(16, 17.5, 4.5, 4) end)
  s:part(M.ruby, { round = true, shadow = 0 }, function(m) m:ellipse(15.5, 17, 1.8, 1.8) end)
  s:glow(15, 16, 1, M.ruby)
  s:turret("laser", 28, 13, M.gold, 2, true, M.sun)
  s:engine(2, 12, 4, 11, M.gold, true)
  if cmd then crown(s, 36, 17.5, 5) end
  return s:finish()
end

-- Battleship "Basilica": a great marble delta carrying a nave of domes.
T.battleship = function(cmd)
  local s = F.new(size("battleship"))
  s.glowRamp = FL
  s:part(M.marble, { bevel = 2 }, function(m) m:poly({ { 6, 16 }, { 20, 2 }, { 34, 0 }, { 60, 16 } }, true) end)
  s:part(M.gold, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 20, 2 }, { 34, 0 }, { 60, 16 }, { 55, 16 }, { 32, 3 }, { 21, 4.5 } }, true) end)
  stainedGlass(s, 24, 44, 8, true)
  for _, x in ipairs({ 26, 36, 46 }) do s:turret("laser", x, 11.5 - (x - 26) * 0.12, M.gold, 1, true, M.sun) end
  s:part(M.gold, { bevel = 2 }, function(m) m:poly({ { 2, 14 }, { 62, 13 }, { 86, 21.5 }, { 62, 30 }, { 2, 29 } }) end)
  s:part(M.marble, { bevel = 2, shadow = 0 }, function(m) m:poly({ { 8, 16 }, { 58, 15.5 }, { 74, 21.5 }, { 58, 27.5 }, { 8, 27 } }) end)
  for _, d in ipairs({ { 20, 5 }, { 34, 6 }, { 48, 4.5 } }) do
    s:part(M.gold, { round = true }, function(m) m:ellipse(d[1], 21.5, d[2] + 0.5, d[2]) end)
    s:part(M.ruby, { round = true, shadow = 0 }, function(m) m:ellipse(d[1] - 0.5, 21, d[2] * 0.4, d[2] * 0.4) end)
    s:glow(d[1] - 1, 20, 1, M.ruby)
  end
  s:px(85, 21, M.sun[5]); s:glow(84, 21, 2.5, M.sun)
  s:engine(2, 15, 5, 12, M.gold, true); s:engine(2, 20, 3, 10, M.gold)
  if cmd then crown(s, 62, 21.5, 6) end
  return s:finish()
end

-- Carrier "Reliquary": a marble hall lined with arched hangars around a relic dome.
T.carrier = function(cmd)
  local s = F.new(size("carrier"))
  s.glowRamp = FL
  -- swept wings behind the hall
  s:part(M.marble, { bevel = 2 }, function(m) m:poly({ { 4, 10 }, { 16, 0 }, { 30, 0 }, { 40, 8 } }, true) end)
  s:part(M.gold, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 16, 0 }, { 30, 0 }, { 31, 2 }, { 17, 2.5 } }, true) end)
  s:part(M.gold, { bevel = 2 }, function(m) m:poly({ { 6, 6 }, { 74, 5 }, { 84, 11 }, { 6, 13 } }, true) end)
  s:part(M.marble, { bevel = 2 }, function(m) m:poly({ { 6, 11 }, { 72, 9 }, { 96, 18 }, { 104, 25.5 }, { 96, 33 }, { 72, 42 }, { 6, 40 } }) end)
  -- arched hangars along both flanks, lit inside
  for x = 14, 62, 12 do
    s:map(x, 11, { ".gggg.", "g####g", "g#ss#g", "g#ss#g" }, { g = M.gold[6], ["#"] = M.gold[1], s = M.sun[2] }, true)
  end
  stainedGlass(s, 10, 70, 7, true)
  -- relic dome
  s:part(M.gold, { round = true }, function(m) m:ellipse(44, 25.5, 11, 8.5) end)
  s:part(M.ruby, { round = true, shadow = 0 }, function(m) m:ellipse(43, 25, 5, 4.5) end)
  s:glow(42, 24, 2.5, M.ruby)
  s:part(M.gold, { bevel = 2 }, function(m) m:poly({ { 78, 19 }, { 104, 25.5 }, { 78, 32 }, { 84, 25.5 } }) end)
  s:px(103, 25, M.sun[5]); s:glow(102, 25, 2, M.sun)
  for _, x in ipairs({ 24, 64 }) do s:turret("laser", x, 25.5, M.gold, 2, false, M.sun) end
  s:turret("laser", 50, 6.5, M.gold, 1, true, M.sun)
  s:engine(6, 13, 6, 13, M.gold, true); s:engine(6, 22, 7, 15, M.gold)
  if cmd then crown(s, 44, 25.5, 7) end
  return s:finish()
end

-- Dreadnought "Inquisitor": a lance held in flying buttresses, a ruby prism at the tip.
T.dreadnought = function(cmd)
  local s = F.new(size("dreadnought"))
  s.glowRamp = FL
  s:part(M.gold, { bevel = 2 }, function(m) m:arc(48, 25.5, 25, 3, 1.1, math.pi, true) end)
  s:part(M.marble, { bevel = 1 }, function(m) m:arc(70, 25.5, 21, 2.5, 0.75, math.pi, true) end)
  s:part(M.gold, { bevel = 1, tone = -0.1 }, function(m) m:bar(59, 4, 63, 18, 1.2, true); m:bar(85, 12, 82, 18, 1.2, true) end)
  for _, p in ipairs({ { 59, 3 }, { 85, 10 } }) do
    s:part(M.gold, { round = true }, function(m) m:ellipse(p[1], p[2] + 1.5, 2.5, 2, true) end)
    s:glow(p[1], p[2] + 1, 1, M.sun, true)
  end
  s:part(M.gold, { bevel = 2 }, function(m) m:poly({ { 4, 18 }, { 80, 17 }, { 106, 25.5 }, { 80, 34 }, { 4, 33 } }) end)
  s:part(M.marble, { bevel = 2, shadow = 0 }, function(m) m:poly({ { 10, 20.5 }, { 76, 20 }, { 90, 25.5 }, { 76, 31 }, { 10, 30.5 } }) end)
  for _, x in ipairs({ 18, 28, 38, 48, 58 }) do
    s:part(M.gold, { round = true }, function(m) m:ellipse(x, 25.5, 2.8, 2.5) end)
    s:px(x - 1, 24, M.ruby[3])
  end
  -- the prism
  s:part(M.ruby, { round = true, tone = 0.1 }, function(m) m:poly({ { 84, 16 }, { 100, 25.5 }, { 84, 35 }, { 77, 25.5 } }) end)
  s:line(84, 18, 96, 25, M.ruby[5])
  s:glow(90, 25, 4, M.ruby)
  s:px(111, 25, M.sun[5]); s:glow(110, 25, 2.5, M.sun)
  s:turret("laser", 66, 12, M.gold, 2, true, M.sun)
  s:engine(4, 19, 6, 15, M.gold, true)
  if cmd then crown(s, 38, 25.5, 7) end
  return s:finish()
end

-- Titan "Sunspire": a crescent cradling a captive sun, pierced by a lance.
T.titan = function(cmd)
  local s = F.new(size("titan"))
  s.glowRamp = FL
  s:part(M.gold, { bevel = 3 }, function(m)
    m:arc(56, 27.5, 27.5, function(t) return 2 + 7 * math.sin(math.pi * math.min(1, t * 1.15)) end, 0.75, math.pi + 0.01, true)
  end)
  s:part(M.marble, { bevel = 1, shadow = 0 }, function(m)
    m:arc(56, 27.5, 24, function(t) return 1 + 2.5 * math.sin(math.pi * math.min(1, t * 1.15)) end, 0.95, math.pi - 0.3, true)
  end)
  for i = 0, 10 do
    local a = 1.05 + i * 0.19
    s:px(math.floor(56 + math.cos(a) * 25.5), math.floor(27.5 - math.sin(a) * 25.5), i % 2 == 0 and M.ruby[3] or M.gold[6], true)
  end
  s:part(M.gold, { bevel = 2 }, function(m) m:poly({ { 8, 22 }, { 92, 21 }, { 124, 27.5 }, { 92, 34 }, { 8, 33 } }) end)
  s:part(M.marble, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 12, 24 }, { 88, 23.5 }, { 104, 27.5 }, { 88, 31.5 }, { 12, 31 } }) end)
  for _, a in ipairs({ 1.2, 1.75 }) do
    local x, y = 56 + math.cos(a) * 23, 27.5 - math.sin(a) * 23
    s:part(M.gold, { round = true }, function(m) m:ellipse(x, y, 3.5, 3, true) end)
    s:turret("laser", math.floor(x), y, M.gold, 2, true, M.sun)
  end
  s:part(M.sun, { round = true, shadow = 0, tone = 0.15 }, function(m) m:ellipse(54, 27.5, 9, 9) end)
  s:glow(54, 27, 5, M.sun)
  for i = 0, 7 do
    local a = i * math.pi / 4
    s:line(math.floor(54 + math.cos(a) * 10), math.floor(27 + math.sin(a) * 10), math.floor(54 + math.cos(a) * 13), math.floor(27 + math.sin(a) * 13), M.sun[3])
  end
  for _, x in ipairs({ 22, 32, 74 }) do s:turret("laser", x, 27.5, M.gold, 2, false, M.sun) end
  s:px(123, 27, M.sun[5]); s:glow(122, 27, 3, M.sun)
  s:engine(8, 23, 4, 14, M.gold, true); s:engine(8, 27, 2, 12, M.gold)
  if cmd then
    -- a second, inner halo of rays around the sun marks the command titan
    s:part(M.gold, { bevel = 1 }, function(m) m:arc(54, 27.5, 16, 1.5, -math.pi, math.pi) end)
    s:glow(54, 27, 6, M.sun)
  end
  return s:finish()
end

-- ---------------------------------------------------------------------------
-- Logistics: censers and sun-hearts that pour light into wounded hulls.
-- ---------------------------------------------------------------------------

-- Logi Frigate "Cantor": a lance carrying two censer pods on swept arms.
T.logi_frigate = function()
  local s = F.new(size("logi_frigate"))
  s.glowRamp = FL
  s:part(M.gold, { bevel = 1, tone = -0.1 }, function(m) m:bar(12, 8, 21, 3, 1.1, true) end)
  s:part(M.gold, { round = true }, function(m) m:ellipse(22, 3, 3.2, 2.6, true) end)
  s:part(M.gold, { bevel = 1 }, function(m) m:poly({ { 2, 8 }, { 22, 7.5 }, { 34, 10.5 }, { 22, 13.5 }, { 2, 13 } }) end)
  s:part(M.marble, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 6, 9 }, { 18, 8.6 }, { 26, 10.5 }, { 18, 12.4 }, { 6, 12 } }) end)
  s:part(M.mend, { round = true, shadow = 0 }, function(m) m:ellipse(22, 3, 1.4, 1.2, true) end)
  s:glow(22, 2, 1.5, M.mend, true)
  s:px(25, 10, M.sun[4]); s:glow(25, 10, 1, M.sun)
  s:engine(2, 9, 3, 8, M.gold)
  return s:finish()
end

-- Logi Cruiser "Hospitaller": a nave bearing a sunburst disc and two healing arms.
T.logi_cruiser = function()
  local s = F.new(size("logi_cruiser"))
  s.glowRamp = FL
  s:part(M.marble, { bevel = 2 }, function(m) m:poly({ { 6, 13 }, { 18, 4 }, { 30, 3 }, { 28, 13 } }, true) end)
  s:part(M.gold, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 18, 4 }, { 30, 3 }, { 30, 5 }, { 19, 6 } }, true) end)
  s:part(M.gold, { bevel = 1, tone = -0.1 }, function(m) m:bar(40, 12, 49, 5, 1.2, true) end)
  s:part(M.gold, { round = true }, function(m) m:ellipse(50, 5, 3, 2.5, true) end)
  s:part(M.mend, { round = true, shadow = 0 }, function(m) m:ellipse(50, 5, 1.4, 1.2, true) end)
  s:glow(50, 4, 1.8, M.mend, true)
  s:part(M.gold, { bevel = 2 }, function(m) m:poly({ { 2, 10 }, { 40, 9 }, { 52, 15.5 }, { 40, 22 }, { 2, 21 } }) end)
  s:part(M.marble, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 6, 12 }, { 36, 11.5 }, { 44, 15.5 }, { 36, 19.5 }, { 6, 19 } }) end)
  -- sunburst disc
  s:part(M.marble, { round = true }, function(m) m:ellipse(26, 15.5, 7.5, 7.5) end)
  s:part(M.gold, { bevel = 1, shadow = 0 }, function(m) m:arc(26, 15.5, 7.5, 1.3, -math.pi, math.pi) end)
  for i = 0, 7 do
    local a = i * math.pi / 4
    s:line(math.floor(26 + math.cos(a) * 2.5), math.floor(15 + math.sin(a) * 2.5), math.floor(26 + math.cos(a) * 5), math.floor(15 + math.sin(a) * 5), M.gold[5])
  end
  s:part(M.mend, { round = true, shadow = 0, tone = 0.1 }, function(m) m:ellipse(26, 15.5, 2.2, 2.2) end)
  s:glow(26, 15, 3, M.mend)
  s:engine(2, 11, 4, 10, M.gold, true)
  return s:finish()
end

-- Force Auxiliary "Seraphim": six feathered wings around a healing sun.
T.fax = function()
  local s = F.new(size("fax"))
  s.glowRamp = FL
  for i = 2, 0, -1 do
    s:part(i % 2 == 0 and M.marble or M.gold, { bevel = 2 }, function(m)
      m:poly({ { 22 + i * 14, 21 }, { 34 + i * 14, 21 }, { 18 + i * 16, 2 + i * 4 }, { 10 + i * 16, 1 + i * 4 } }, true)
    end)
    s:part(M.gold, { round = true }, function(m) m:ellipse(13 + i * 16, 2.5 + i * 4, 2.5, 2.2, true) end)
    s:glow(13 + i * 16, 2 + i * 4, 1.4, M.mend, true)
  end
  s:part(M.gold, { bevel = 2 }, function(m) m:poly({ { 4, 18 }, { 72, 17 }, { 104, 25.5 }, { 72, 34 }, { 4, 33 } }) end)
  s:part(M.marble, { bevel = 2, shadow = 0 }, function(m) m:poly({ { 10, 20.5 }, { 68, 20 }, { 86, 25.5 }, { 68, 31 }, { 10, 30.5 } }) end)
  stainedGlass(s, 12, 40, 22, true)
  -- the healing sun
  s:part(M.gold, { bevel = 1 }, function(m) m:arc(56, 25.5, 12, 1.6, -math.pi, math.pi) end)
  s:part(M.mend, { round = true, shadow = 0, tone = 0.15 }, function(m) m:ellipse(56, 25.5, 8, 8) end)
  s:glow(56, 25, 5, M.mend)
  for i = 0, 7 do
    local a = i * math.pi / 4 + math.pi / 8
    s:line(math.floor(56 + math.cos(a) * 13), math.floor(25 + math.sin(a) * 13), math.floor(56 + math.cos(a) * 16), math.floor(25 + math.sin(a) * 16), M.sun[3])
  end
  s:px(103, 25, M.sun[5]); s:glow(102, 25, 2, M.sun)
  s:engine(4, 19, 5, 13, M.gold, true)
  return s:finish()
end

return T
