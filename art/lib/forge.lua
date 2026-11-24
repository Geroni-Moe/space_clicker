-- forge.lua - hand-composed ship builder with height-field lighting (nose right).
--
-- A ship is a stack of parts. Each part is a mask (polygons, ellipses, arcs,
-- with optional mirroring and cut-outs) painted with a material ramp. Paint
-- builds a bevelled height field from the mask, lights it from the top-left
-- and maps the result onto the ramp, so every plate gets chamfered edges,
-- highlights and core shadows for free. Parts cast a soft shadow onto what
-- they sit on.
--
-- Coordinates are pixel edges in ship space: x in [0, w] rear -> nose,
-- y in [0, h] top -> bottom. Mirroring maps y to h - y (odd h keeps a
-- centre row).

local px = dofile("art/lib/px.lua")
local P = dofile("art/lib/palette.lua")
local pc = app.pixelColor

local F = {}
F.__index = F

local MARGIN = 16 -- room behind the hull for engine flames
local LIGHT = { -0.42, -0.72, 0.56 } -- toward the light: left, up, out of screen
do
  local l = math.sqrt(LIGHT[1] ^ 2 + LIGHT[2] ^ 2 + LIGHT[3] ^ 2)
  LIGHT = { LIGHT[1] / l, LIGHT[2] / l, LIGHT[3] / l }
end

function F.new(w, h, glow)
  local s = setmetatable({}, F)
  s.w, s.h = w, h
  s.ox, s.oy = MARGIN + 2, 3
  s.W, s.H = w + MARGIN + 6, h + 6
  s.img = px.image(s.W, s.H)
  s.glowRamp = glow
  s.engines, s.glows, s.post = {}, {}, {}
  return s
end

-- ---------------------------------------------------------------------------
-- Masks
-- ---------------------------------------------------------------------------
local Mask = {}
Mask.__index = Mask

function F:mask()
  return setmetatable({ s = self, W = self.W, H = self.H, bits = {} }, Mask)
end

function Mask:set(x, y, v)
  if x >= 0 and y >= 0 and x < self.W and y < self.H then self.bits[y * self.W + x] = v or nil end
end
function Mask:has(x, y)
  if x < 0 or y < 0 or x >= self.W or y >= self.H then return false end
  return self.bits[y * self.W + x] == true
end

local function polyFill(m, pts, v)
  local s = m.s
  local miny, maxy = math.huge, -math.huge
  for _, p in ipairs(pts) do miny = math.min(miny, p[2]); maxy = math.max(maxy, p[2]) end
  for y = math.floor(miny), math.ceil(maxy) do
    local sy = y + 0.5
    local xs = {}
    for i = 1, #pts do
      local a, b = pts[i], pts[i % #pts + 1]
      if (a[2] <= sy and b[2] > sy) or (b[2] <= sy and a[2] > sy) then
        xs[#xs + 1] = a[1] + (sy - a[2]) / (b[2] - a[2]) * (b[1] - a[1])
      end
    end
    table.sort(xs)
    for i = 1, #xs - 1, 2 do
      for x = math.ceil(xs[i] - 0.5), math.floor(xs[i + 1] - 0.5) do m:set(x + s.ox, y + s.oy, v) end
    end
  end
end

local function mirrorPts(s, pts)
  local out = {}
  for i = #pts, 1, -1 do out[#out + 1] = { pts[i][1], s.h - pts[i][2] } end
  return out
end

--- Add (or with cut=true remove) a polygon; m mirrors it.
function Mask:poly(pts, m, cut)
  local v = not cut or nil
  polyFill(self, pts, v)
  if m then polyFill(self, mirrorPts(self.s, pts), v) end
  return self
end
function Mask:rect(x, y, w, h, m, cut) return self:poly({ { x, y }, { x + w, y }, { x + w, y + h }, { x, y + h } }, m, cut) end

function Mask:ellipse(cx, cy, rx, ry, m, cut)
  local s = self.s
  local v = not cut or nil
  local function one(cyy)
    for y = math.floor(cyy - ry - 1), math.ceil(cyy + ry + 1) do
      for x = math.floor(cx - rx - 1), math.ceil(cx + rx + 1) do
        local nx, ny = (x + 0.5 - cx) / rx, (y + 0.5 - cyy) / ry
        if nx * nx + ny * ny <= 1 then self:set(x + s.ox, y + s.oy, v) end
      end
    end
  end
  one(cy)
  if m then one(s.h - cy) end
  return self
end

--- Ring segment: angles in radians, 0 = nose, positive = up. thick may be a
--- function of the normalised position t in [0, 1] along the arc.
function Mask:arc(cx, cy, r, thick, a0, a1, m, cut)
  local s = self.s
  local v = not cut or nil
  local function one(cyy, flip)
    for y = math.floor(cyy - r - 1), math.ceil(cyy + r + 1) do
      for x = math.floor(cx - r - 1), math.ceil(cx + r + 1) do
        local dx, dy = x + 0.5 - cx, -(y + 0.5 - cyy)
        if flip then dy = -dy end
        local d = math.sqrt(dx * dx + dy * dy)
        local a = math.atan(dy, dx)
        if a >= a0 and a <= a1 then
          local th = type(thick) == "function" and thick((a - a0) / (a1 - a0)) or thick
          if d <= r and d > r - th then self:set(x + s.ox, y + s.oy, v) end
        end
      end
    end
  end
  one(cy, false)
  if m then one(s.h - cy, true) end
  return self
end

--- Thick line (capsule) between two points.
function Mask:bar(x0, y0, x1, y1, r, m, cut)
  local s = self.s
  local v = not cut or nil
  local function one(ya, yb)
    local dx, dy = x1 - x0, yb - ya
    local L2 = math.max(1e-6, dx * dx + dy * dy)
    for y = math.floor(math.min(ya, yb) - r - 1), math.ceil(math.max(ya, yb) + r + 1) do
      for x = math.floor(math.min(x0, x1) - r - 1), math.ceil(math.max(x0, x1) + r + 1) do
        local t = math.max(0, math.min(1, ((x + 0.5 - x0) * dx + (y + 0.5 - ya) * dy) / L2))
        local qx, qy = x0 + dx * t, ya + dy * t
        if (x + 0.5 - qx) ^ 2 + (y + 0.5 - qy) ^ 2 <= r * r then self:set(x + s.ox, y + s.oy, v) end
      end
    end
  end
  one(y0, y1)
  if m then one(s.h - y0, s.h - y1) end
  return self
end

function Mask:cut(other)
  for k in pairs(other.bits) do self.bits[k] = nil end
  return self
end

-- ---------------------------------------------------------------------------
-- Painting
-- ---------------------------------------------------------------------------

--- Chamfer distance (in px) from each mask pixel to the nearest outside pixel.
local function distance(m)
  local W, H = m.W, m.H
  local INF = 1e9
  local d = {}
  for y = 0, H - 1 do
    for x = 0, W - 1 do d[y * W + x] = m:has(x, y) and INF or 0 end
  end
  local function g(x, y)
    if x < 0 or y < 0 or x >= W or y >= H then return 0 end
    return d[y * W + x]
  end
  for y = 0, H - 1 do
    for x = 0, W - 1 do
      local i = y * W + x
      if d[i] > 0 then
        d[i] = math.min(d[i], g(x - 1, y) + 1, g(x, y - 1) + 1, g(x - 1, y - 1) + 1.414, g(x + 1, y - 1) + 1.414)
      end
    end
  end
  for y = H - 1, 0, -1 do
    for x = W - 1, 0, -1 do
      local i = y * W + x
      if d[i] > 0 then
        d[i] = math.min(d[i], g(x + 1, y) + 1, g(x, y + 1) + 1, g(x + 1, y + 1) + 1.414, g(x - 1, y + 1) + 1.414)
      end
    end
  end
  return d
end

local function darken(p, f)
  return pc.rgba(math.floor(pc.rgbaR(p) * f), math.floor(pc.rgbaG(p) * f), math.floor(pc.rgbaB(p) * f), pc.rgbaA(p))
end

--- Paint a mask with a material ramp (dark -> light).
--- opts.bevel   chamfer width in px (default 2)
--- opts.round   true = pillowed/cylindrical profile instead of a flat chamfer
--- opts.tone    brightness offset in [-1, 1] (default 0)
--- opts.contrast lighting strength (default 1.3)
--- opts.dither  ordered dithering between ramp steps (default off)
--- opts.shadow  cast shadow length onto existing pixels (default 1, 0 = none)
--- opts.flat    single ramp index, no lighting (for glass, decals)
--- opts.cyl     strength of the top-lit gradient across each column (default 0.45)
function F:paint(m, ramp, opts)
  opts = opts or {}
  local W, H = m.W, m.H
  local B = opts.bevel or 2
  local d = distance(m)
  local hgt = {}
  local maxD = 0
  if opts.round then for _, v in pairs(d) do maxD = math.max(maxD, v) end end
  local function height(x, y)
    if x < 0 or y < 0 or x >= W or y >= H then return 0 end
    local v = d[y * W + x]
    if v == 0 then return 0 end
    if opts.round then
      local t = math.min(1, v / math.max(1, opts.radius or maxD))
      return math.sqrt(1 - (1 - t) ^ 2) * (opts.radius or maxD) * 1.2
    end
    return math.min(v, B)
  end
  for y = 0, H - 1 do for x = 0, W - 1 do hgt[y * W + x] = height(x, y) end end
  local function hh(x, y)
    if x < 0 or y < 0 or x >= W or y >= H then return 0 end
    return hgt[y * W + x]
  end
  local flatI = LIGHT[3]
  local contrast = opts.contrast or 1.3
  local tone = opts.tone or 0
  -- cast shadow first, onto pixels already on the canvas
  local sh = opts.shadow == nil and 1 or opts.shadow
  if sh > 0 then
    for y = 0, H - 1 do
      for x = 0, W - 1 do
        if not m:has(x, y) and px.opaque(self.img, x, y) then
          for k = 1, sh do
            if m:has(x - k, y - k) or m:has(x, y - k) then
              self.img:drawPixel(x, y, darken(self.img:getPixel(x, y), 0.55))
              break
            end
          end
        end
      end
    end
  end
  local cy = self.h / 2 + self.oy
  -- per-column extents, for a cylindrical top-lit gradient across each part
  local cyl = opts.cyl or (opts.round and 0 or 0.45)
  local top, bot = {}, {}
  if cyl ~= 0 then
    for x = 0, W - 1 do
      for y = 0, H - 1 do
        if m:has(x, y) then top[x] = top[x] or y; bot[x] = y end
      end
    end
  end
  for y = 0, H - 1 do
    for x = 0, W - 1 do
      if m:has(x, y) then
        local col
        if opts.flat then
          col = ramp[opts.flat]
        else
          local nx = -(hh(x + 1, y) - hh(x - 1, y)) / 2
          local ny = -(hh(x, y + 1) - hh(x, y - 1)) / 2
          local nz = 1
          local l = math.sqrt(nx * nx + ny * ny + nz * nz)
          local dot = (nx * LIGHT[1] + ny * LIGHT[2] + nz * LIGHT[3]) / l
          -- gentle top-to-bottom falloff across the whole ship
          local fall = -0.12 * (y - cy) / math.max(1, self.h / 2)
          local v = 0.55 + (dot - flatI) * contrast + fall + tone
          if cyl ~= 0 and bot[x] > top[x] + 2 then
            local rel = (y - top[x]) / (bot[x] - top[x])
            v = v + cyl * (0.35 - rel) * (rel > 0.75 and 1.4 or 1)
          end
          if not opts.dither then
            local i = math.floor(math.max(0, math.min(0.999, v)) * #ramp) + 1
            col = ramp[i]
          else
            col = px.ramp(ramp, v, x, y)
          end
        end
        px.set(self.img, x, y, col)
      end
    end
  end
  return m
end

--- Convenience: build and paint in one call. fn(mask) adds the geometry.
function F:part(ramp, opts, fn)
  local m = self:mask()
  fn(m)
  return self:paint(m, ramp, opts)
end

-- ---------------------------------------------------------------------------
-- Detail
-- ---------------------------------------------------------------------------

function F:px(x, y, col, m)
  px.set(self.img, x + self.ox, y + self.oy, col)
  if m then px.set(self.img, x + self.ox, self.h - 1 - y + self.oy, col) end
end
function F:hline(x0, x1, y, col, m) for x = x0, x1 do self:px(x, y, col, m) end end
function F:vline(x, y0, y1, col, m) for y = y0, y1 do self:px(x, y, col, m) end end
function F:rect(x, y, w, h, col, m)
  for yy = y, y + h - 1 do for xx = x, x + w - 1 do self:px(xx, yy, col, m) end end
end
function F:line(x0, y0, x1, y1, col, m)
  px.line(self.img, x0 + self.ox, y0 + self.oy, x1 + self.ox, y1 + self.oy, col)
  if m then px.line(self.img, x0 + self.ox, self.h - 1 - y0 + self.oy, x1 + self.ox, self.h - 1 - y1 + self.oy, col) end
end
function F:dots(x0, x1, y, col, step, m) for x = x0, x1, step or 2 do self:px(x, y, col, m) end end
--- ASCII stamp in ship space; key maps chars to colours.
function F:map(x, y, rows, key, m)
  for r, row in ipairs(rows) do
    for i = 1, #row do
      local ch = row:sub(i, i)
      if ch ~= "." and ch ~= " " then self:px(x + i - 1, y + r - 1, key[ch], false) end
    end
  end
  if m then
    for r, row in ipairs(rows) do
      for i = 1, #row do
        local ch = row:sub(i, i)
        if ch ~= "." and ch ~= " " then self:px(x + i - 1, self.h - 1 - (y + r - 1), key[ch], false) end
      end
    end
  end
end

local DIGITS = {
  ["0"] = { "###", "#.#", "#.#", "#.#", "###" }, ["1"] = { ".#.", "##.", ".#.", ".#.", "###" },
  ["2"] = { "###", "..#", "###", "#..", "###" }, ["3"] = { "###", "..#", ".##", "..#", "###" },
  ["4"] = { "#.#", "#.#", "###", "..#", "..#" }, ["5"] = { "###", "#..", "###", "..#", "###" },
  ["6"] = { "###", "#..", "###", "#.#", "###" }, ["7"] = { "###", "..#", ".#.", ".#.", ".#." },
  ["8"] = { "###", "#.#", "###", "#.#", "###" }, ["9"] = { "###", "#.#", "###", "..#", "###" },
}
--- Hull number decal in a 3x5 font.
function F:digits(x, y, str, col)
  for i = 1, #str do
    local g = DIGITS[str:sub(i, i)]
    if g then self:map(x + (i - 1) * 4, y, g, { ["#"] = col }) end
  end
end

--- A glowing light: core pixel(s) plus a halo drawn after the outline, so it
--- blooms past the hull edge.
function F:glow(x, y, r, ramp, m)
  self.glows[#self.glows + 1] = { x = x, y = y, r = r, ramp = ramp }
  if m then self.glows[#self.glows + 1] = { x = x, y = self.h - 1 - y, r = r, ramp = ramp } end
end

--- Engine nozzle (shaded bell) at rear x, rows y..y+h-1, flame length len.
function F:engine(x, y, h, len, ramp, m)
  local function one(yy)
    self:part(ramp, { bevel = 1, shadow = 0 }, function(mk) mk:rect(x - 1, yy - 0.5, 3, h + 1) end)
    for i = 0, h - 1 do self:px(x - 1, yy + i, P.outline) end
    self.engines[#self.engines + 1] = { x = x - 1, y = yy, h = h, len = len }
  end
  one(y)
  if m then one(self.h - y - h) end
end

--- Weapon mount, drawn as its own lit part with barrels toward the nose.
--- kind: "auto" (square, twin barrels), "laser" (dome, long emitter with a hot
--- tip), "blaster" (dome, twin stubby barrels), "missile" (box of tubes).
--- size scales the mount (1 = small, 2 = medium, 3 = heavy).
function F:turret(kind, x, y, mat, size, m, tip)
  size = size or 1
  local function one(yy, flip)
    local r = 1 + size * 0.75
    if kind == "auto" then
      local L = 3 + size * 2
      local a, b = yy - 1, yy + 1
      if flip then a, b = b, a end
      self:hline(x + r, x + r + L, math.floor(yy - 1), mat[#mat])
      self:hline(x + r, x + r + L - 1, math.floor(yy + 1), mat[#mat - 1])
      self:part(mat, { bevel = 1 }, function(mk) mk:rect(x - r, yy - r, r * 2 + 0.5, r * 2 + 0.5) end)
      self:px(math.floor(x), math.floor(yy) - 1, mat[#mat])
    elseif kind == "laser" then
      local L = 4 + size * 3
      self:hline(x + 1, x + r + L, math.floor(yy), mat[#mat])
      self:part(mat, { round = true }, function(mk) mk:ellipse(x, yy, r + 0.3, r) end)
      if tip then self:glow(x + r + L, math.floor(yy), 1, tip) end
    elseif kind == "blaster" then
      local L = 2 + size
      for _, dy in ipairs({ -1, 1 }) do
        self:part(mat, { bevel = 1, shadow = 0 }, function(mk) mk:rect(x, yy + dy - 0.5 - (size > 1 and 0.5 or 0), r + L, size > 1 and 2 or 1) end)
      end
      self:part(mat, { round = true }, function(mk) mk:ellipse(x, yy, r + 0.5, r + 0.5) end)
      if tip then self:glow(x + r + L, math.floor(yy) - 1, 0.5, tip); self:glow(x + r + L, math.floor(yy) + 1, 0.5, tip) end
    elseif kind == "missile" then
      local n = 1 + size
      self:part(mat, { bevel = 1 }, function(mk) mk:rect(x - n, yy - n, n * 2 + 1, n * 2) end)
      for i = 0, n - 1 do
        for j = 0, n - 1 do
          self:px(math.floor(x - n + 1 + i * 2), math.floor(yy - n + 1 + j * 2), P.outline)
          if tip then self:px(math.floor(x - n + 1 + i * 2), math.floor(yy - n + 1 + j * 2), tip[2]) end
        end
      end
    end
  end
  one(y, false)
  if m then one(self.h - y, true) end
end

--- Panel seams: a dark line with a lit pixel row beside it, only where the
--- hull already has paint. Vertical seams at x positions in xs between y0..y1.
function F:seams(xs, y0, y1, dark, lite)
  for _, x in ipairs(xs) do
    for y = y0, y1 do
      if px.opaque(self.img, x + self.ox, y + self.oy) and px.opaque(self.img, x + self.ox, y + self.oy - 1)
        and px.opaque(self.img, x + self.ox, y + self.oy + 1) then
        self:px(x, y, dark)
        if lite and px.opaque(self.img, x + 1 + self.ox, y + self.oy) then self:px(x + 1, y, lite) end
      end
    end
  end
end

function F:after(fn) self.post[#self.post + 1] = fn end

-- ---------------------------------------------------------------------------
-- Finish
-- ---------------------------------------------------------------------------

local function drawGlow(s, img, g)
  local R = g.ramp
  local cx, cy = g.x + s.ox + 0.5, g.y + s.oy + 0.5
  local r = g.r
  for y = math.floor(cy - r - 1), math.ceil(cy + r + 1) do
    for x = math.floor(cx - r - 1), math.ceil(cx + r + 1) do
      local d = math.sqrt((x + 0.5 - cx) ^ 2 + (y + 0.5 - cy) ^ 2) / math.max(0.5, r)
      if d <= 1 then
        local t = 1 - d
        local i = math.max(1, math.min(#R, math.floor(t * #R + 0.5)))
        -- the outer halo only lands on transparent pixels or darker hull
        if t > 0.2 or not px.opaque(img, x, y) then
          if t > 0.2 or (x + y) % 2 == 0 then px.set(img, x, y, R[i]) end
        end
      end
    end
  end
end

local function drawFlames(s, img, frame)
  local g = s.glowRamp
  for _, e in ipairs(s.engines) do
    for yy = e.y, e.y + e.h - 1 do
      local mid = (e.h - 1) / 2
      local taper = math.floor(math.abs((yy - e.y) - mid) * 1.3 + 0.5)
      local len = e.len - taper + (frame == 2 and 2 or 0) + ((yy + frame) % 2)
      for i = 0, len - 1 do
        local t = i / math.max(1, len)
        local col
        if i == 0 then col = g[#g]
        elseif t < 0.3 then col = g[#g - 1]
        elseif t < 0.6 then col = g[#g - 2]
        else col = g[math.max(1, #g - 3)] end
        local xx = e.x + s.ox - 1 - i
        if not px.opaque(img, xx, yy + s.oy) then px.set(img, xx, yy + s.oy, col) end
      end
    end
  end
end

--- Crop to the bounding box of opaque pixels.
local function trim(img)
  local x0, y0, x1, y1 = img.width, img.height, -1, -1
  for y = 0, img.height - 1 do
    for x = 0, img.width - 1 do
      if px.opaque(img, x, y) then
        x0, y0 = math.min(x0, x), math.min(y0, y)
        x1, y1 = math.max(x1, x), math.max(y1, y)
      end
    end
  end
  local out = px.image(x1 - x0 + 1, y1 - y0 + 1)
  out:drawImage(img, Point(-x0, -y0))
  return out
end

--- Returns two frames (engine flicker; glows pulse on the second), plus
--- `frames.icon`: the hull without flames, cropped tight, for the UI.
function F:finish()
  px.outline(self.img, P.outline)
  for _, fn in ipairs(self.post) do fn(self) end
  local frames = {}
  for f = 1, 2 do
    local img = px.copy(self.img)
    for _, g in ipairs(self.glows) do
      drawGlow(self, img, { x = g.x, y = g.y, r = g.r + (f == 2 and 0.5 or 0), ramp = g.ramp })
    end
    if f == 1 then frames.icon = trim(img) end
    drawFlames(self, img, f)
    frames[f] = img
  end
  return frames
end

return F
