-- Parallax backdrop: tileable star layers, nebula clouds and planets.
--   aseprite --batch --script art/scripts/backdrop.lua

local px = dofile("art/lib/px.lua")
local P = dofile("art/lib/palette.lua")

-- ---------------------------------------------------------------------------
-- Star layers (256x256, tile in both axes)
-- ---------------------------------------------------------------------------
local function stars(seed, count, cols, bigChance)
  local img = px.image(256, 256)
  local rnd = px.rng(seed)
  for _ = 1, count do
    local x, y = rnd(0, 255), rnd(0, 255)
    local col = cols[rnd(1, #cols)]
    px.set(img, x, y, col)
    if rnd() < bigChance then
      local dim = cols[1]
      px.set(img, (x + 1) % 256, y, dim); px.set(img, (x + 255) % 256, y, dim)
      px.set(img, x, (y + 1) % 256, dim); px.set(img, x, (y + 255) % 256, dim)
      px.set(img, x, y, "#ffffff")
    end
  end
  return img
end

px.export(stars(7, 420, { "#2a3152", "#343d66", "#4a5480" }, 0), "stars_far")
px.export(stars(19, 120, { "#5a6a9e", "#8090c0", "#b0bce0" }, 0.08), "stars_mid")
px.export(stars(31, 28, { "#9fb3d1", "#dbe7f5", "#fff2b0", "#b5f6ff" }, 0.5), "stars_near")

-- ---------------------------------------------------------------------------
-- Nebulae (384x256, tile horizontally) in each faction's hue.
-- ---------------------------------------------------------------------------
local nebulaRamps = {
  crimson = { "#1a0d18", "#2e1222", "#4a1a2c", "#6e2636", "#9a3a3e" },
  verdant = { "#0d1a18", "#11291f", "#1a3d26", "#2a5a2e", "#467a3a" },
  void = { "#130d24", "#1e1238", "#2e1a52", "#46267a", "#6a3aa6" },
  synod = { "#0f1216", "#161b22", "#222a35", "#323e4e", "#4a5a6e" },
}
for i, name in ipairs(P.factionOrder) do
  local ramp = nebulaRamps[name]
  local W, H = 384, 256
  local img = px.image(W, H)
  for y = 0, H - 1 do
    for x = 0, W - 1 do
      local n = px.fbm(x, y, 100 + i * 17, 96, 5, W, H)
      local warp = px.fbm(x + 50, y + 90, 300 + i, 48, 3, W, H)
      local v = n * 0.9 + warp * 0.5 - 0.66
      -- keep the band mostly mid-screen so the HUD stays readable
      local band = math.max(0, 1 - math.abs((y - H * 0.5) / (H * 0.5)) ^ 2)
      v = v * band * 2.4
      -- soft edge: dither out to transparent instead of a hard cutoff
      if v > 0 and (v > 0.18 or v / 0.18 > px.bayer(x, y)) then
        px.set(img, x, y, px.ramp(ramp, v, x, y))
      end
    end
  end
  px.export(img, "nebula_" .. name)
end

-- ---------------------------------------------------------------------------
-- Planets (lit from the upper left, dithered terminator)
-- ---------------------------------------------------------------------------
local function planet(size, seed, surface, opts)
  opts = opts or {}
  local img = px.image(size, size)
  local c, r = size / 2, size / 2 - 3
  local lx, ly, lz = -0.6, -0.5, 0.62
  for y = 0, size - 1 do
    for x = 0, size - 1 do
      local nx, ny = (x + 0.5 - c) / r, (y + 0.5 - c) / r
      local d2 = nx * nx + ny * ny
      if d2 <= 1 then
        local nz = math.sqrt(1 - d2)
        local light = math.max(0, nx * lx + ny * ly + nz * lz)
        local u = x + nx * 6
        local tex
        if opts.bands then
          tex = px.fbm(u * 0.3, y * 2.4, seed, 18, 3) * 0.6 + (math.sin(y * 0.35 + px.fbm(x, y, seed + 5, 22, 2) * 4) * 0.5 + 0.5) * 0.5
        else
          tex = px.fbm(u, y, seed, 14, 4)
        end
        local v = light * 0.85 + (tex - 0.5) * 0.55 + 0.05
        px.set(img, x, y, px.ramp(surface, v, x, y))
        -- atmosphere rim on the lit side
        if opts.atmo and d2 > 0.86 and light > 0.25 then px.set(img, x, y, opts.atmo) end
      end
    end
  end
  if opts.ring then
    local front = px.image(size, size)
    for y = 0, size - 1 do
      for x = 0, size - 1 do
        local rx, ry = (x + 0.5 - c) / (r * 1.7), (y + 0.5 - c) / (r * 0.5)
        local q = rx * rx + ry * ry
        if q <= 1 and q > 0.5 then
          local col = (q > 0.75) and opts.ring[2] or opts.ring[1]
          if q > 0.62 and q < 0.68 then col = nil end
          if col and (x + y) % 5 == 0 then col = opts.ring[3] end
          local behind = y < c and ((x + 0.5 - c) ^ 2 + (y + 0.5 - c) ^ 2) < r * r
          if col and not behind then px.set(front, x, y, col) end
        end
      end
    end
    px.stamp(img, front, 0, 0)
  end
  return img
end

px.export(planet(112, 5, { "#1b0f24", "#3a1f45", "#6b3a5c", "#a35d63", "#d99a7a", "#f2d3a0" },
  { bands = true, ring = { "#6b5a7a", "#a597b5", "#d6cce0" } }), "planet_ringed")
px.export(planet(80, 9, { "#0f1a2e", "#1a3352", "#2e5c7a", "#5a9cb0", "#a8dde0", "#effcff" },
  { atmo = "#b5f6ff" }), "planet_ice")
px.export(planet(72, 13, { "#1a0a08", "#3d1410", "#7a2414", "#c2401a", "#f0802a", "#ffc75a" },
  {}), "planet_lava")
px.export(planet(96, 21, { "#0d1a14", "#1a3324", "#2e5236", "#4f7a3e", "#8fae5a", "#d6e0a0" },
  { atmo = "#a9e8c0" }), "planet_jungle")

-- Preview: compose a sample backdrop.
local W, H = 640, 400
local sheet = px.image(W, H)
px.rect(sheet, 0, 0, W, H, P.space[1])
local function tile(src, ox)
  for y = 0, H - 1, src.height do
    for x = -ox, W - 1, src.width do px.stamp(sheet, src, x, y) end
  end
end
local function load(name) local s = Sprite { fromFile = "assets/sprites/" .. name .. ".png" }; local i = px.copy(s.cels[1].image); s:close(); return i end
tile(load("stars_far"), 0)
tile(load("nebula_void"), 40)
tile(load("stars_mid"), 10)
px.stamp(sheet, load("planet_ringed"), 430, 30)
tile(load("stars_near"), 70)
local spr = Sprite(W, H, ColorMode.RGB)
spr.cels[1].image = px.scale(sheet, 2)
spr:resize(W * 2, H * 2)
spr.cels[1].image = px.scale(sheet, 2)
spr:saveCopyAs("art/previews/backdrop.png")
spr:close()
