-- Combat effects and projectiles.
--   aseprite --batch --script art/scripts/fx.lua

local px = dofile("art/lib/px.lua")
local P = dofile("art/lib/palette.lua")

local preview = {}
local function out(frames, name, dur)
  if type(frames) == "userdata" then frames = { frames } end
  px.export(frames, name, { duration = dur or 0.06 })
  preview[#preview + 1] = frames
end

-- ---------------------------------------------------------------------------
-- Explosions: a noisy heat field that expands, cools and breaks into smoke.
-- ---------------------------------------------------------------------------
--- Explosion: a white flash, a banded fireball of several lobes thrown
--- outward, a broken shockwave ring and flying sparks, then lit smoke puffs
--- that drift apart and thin out. Flat colour bands, no dithering on the fire.
local function explosion(size, nframes, seed)
  local frames = {}
  local c = size / 2
  local fire = P.fire
  local rnd = px.rng(seed)
  local lobes, smoke, sparks = {}, {}, {}
  for i = 1, 7 do
    local a = (i / 7) * math.pi * 2 + rnd() * 0.8
    lobes[i] = { a = a, d = rnd() * 0.12, r = 0.6 + rnd() * 0.45 }
  end
  lobes[8] = { a = 0, d = 0, r = 1.15 }
  for i = 1, 6 do
    local a = rnd() * math.pi * 2
    smoke[i] = { a = a, d = 0.05 + rnd() * 0.12, r = 0.55 + rnd() * 0.35 }
  end
  for i = 1, 12 do
    sparks[i] = { a = rnd() * math.pi * 2, v = 0.6 + rnd() * 0.5 }
  end
  local SMOKE = { "#24242e", "#3a3a48", "#565666", "#7a7a8a" }
  for f = 1, nframes do
    local t = (f - 1) / (nframes - 1)
    local img = px.image(size, size)
    -- smoke first, so fire sits on top of it
    if t > 0.3 then
      local st = (t - 0.3) / 0.7
      for y = 0, size - 1 do
        for x = 0, size - 1 do
          local best, lit = 0, 0
          for _, p in ipairs(smoke) do
            local px_, py = c + math.cos(p.a) * size * (p.d + st * 0.18), c + math.sin(p.a) * size * (p.d + st * 0.18) - st * size * 0.05
            local r = size * p.r * (0.16 + 0.12 * st)
            local dx, dy = x + 0.5 - px_, y + 0.5 - py
            local v = 1 - math.sqrt(dx * dx + dy * dy) / r
            if v > best then best, lit = v, (-dx - dy) / r end
          end
          local edge = 0.08 + st * 0.45 + (px.fbm(x, y, seed + 7, 4, 2) - 0.5) * 0.25
          if best > edge then
            local k = math.floor(math.max(0, math.min(3.99, 1.2 + lit * 1.6 + best * 1.2 - st * 1.2))) + 1
            -- late smoke thins to a checker so it fades into space
            if st < 0.75 or (x + y + f) % 2 == 0 then px.set(img, x, y, SMOKE[k]) end
          end
        end
      end
    end
    -- fireball lobes, cooling and breaking apart with age
    local grow = 0.2 + 0.26 * math.sqrt(t)
    local shrink = math.max(0, 1 - math.max(0, t - 0.35) * 1.6)
    if shrink > 0 then
      for y = 0, size - 1 do
        for x = 0, size - 1 do
          local heat = -1
          for _, p in ipairs(lobes) do
            local lx = c + math.cos(p.a) * size * (p.d + t * 0.16)
            local ly = c + math.sin(p.a) * size * (p.d + t * 0.16)
            local r = size * grow * p.r * shrink * (p.d == 0 and 0.7 or 0.55)
            local v = 1 - math.sqrt((x + 0.5 - lx) ^ 2 + (y + 0.5 - ly) ^ 2) / math.max(0.5, r)
            heat = math.max(heat, v)
          end
          -- ragged edges only: noise never lights pixels away from a lobe
          if heat > -0.1 then heat = heat + (px.fbm(x, y, seed + f, 3, 2) - 0.5) * 0.3 end
          local temp = heat * (1.35 - t * 1.5)
          local col
          if heat > 0.02 then
            if temp > 0.8 then col = fire[6]
            elseif temp > 0.6 then col = fire[5]
            elseif temp > 0.4 then col = fire[4]
            elseif temp > 0.22 then col = fire[3]
            elseif temp > 0.06 then col = fire[2] end
          end
          if col then px.set(img, x, y, col) end
        end
      end
    end
    -- broken shockwave ring
    if t > 0.05 and t < 0.55 then
      local rr = size * (0.18 + 0.62 * t / 0.55)
      local col = t < 0.3 and "#fff6d6" or "#ffc75a"
      for k = 0, 179 do
        local a = k / 180 * math.pi * 2
        if px.fbm(math.cos(a) * 9 + 9, math.sin(a) * 9 + 9, seed + 3, 4, 1) > 0.42 then
          px.set(img, c + math.cos(a) * rr, c + math.sin(a) * rr, col)
        end
      end
    end
    -- sparks with short trails
    if t > 0.08 and t < 0.9 then
      for _, sp in ipairs(sparks) do
        local d = size * 0.5 * sp.v * math.sqrt(t)
        local x, y = c + math.cos(sp.a) * d, c + math.sin(sp.a) * d
        px.set(img, x, y, t < 0.5 and fire[6] or fire[5])
        px.set(img, x - math.cos(sp.a) * 1.5, y - math.sin(sp.a) * 1.5, fire[4])
      end
    end
    if f == 1 then
      px.disc(img, c, c, size * 0.2, fire[5])
      px.disc(img, c, c, size * 0.13, fire[6])
      px.line(img, c - size * 0.32, c, c + size * 0.32, c, fire[6])
      px.line(img, c, c - size * 0.32, c, c + size * 0.32, fire[6])
    end
    frames[f] = img
  end
  return frames
end

out(explosion(32, 8, 11), "explosion_small", 0.06)
out(explosion(56, 10, 23), "explosion_big", 0.07)

-- Sparks: a 4-point star that shrinks.
do
  local frames = {}
  local ramp = { "#ffffff", "#fff2b0", "#f2c94c", "#c98a1c" }
  for f = 1, 4 do
    local img = px.image(11, 11)
    local r = 5 - f
    local col = ramp[f]
    for i = -r, r do px.set(img, 5 + i, 5, col); px.set(img, 5, 5 + i, col) end
    if f <= 2 then
      px.set(img, 4, 4, ramp[f + 1]); px.set(img, 6, 6, ramp[f + 1])
      px.set(img, 4, 6, ramp[f + 1]); px.set(img, 6, 4, ramp[f + 1])
    end
    px.set(img, 5, 5, "#ffffff")
    frames[f] = img
  end
  out(frames, "spark")
end

-- Shield impact: a cyan arc facing the hit.
do
  local frames = {}
  for f = 1, 5 do
    local img = px.image(12, 24)
    local ramp = { P.cyan[4], P.cyan[3], P.cyan[3], P.cyan[2], P.cyan[1] }
    local r = 9 + f
    for y = 0, 23 do
      for x = 0, 11 do
        local d = math.sqrt((x + 0.5 - (12 - r)) ^ 2 + ((y + 0.5 - 12) * 1.1) ^ 2)
        if d <= r and d > r - (f <= 2 and 2 or 1) and math.abs(y - 11.5) < 11 - f then
          if f < 4 or (x + y) % 2 == 0 then px.set(img, x, y, ramp[f]) end
        end
      end
    end
    frames[f] = img
  end
  out(frames, "shield_hit")
end

-- Credit chip: spinning round coin.
do
  local frames = {}
  local widths = { 7, 6, 4, 2, 4, 6 }
  for f = 1, 6 do
    local img = px.image(9, 9)
    local w = widths[f]
    px.ellipse(img, 4.5, 4.5, w / 2, 3.5, P.gold[2])
    if w >= 4 then
      px.ellipse(img, 4.5, 4.5, w / 2 - 1, 2.5, P.gold[3])
      px.set(img, 4, 3, P.gold[4]); px.set(img, 4, 4, P.gold[4]); px.set(img, 4, 5, P.gold[2])
    else
      px.ellipse(img, 4.5, 4.5, w / 2, 3.5, P.gold[3])
    end
    if f == 1 then px.set(img, 3, 2, "#ffffff") end
    px.outline(img, P.outline)
    frames[f] = img
  end
  out(frames, "credit", 0.08)
end

-- Smoke puff.
do
  local frames = {}
  for f = 1, 5 do
    local img = px.image(10, 10)
    local r = 1.5 + f * 0.7
    for y = 0, 9 do
      for x = 0, 9 do
        local d = math.sqrt((x + 0.5 - 5) ^ 2 + (y + 0.5 - 5) ^ 2)
        if d < r and ((f < 4) or (x + y + f) % 2 == 0) then
          px.set(img, x, y, px.ramp(P.smoke, 1 - d / r - f * 0.12 + 0.4, x, y))
        end
      end
    end
    frames[f] = img
  end
  out(frames, "smoke")
end

-- Muzzle flash.
do
  local frames = {}
  for f = 1, 3 do
    local img = px.image(8, 8)
    local r = 4 - f
    px.disc(img, 4, 4, r + 0.5, P.engine[3])
    px.disc(img, 4, 4, r - 0.5, "#ffffff")
    if f == 1 then
      px.line(img, 4, 0, 4, 7, P.engine[3]); px.line(img, 0, 4, 7, 4, P.engine[3])
      px.set(img, 4, 4, "#ffffff")
    end
    frames[f] = img
  end
  out(frames, "muzzle")
end

-- Warp-in streak: a long bright smear collapsing into a flash.
do
  local frames = {}
  for f = 1, 6 do
    local img = px.image(64, 16)
    local len = math.floor(64 * (1 - (f - 1) / 6))
    local th = f <= 4 and 1 or 0
    for x = 64 - len, 63 do
      local t = (x - (64 - len)) / math.max(1, len)
      local col = t > 0.85 and "#ffffff" or (t > 0.5 and P.purple[4] or P.purple[3])
      for dy = -th, th do px.set(img, x, 8 + dy, col) end
      if t > 0.9 then px.set(img, x, 6, P.purple[3]); px.set(img, x, 9 + th, P.purple[3]) end
    end
    if f >= 3 then
      px.disc(img, 58, 8, 7 - f, P.purple[4])
      px.disc(img, 58, 8, 5 - f, "#ffffff")
    end
    frames[f] = img
  end
  out(frames, "warp_in", 0.05)
end

-- Click reticle: snapping brackets that expand and fade.
do
  local frames = {}
  local cols = { "#ffffff", P.cyan[4], P.cyan[3], P.cyan[2], P.cyan[1] }
  for f = 1, 5 do
    local img = px.image(21, 21)
    local r = 3 + f * 1.6
    local c = 10
    local col = cols[f]
    local L = 2
    for _, s in ipairs({ { -1, -1 }, { 1, -1 }, { -1, 1 }, { 1, 1 } }) do
      local cx, cy = c + s[1] * math.floor(r), c + s[2] * math.floor(r)
      for i = 0, L do
        px.set(img, cx - s[1] * i, cy, col)
        px.set(img, cx, cy - s[2] * i, col)
      end
    end
    if f <= 2 then px.set(img, c, c, col) end
    frames[f] = img
  end
  out(frames, "reticle", 0.05)
end

-- ---------------------------------------------------------------------------
-- Projectiles
-- ---------------------------------------------------------------------------
local function bolt(len, th, ramp)
  local img = px.image(len, th * 2 + 1)
  for x = 0, len - 1 do
    local t = x / (len - 1)
    px.set(img, x, th, t > 0.7 and "#ffffff" or ramp[3])
    if th >= 1 then
      local c = t > 0.5 and ramp[3] or ramp[2]
      if x > 1 then px.set(img, x, th - 1, c); px.set(img, x, th + 1, c) end
    end
    if th >= 2 and x > 3 and x < len - 1 then
      px.set(img, x, th - 2, ramp[1]); px.set(img, x, th + 2, ramp[1])
    end
  end
  return img
end

out(bolt(8, 1, P.cyan), "bolt_cyan")
out(bolt(10, 1, P.engine), "bolt_blue")
out(bolt(14, 2, P.cyan), "bolt_heavy")
out(bolt(8, 1, P.red), "bolt_red")
out(bolt(8, 1, P.green), "bolt_green")
out(bolt(8, 1, P.purple), "bolt_purple")
out(bolt(8, 1, P.gold), "bolt_gold")

-- Plasma orb (2 frames, pulsing).
do
  local frames = {}
  for f = 1, 2 do
    local img = px.image(9, 9)
    px.disc(img, 4.5, 4.5, 4 - (f - 1) * 0.5, P.green[2])
    px.disc(img, 4.5, 4.5, 3 - (f - 1) * 0.5, P.green[3])
    px.disc(img, 4.5, 4.5, 1.5, P.green[4])
    px.set(img, 3, 3, "#ffffff")
    frames[f] = img
  end
  out(frames, "plasma")
end

-- Missile with flicker exhaust.
do
  local frames = {}
  for f = 1, 2 do
    local img = px.image(12, 5)
    px.rect(img, 4, 1, 6, 3, P.steel[3])
    px.rect(img, 4, 1, 6, 1, P.steel[4])
    px.set(img, 10, 2, P.red[3]); px.set(img, 11, 2, P.red[3]); px.set(img, 10, 1, P.red[2]); px.set(img, 10, 3, P.red[2])
    px.set(img, 4, 0, P.steel[2]); px.set(img, 4, 4, P.steel[2]); px.set(img, 5, 0, P.steel[2]); px.set(img, 5, 4, P.steel[2])
    px.set(img, 3, 2, "#ffffff")
    px.set(img, 2, 2, P.fire[5]); px.set(img, 1, 2, P.fire[4])
    if f == 2 then px.set(img, 0, 2, P.fire[3]); px.set(img, 2, 1, P.fire[4]); px.set(img, 2, 3, P.fire[4]) end
    frames[f] = img
  end
  out(frames, "missile")
end

-- Heavy shell.
do
  local img = px.image(7, 5)
  px.poly(img, { { 0, 1 }, { 5, 1 }, { 7, 2.5 }, { 5, 4 }, { 0, 4 } }, P.gold[2])
  px.rect(img, 0, 1, 5, 1, P.gold[3])
  px.set(img, 5, 2, P.gold[4])
  px.rect(img, 0, 3, 5, 1, P.gold[1])
  out(img, "shell")
end

-- Strike-craft launched by carriers (2 frames).
do
  local frames = {}
  for f = 1, 2 do
    local img = px.image(9, 7)
    px.map(img, 2, 1, {
      "33....",
      ".2443.",
      "e12445",
      ".2443.",
      "33....",
    }, { ["2"] = P.steel[2], ["3"] = P.steel[3], ["4"] = P.steel[4], ["5"] = P.cyan[3], ["1"] = P.steel[1], e = P.engine[3] })
    px.set(img, 1, 3, f == 1 and P.engine[2] or P.engine[3])
    if f == 2 then px.set(img, 0, 3, P.engine[1]) end
    frames[f] = img
  end
  out(frames, "strike_craft")
end

-- Beam segment: a vertical cross-section stretched along x in-game.
do
  local function beam(ramp, name)
    local img = px.image(4, 9)
    local rows = { ramp[1], ramp[2], ramp[3], "#ffffff", "#ffffff", "#ffffff", ramp[3], ramp[2], ramp[1] }
    for y = 0, 8 do
      for x = 0, 3 do
        if (y == 0 or y == 8) and (x % 2 == 1) then
          -- broken outer fringe gives the beam a crackle when animated
        else
          px.set(img, x, y, rows[y + 1])
        end
      end
    end
    out(img, name)
  end
  beam(P.cyan, "beam_cyan")
  beam(P.purple, "beam_purple")
  beam(P.red, "beam_red")
  beam(P.gold, "beam_gold")
  beam(P.green, "beam_green")
end

-- Debris chunks (3 variants as frames).
do
  local frames = {}
  local shapes = {
    { ".33", "324", "21." },
    { "34.", "232", ".21" },
    { ".3.", "342", "221" },
  }
  for f = 1, 3 do
    local img = px.image(5, 5)
    px.map(img, 1, 1, shapes[f], { ["1"] = P.smoke[2], ["2"] = P.smoke[3], ["3"] = P.smoke[4], ["4"] = P.fire[5] })
    frames[f] = img
  end
  out(frames, "debris")
end

-- ---------------------------------------------------------------------------
-- Contact sheet
-- ---------------------------------------------------------------------------
local k, maxW = 4, 1200
local x, y, rowH, placed = 4, 4, 0, {}
for _, frames in ipairs(preview) do
  local fw, fh = frames[1].width * k, frames[1].height * k
  local w = (fw + 4) * #frames
  if x + w > maxW then x = 4; y = y + rowH + 10; rowH = 0 end
  for i, f in ipairs(frames) do placed[#placed + 1] = { img = f, x = x + (i - 1) * (fw + 4), y = y } end
  x = x + w + 12
  rowH = math.max(rowH, fh)
end
local H = y + rowH + 4
local sheet = px.image(maxW, H)
px.rect(sheet, 0, 0, maxW, H, P.space[2])
for _, p in ipairs(placed) do px.stamp(sheet, px.scale(p.img, k), p.x, p.y) end
local spr = Sprite(maxW, H, ColorMode.RGB)
spr.cels[1].image = sheet
spr:saveCopyAs("art/previews/fx.png")
spr:close()
