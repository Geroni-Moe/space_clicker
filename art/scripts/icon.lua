-- App icon: a gilded Throne flagship crossing a purple nebula in front of a
-- ringed planet, inside a macOS-style rounded square. Drawn as pixel masters
-- at 16, 32 and 64 px (so small sizes stay readable) and integer-upscaled to
-- every iconset size. scripts/make_icns.sh turns them into VoidArmada.icns.
--   aseprite --batch --script art/scripts/icon.lua

local px = dofile("art/lib/px.lua")
local P = dofile("art/lib/palette.lua")
local F = dofile("art/lib/forge.lua")
local M = dofile("art/lib/materials.lua")

local OUT = "assets/icon/"
local SKY = { "#07080f", "#0d1020", "#151a33", "#22174a", "#33205e" }
local NEB = { "#3a1a5e", "#5a2a86", "#8a46b8", "#c27ae0" }
local PLANET = { "#2a1438", "#4a1f4a", "#7a3446", "#b85a3e", "#e8925a", "#ffd08a" }

--- Rounded-square mask: n px canvas, `inset` px margin, corner radius r.
local function inside(n, inset, r, x, y)
  local lo, hi = inset, n - 1 - inset
  if x < lo or x > hi or y < lo or y > hi then return false end
  local cx = x < lo + r and lo + r or (x > hi - r and hi - r or x)
  local cy = y < lo + r and lo + r or (y > hi - r and hi - r or y)
  return (x - cx) ^ 2 + (y - cy) ^ 2 <= r * r + 0.5
end

--- The hero ship at three levels of detail.
local function hero(n)
  if n >= 64 then
    local s = F.new(40, 19)
    s.glowRamp = M.flame.throne
    s:part(M.gold, { bevel = 1 }, function(m)
      m:arc(17, 9.5, 9.5, function(t) return 1 + 2.5 * math.sin(math.pi * math.min(1, t * 1.2)) end, 0.9, math.pi - 0.15, true)
    end)
    s:part(M.gold, { bevel = 1 }, function(m) m:poly({ { 2, 6.5 }, { 26, 6 }, { 40, 9.5 }, { 26, 13 }, { 2, 12.5 } }) end)
    s:part(M.marble, { bevel = 1, shadow = 0 }, function(m) m:poly({ { 6, 7.8 }, { 24, 7.6 }, { 32, 9.5 }, { 24, 11.4 }, { 6, 11.2 } }) end)
    s:part(M.sun, { round = true, shadow = 0, tone = 0.15 }, function(m) m:ellipse(15, 9.5, 3, 3) end)
    s:glow(15, 9, 2.5, M.sun)
    s:px(39, 9, M.sun[5]); s:glow(39, 9, 1.5, M.sun)
    s:engine(2, 8, 3, 9, M.gold)
    return s:finish()[2]
  elseif n >= 32 then
    local s = F.new(20, 9)
    s.glowRamp = M.flame.throne
    s:part(M.gold, { bevel = 1 }, function(m) m:poly({ { 4, 4 }, { 9, 4 }, { 13, 0 }, { 10, 0 } }, true) end)
    s:part(M.gold, { bevel = 1 }, function(m) m:poly({ { 1, 3 }, { 13, 2.5 }, { 20, 4.5 }, { 13, 6.5 }, { 1, 6 } }) end)
    s:px(8, 4, M.sun[5]); s:px(19, 4, M.sun[5])
    s:engine(1, 3, 3, 5, M.gold)
    return s:finish()[2]
  end
  -- 16 px: a bright dart and its flame, nothing more
  local img = px.image(12, 5)
  px.map(img, 0, 0, {
    "...gg.......",
    "ofggGGGgg...",
    "ofgGGGGGGGw.",
    "ofggGGGgg...",
    "...gg.......",
  }, { g = M.gold[5], G = M.gold[6], w = "#ffffff", o = M.flame.throne[2], f = M.flame.throne[3] })
  return img
end

local function render(n)
  local img = px.image(n, n)
  local inset = math.max(1, math.floor(n * 6 / 64 + 0.5))
  local r = math.floor(n * 11 / 64 + 0.5)
  for y = 0, n - 1 do
    for x = 0, n - 1 do
      if inside(n, inset, r, x, y) then
        -- sky gradient, deepening toward the top
        local t = (y - inset) / (n - 2 * inset)
        local col = px.ramp(SKY, 0.25 + t * 0.7, x, y)
        -- nebula band across the middle
        local neb = px.fbm(x * 64 / n, y * 64 / n, 3, 18, 3) - math.abs(t - 0.5) * 0.9
        if neb > 0.32 then col = NEB[math.min(#NEB, 1 + math.floor((neb - 0.32) * 9))] end
        px.set(img, x, y, col)
      end
    end
  end
  -- stars
  local rnd = px.rng(17)
  for _ = 1, math.floor(n * n / 90) do
    local x, y = rnd(0, n - 1), rnd(0, n - 1)
    if inside(n, inset + 1, r, x, y) then px.set(img, x, y, rnd(1, 3) == 1 and "#b5f6ff" or "#dbe7f5") end
  end
  -- ringed planet in the lower right
  local pcx, pcy, pr = n * 0.74, n * 0.76, n * 0.25
  for y = 0, n - 1 do
    for x = 0, n - 1 do
      local dx, dy = (x + 0.5 - pcx) / pr, (y + 0.5 - pcy) / pr
      local d2 = dx * dx + dy * dy
      if d2 <= 1 and inside(n, inset, r, x, y) then
        -- lit from the upper left, banded
        local light = 0.55 - dx * 0.45 - dy * 0.5 + math.sin((y + 0.5) / pr * 9) * 0.06
        local k = math.max(1, math.min(#PLANET, math.floor(light * #PLANET) + 1))
        px.set(img, x, y, PLANET[k])
      end
    end
  end
  if n >= 32 then
    for x = 0, n - 1 do
      local t = (x + 0.5 - pcx) / (pr * 1.7)
      local y = math.floor(pcy - t * pr * 0.55 + 0.5)
      if math.abs(t) <= 1 and inside(n, inset, r, x, y) then
        local behind = ((x + 0.5 - pcx) / pr) ^ 2 + ((y + 0.5 - pcy) / pr) ^ 2 <= 1 and t < 0
        if not behind then px.set(img, x, y, "#f6e2b0") end
      end
    end
  end
  -- the hero, nose toward the upper right of centre
  local ship = hero(n)
  local sx, sy = math.floor((n - ship.width) / 2 - n * 0.02), math.floor(n * 0.36 - ship.height / 2)
  for y = 0, ship.height - 1 do
    for x = 0, ship.width - 1 do
      -- clipped to the rounded square, so the flame trails in from the edge
      if px.opaque(ship, x, y) and inside(n, inset, r, sx + x, sy + y) then
        px.set(img, sx + x, sy + y, px.get(ship, x, y))
      end
    end
  end
  -- rim light on the top edge, dark edge below so it reads on any dock
  for y = 0, n - 1 do
    for x = 0, n - 1 do
      if inside(n, inset, r, x, y) then
        if not inside(n, inset, r, x, y - 1) then px.set(img, x, y, "#5a6699") end
        if not inside(n, inset, r, x, y + 1) or not inside(n, inset, r, x - 1, y) or not inside(n, inset, r, x + 1, y) then
          if inside(n, inset, r, x, y - 1) then px.set(img, x, y, "#0a0b14") end
        end
      elseif inside(n, inset, r, x, y - 1) and n >= 32 then
        px.set(img, x, y, "#00000060") -- soft drop shadow
      end
    end
  end
  return img
end

local function save(img, name)
  local spr = Sprite(img.width, img.height, ColorMode.RGB)
  spr.cels[1].image = img
  spr:saveCopyAs(OUT .. name)
  spr:close()
end

local masters = { [16] = render(16), [32] = render(32), [64] = render(64) }
-- size -> master it is drawn from
for _, size in ipairs({ 16, 32, 64, 128, 256, 512, 1024 }) do
  local m = size <= 64 and masters[size] or masters[64]
  local img = size == m.width and m or px.scale(m, size // m.width)
  save(img, "icon_" .. size .. ".png")
  print("[icon] " .. size)
end
-- editable source of the 64 px master
local src = Sprite(64, 64, ColorMode.RGB)
src.cels[1].image = masters[64]
src:saveAs("art/src/icon.aseprite")
src:close()
