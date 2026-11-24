-- px.lua - pixel-art helpers for Void Armada's Aseprite batch scripts.
--
-- Run every script from the repo root so relative paths resolve:
--   /Applications/Aseprite.app/Contents/MacOS/aseprite --batch --script art/scripts/<name>.lua
--
-- All drawing happens on Aseprite `Image` objects (RGB). Colours are hex
-- strings ("#6b7fa8"), raw pixel values, or nil/"clear" for transparent.
-- Nothing anti-aliases: every write is an exact palette colour.

local px = {}
local pc = app.pixelColor

px.CLEAR = pc.rgba(0, 0, 0, 0)
local cache = {}

function px.c(col)
  if col == nil or col == "clear" then return px.CLEAR end
  if type(col) == "number" then return col end
  local v = cache[col]
  if v then return v end
  local hex = col:gsub("^#", "")
  local r = tonumber(hex:sub(1, 2), 16)
  local g = tonumber(hex:sub(3, 4), 16)
  local b = tonumber(hex:sub(5, 6), 16)
  local a = #hex == 8 and tonumber(hex:sub(7, 8), 16) or 255
  v = pc.rgba(r, g, b, a)
  cache[col] = v
  return v
end

function px.image(w, h)
  local img = Image(w, h, ColorMode.RGB)
  img:clear(px.CLEAR)
  return img
end

local function inb(img, x, y) return x >= 0 and y >= 0 and x < img.width and y < img.height end

function px.set(img, x, y, col)
  x, y = math.floor(x), math.floor(y)
  if inb(img, x, y) then img:drawPixel(x, y, px.c(col)) end
end

function px.get(img, x, y)
  if not inb(img, x, y) then return px.CLEAR end
  return img:getPixel(x, y)
end

function px.opaque(img, x, y) return pc.rgbaA(px.get(img, x, y)) > 0 end

function px.rect(img, x, y, w, h, col)
  for yy = y, y + h - 1 do
    for xx = x, x + w - 1 do px.set(img, xx, yy, col) end
  end
end

function px.line(img, x0, y0, x1, y1, col)
  x0, y0, x1, y1 = math.floor(x0), math.floor(y0), math.floor(x1), math.floor(y1)
  local dx, dy = math.abs(x1 - x0), -math.abs(y1 - y0)
  local sx, sy = x0 < x1 and 1 or -1, y0 < y1 and 1 or -1
  local err = dx + dy
  while true do
    px.set(img, x0, y0, col)
    if x0 == x1 and y0 == y1 then break end
    local e2 = 2 * err
    if e2 >= dy then err = err + dy; x0 = x0 + sx end
    if e2 <= dx then err = err + dx; y0 = y0 + sy end
  end
end

--- Filled ellipse centred on (cx, cy); centre may be fractional (x.5 = between pixels).
function px.ellipse(img, cx, cy, rx, ry, col)
  for y = math.floor(cy - ry - 1), math.ceil(cy + ry + 1) do
    for x = math.floor(cx - rx - 1), math.ceil(cx + rx + 1) do
      local nx, ny = (x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry
      if nx * nx + ny * ny <= 1.0 then px.set(img, x, y, col) end
    end
  end
end

function px.disc(img, cx, cy, r, col) px.ellipse(img, cx, cy, r, r, col) end

function px.ring(img, cx, cy, r, thick, col)
  for y = math.floor(cy - r - 1), math.ceil(cy + r + 1) do
    for x = math.floor(cx - r - 1), math.ceil(cx + r + 1) do
      local d = math.sqrt((x + 0.5 - cx) ^ 2 + (y + 0.5 - cy) ^ 2)
      if d <= r and d > r - thick then px.set(img, x, y, col) end
    end
  end
end

--- Scanline polygon fill, sampling pixel centres. pts = {{x,y},...}
function px.poly(img, pts, col)
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
      for x = math.ceil(xs[i] - 0.5), math.floor(xs[i + 1] - 0.5) do px.set(img, x, y, col) end
    end
  end
end

--- Draw an ASCII map at (x, y). key maps chars to colours; '.' and ' ' are skipped.
function px.map(img, x, y, rows, key)
  for r, row in ipairs(rows) do
    for i = 1, #row do
      local ch = row:sub(i, i)
      if ch ~= "." and ch ~= " " then
        local col = key[ch]
        assert(col, "px.map: no colour for '" .. ch .. "'")
        px.set(img, x + i - 1, y + r - 1, col)
      end
    end
  end
end

function px.copy(img)
  local out = px.image(img.width, img.height)
  out:drawImage(img, Point(0, 0))
  return out
end

--- Copy src onto dst at (x, y), skipping transparent source pixels.
function px.stamp(dst, src, x, y)
  for yy = 0, src.height - 1 do
    for xx = 0, src.width - 1 do
      local p = src:getPixel(xx, yy)
      if pc.rgbaA(p) > 0 then px.set(dst, x + xx, y + yy, p) end
    end
  end
end

function px.pad(img, n)
  local out = px.image(img.width + 2 * n, img.height + 2 * n)
  px.stamp(out, img, n, n)
  return out
end

--- 1px outline around every opaque pixel (4-neighbour, optional diagonals).
function px.outline(img, col, diag)
  col = px.c(col or "#0a0b14")
  local marks = {}
  for y = 0, img.height - 1 do
    for x = 0, img.width - 1 do
      if not px.opaque(img, x, y) then
        local hit = px.opaque(img, x - 1, y) or px.opaque(img, x + 1, y)
          or px.opaque(img, x, y - 1) or px.opaque(img, x, y + 1)
        if diag and not hit then
          hit = px.opaque(img, x - 1, y - 1) or px.opaque(img, x + 1, y - 1)
            or px.opaque(img, x - 1, y + 1) or px.opaque(img, x + 1, y + 1)
        end
        if hit then marks[#marks + 1] = { x, y } end
      end
    end
  end
  for _, m in ipairs(marks) do img:drawPixel(m[1], m[2], col) end
end

--- Replace colours by table {fromHex = toHex}.
function px.recolor(img, swaps)
  local lookup = {}
  for from, to in pairs(swaps) do lookup[px.c(from)] = px.c(to) end
  for y = 0, img.height - 1 do
    for x = 0, img.width - 1 do
      local v = lookup[img:getPixel(x, y)]
      if v then img:drawPixel(x, y, v) end
    end
  end
end

-- ---------------------------------------------------------------------------
-- Noise and dithering
-- ---------------------------------------------------------------------------

function px.rng(seed)
  local s = seed % 2147483647
  if s <= 0 then s = s + 2147483646 end
  return function(a, b)
    s = (s * 16807) % 2147483647
    local f = (s - 1) / 2147483646
    if a == nil then return f end
    return a + math.floor(f * (b - a + 1))
  end
end

local function hash(ix, iy, seed)
  local n = ix * 374761393 + iy * 668265263 + seed * 1442695041
  n = (n ~ (n >> 13)) * 1274126177
  n = n ~ (n >> 16)
  return (n & 0xffff) / 0xffff
end

--- Tileable value noise with period (pw, ph) in cells.
function px.vnoise(x, y, seed, pw, ph)
  local x0, y0 = math.floor(x), math.floor(y)
  local fx, fy = x - x0, y - y0
  local function h(ix, iy)
    if pw then ix = ix % pw end
    if ph then iy = iy % ph end
    return hash(ix, iy, seed)
  end
  local sx, sy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
  local a = h(x0, y0) + (h(x0 + 1, y0) - h(x0, y0)) * sx
  local b = h(x0, y0 + 1) + (h(x0 + 1, y0 + 1) - h(x0, y0 + 1)) * sx
  return a + (b - a) * sy
end

--- Fractal noise; when tileW/tileH given the result tiles over that many pixels.
function px.fbm(x, y, seed, scale, octaves, tileW, tileH)
  local sum, amp, norm, freq = 0, 1, 0, 1
  for o = 1, octaves do
    local cells = scale * freq
    local pw = tileW and math.floor(tileW / cells + 0.5) or nil
    local ph = tileH and math.floor(tileH / cells + 0.5) or nil
    sum = sum + amp * px.vnoise(x / cells, y / cells, seed + o * 31, pw, ph)
    norm = norm + amp
    amp = amp * 0.5
    freq = freq * 0.5
  end
  return sum / norm
end

local BAYER = { 0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5 }
function px.bayer(x, y) return (BAYER[(y % 4) * 4 + (x % 4) + 1] + 0.5) / 16 end

--- Pick from a colour ramp by t in [0,1) with ordered dithering between steps.
function px.ramp(ramp, t, x, y)
  t = math.max(0, math.min(0.9999, t))
  local f = t * (#ramp - 1)
  local i = math.floor(f)
  local frac = f - i
  if frac > px.bayer(x, y) then i = i + 1 end
  return ramp[math.min(#ramp, i + 1)]
end

-- ---------------------------------------------------------------------------
-- Export
-- ---------------------------------------------------------------------------

--- Save frames (list of equal-size images) as a horizontal PNG sheet plus an
--- editable .aseprite source with one frame per image.
function px.export(frames, name, opts)
  opts = opts or {}
  if type(frames) == "userdata" then frames = { frames } end
  local w, h = frames[1].width, frames[1].height
  local png = opts.png or ("assets/sprites/" .. name .. ".png")
  local ase = opts.ase or ("art/src/" .. name .. ".aseprite")

  local sheet = Sprite(w * #frames, h, ColorMode.RGB)
  local sheetImg = px.image(w * #frames, h)
  for i, f in ipairs(frames) do sheetImg:drawImage(f, Point((i - 1) * w, 0)) end
  sheet.cels[1].image = sheetImg
  sheet:saveCopyAs(png)
  sheet:close()

  local src = Sprite(w, h, ColorMode.RGB)
  for i, f in ipairs(frames) do
    if i > 1 then src:newEmptyFrame() end
    src:newCel(src.layers[1], i, f, Point(0, 0))
    src.frames[i].duration = opts.duration or 0.08
  end
  src:saveAs(ase)
  src:close()
  print(string.format("[art] %-32s %dx%d x%d", name, w, h, #frames))
end

--- Nearest-neighbour upscale (for previews).
function px.scale(img, k)
  local out = px.image(img.width * k, img.height * k)
  for y = 0, img.height - 1 do
    for x = 0, img.width - 1 do
      local p = img:getPixel(x, y)
      if pc.rgbaA(p) > 0 then
        for yy = 0, k - 1 do
          for xx = 0, k - 1 do out:drawPixel(x * k + xx, y * k + yy, p) end
        end
      end
    end
  end
  return out
end

return px
