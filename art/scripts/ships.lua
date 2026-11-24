-- Every ship in the game: 9 escort classes, 3 logistics hulls and 6 flagship tiers per player
-- faction (art/ships/<race>.lua) and 5 tiers per enemy faction
-- (art/ships/enemies.lua), plus a flame-free cropped "portrait" of every
-- player hull for the UI. Writes a contact sheet per faction to art/previews/.
--   aseprite --batch --script art/scripts/ships.lua
-- Set SHIPS_ONLY=<race|enemies> to rebuild a single set while iterating.

local px = dofile("art/lib/px.lua")
local P = dofile("art/lib/palette.lua")
local M = dofile("art/lib/materials.lua")

local only = os.getenv("SHIPS_ONLY")

local function sheet(rows, path)
  -- rows: list of lists of images; each row laid out left to right at 3x
  local K, pad = 3, 12
  local W, H = 0, pad
  for _, row in ipairs(rows) do
    local w, h = pad, 0
    for _, img in ipairs(row) do w = w + img.width * K + pad; h = math.max(h, img.height * K) end
    W = math.max(W, w)
    H = H + h + pad
  end
  local out = px.image(W, H)
  px.rect(out, 0, 0, W, H, P.space[2])
  local y = pad
  for _, row in ipairs(rows) do
    local x, h = pad, 0
    for _, img in ipairs(row) do h = math.max(h, img.height * K) end
    for _, img in ipairs(row) do
      px.stamp(out, px.scale(img, K), x, y + math.floor((h - img.height * K) / 2))
      x = x + img.width * K + pad
    end
    y = y + h + pad
  end
  local spr = Sprite(W, H, ColorMode.RGB)
  spr.cels[1].image = out
  spr:saveCopyAs(path)
  spr:close()
end

for _, race in ipairs(P.raceOrder) do
  if not only or only == race then
    local T = dofile("art/ships/" .. race .. ".lua")
    local fleetRow, bigRow, flagRow = {}, {}, {}
    for i, class in ipairs(M.ORDER) do
      local frames = T[class](false)
      px.export(frames, "fleet_" .. race .. "_" .. class, { duration = 0.12 })
      px.export({ frames.icon }, "portrait_fleet_" .. race .. "_" .. class)
      table.insert(i <= 5 and fleetRow or bigRow, frames[2])
    end
    local logiRow = {}
    for _, class in ipairs(M.LOGI) do
      local frames = T[class](false)
      px.export(frames, "fleet_" .. race .. "_" .. class, { duration = 0.12 })
      px.export({ frames.icon }, "portrait_fleet_" .. race .. "_" .. class)
      logiRow[#logiRow + 1] = frames[2]
    end
    for t, class in ipairs(M.FLAGSHIP) do
      local frames = T[class](true)
      px.export(frames, "flagship_" .. race .. "_" .. t, { duration = 0.12 })
      px.export({ frames.icon }, "portrait_flagship_" .. race .. "_" .. t)
      flagRow[#flagRow + 1] = frames[2]
    end
    sheet({ fleetRow, bigRow, logiRow, flagRow }, "art/previews/ships_" .. race .. ".png")
  end
end

if not only or only == "enemies" then
  local E = dofile("art/ships/enemies.lua")
  local rows = {}
  for _, f in ipairs(P.factionOrder) do
    local row = {}
    for t = 1, 5 do
      local frames = E[f](t)
      px.export(frames, "enemy_" .. f .. "_" .. t, { duration = 0.12 })
      row[#row + 1] = frames[2]
    end
    local frames = E.logi[f]()
    px.export(frames, "enemy_" .. f .. "_logi", { duration = 0.12 })
    row[#row + 1] = frames[2]
    rows[#rows + 1] = row
  end
  sheet(rows, "art/previews/ships_enemies.png")
end
