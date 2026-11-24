-- UI chrome (9-slice panels/buttons) and icons.
--   aseprite --batch --script art/scripts/ui.lua

local px = dofile("art/lib/px.lua")
local P = dofile("art/lib/palette.lua")
local U = P.ui

local preview = {}
local function out(img, name)
  px.export(img, "ui/" .. name)
  preview[#preview + 1] = img
end
-- 9-slice chrome is exported at 2x so its borders match the 2x world pixels.
local function chrome(img, name)
  px.export(px.scale(img, 2), "ui/" .. name)
  preview[#preview + 1] = img
end

-- ---------------------------------------------------------------------------
-- 9-slice frames. Corner/border size noted for the game's TextureSlicer.
-- ---------------------------------------------------------------------------

--- Bevelled metal frame, `n` px border. fill nil = hollow centre.
local function frame(w, h, cols)
  local img = px.image(w, h)
  px.rect(img, 1, 1, w - 2, h - 2, cols.fill)
  -- outer dark edge
  px.rect(img, 1, 0, w - 2, 1, P.outline); px.rect(img, 1, h - 1, w - 2, 1, P.outline)
  px.rect(img, 0, 1, 1, h - 2, P.outline); px.rect(img, w - 1, 1, 1, h - 2, P.outline)
  -- bevel: light top/left, dark bottom/right
  px.rect(img, 1, 1, w - 2, 1, cols.hi); px.rect(img, 1, 1, 1, h - 2, cols.hi)
  px.rect(img, 1, h - 2, w - 2, 1, cols.lo); px.rect(img, w - 2, 1, 1, h - 2, cols.lo)
  px.set(img, 1, h - 2, cols.mid); px.set(img, w - 2, 1, cols.mid)
  -- inner trim line
  if cols.trim then
    px.rect(img, 3, 3, w - 6, 1, cols.trim); px.rect(img, 3, h - 4, w - 6, 1, cols.trim)
    px.rect(img, 3, 3, 1, h - 6, cols.trim); px.rect(img, w - 4, 3, 1, h - 6, cols.trim)
  end
  return img
end

-- Main panel: 32x32, border 8.
do
  local img = frame(32, 32, { fill = U[2], hi = U[5], mid = U[4], lo = U[1], trim = U[3] })
  -- corner rivets
  for _, c in ipairs({ { 5, 5 }, { 26, 5 }, { 5, 26 }, { 26, 26 } }) do
    px.set(img, c[1], c[2], U[6]); px.set(img, c[1] + 1, c[2] + 1, U[1])
  end
  -- accent notches on the top edge
  px.rect(img, 12, 1, 8, 1, P.cyan[2])
  chrome(img, "panel")
end

-- Header strip: 32x16, border 6.
do
  local img = frame(32, 16, { fill = U[3], hi = U[6], mid = U[5], lo = U[1] })
  px.rect(img, 3, 13, 26, 1, P.cyan[2])
  chrome(img, "header")
end

-- Buttons: 24x24, border 6.
local function button(fill, hi, lo, glow)
  local img = frame(24, 24, { fill = fill, hi = hi, mid = fill, lo = lo })
  if glow then
    px.rect(img, 2, 2, 20, 1, glow)
    px.rect(img, 2, 21, 20, 1, glow)
    px.rect(img, 2, 2, 1, 20, glow)
    px.rect(img, 21, 2, 1, 20, glow)
  end
  return img
end
chrome(button(U[3], U[5], U[1]), "button")
chrome(button(U[4], U[6], U[2], P.cyan[2]), "button_hover")
chrome(button(U[2], U[1], U[4]), "button_pressed")
chrome(button("#151924", "#252b3a", "#0b0d14"), "button_disabled")
chrome(button("#14363f", P.cyan[2], "#08181d", P.cyan[3]), "button_ready")
chrome(button("#3a2a0c", P.gold[2], "#1a1206", P.gold[3]), "button_gold")

-- HP bar frame: 16x8, border 3.
do
  local img = px.image(16, 8)
  px.rect(img, 0, 0, 16, 8, P.outline)
  px.rect(img, 1, 1, 14, 6, "#05060b")
  px.rect(img, 1, 1, 14, 1, U[4])
  chrome(img, "bar")
end

-- ---------------------------------------------------------------------------
-- 16x16 icons
-- ---------------------------------------------------------------------------
local function icon(rows, key)
  local img = px.image(16, 16)
  px.map(img, 0, 0, rows, key)
  return img
end
local function outlined(img) px.outline(img, P.outline); return img end

local K = {
  w = "#ffffff", g = P.gold[3], G = P.gold[2], d = P.gold[1], y = P.gold[4],
  c = P.cyan[3], C = P.cyan[2], b = P.cyan[1], l = P.cyan[4],
  s = P.steel[3], S = P.steel[2], h = P.steel[4], k = P.steel[1],
  r = P.red[3], R = P.red[2], o = P.fire[4], O = P.fire[3], p = P.purple[3], P = P.purple[2], q = P.purple[4],
  e = P.green[3], E = P.green[2],
}

out(outlined(icon({
  "................",
  "................",
  ".....GGGGGG.....",
  "....GggggggG....",
  "...GggyyyyggG...",
  "...GgyGGGGygG...",
  "...GgyG..GggG...",
  "...GgyG.....G...",
  "...GgyG..GggG...",
  "...GgyGGGGygG...",
  "...GggyyyyggG...",
  "....GggggggG....",
  ".....GGGGGG.....",
  "................",
  "................",
  "................",
}, K)), "icon_credits")

out(outlined(icon({
  "................",
  ".......c........",
  ".......c........",
  ".....CCcCC......",
  "....C..c..C.....",
  "...C...c...C....",
  "...C.......C....",
  ".cccc..w..cccc..",
  "...C.......C....",
  "...C...c...C....",
  "....C..c..C.....",
  ".....CCcCC......",
  ".......c........",
  ".......c........",
  "................",
  "................",
}, K)), "icon_dps")

out(outlined(icon({
  "................",
  "....w...........",
  "....ww..........",
  "....wlw.........",
  "....wllw........",
  "....wlllw.......",
  "....wllllw......",
  "....wlllllw.....",
  "....wllllwww....",
  "....wlwlw.......",
  "....ww.wlw......",
  "....w...wlw.....",
  ".........ww.....",
  "................",
  "................",
  "................",
}, K)), "icon_click")

out(outlined(icon({
  "................",
  ".......y........",
  ".......y........",
  "......ygG.......",
  "......ygG.......",
  "..yyyyyggGGGGG..",
  "...GgggwwggGG...",
  "....GgggggGG....",
  ".....GgggGG.....",
  ".....GggGgG.....",
  "....GgG..GgG....",
  "....GG....GG....",
  "...G........G...",
  "................",
  "................",
  "................",
}, K)), "icon_crit")

out(outlined(icon({
  "................",
  "................",
  "..ss........ss..",
  "..shs......shs..",
  "...shsSSSSshs...",
  "....sSccccSs....",
  "....SccllccS....",
  "....SclwwlcS....",
  "....SccllccS....",
  "....sSccccSs....",
  "...shsSSSSshs...",
  "..shs......shs..",
  "..ss........ss..",
  "................",
  "................",
  "................",
}, K)), "icon_drone")

out(outlined(icon({
  "................",
  ".......hh.......",
  "......hssh......",
  ".....hssssh.....",
  "....hsssssSh....",
  "...hssccsSSSh...",
  "...hscllcSSSh...",
  "...hscllcSSSk...",
  "...hssccsSSSk...",
  "....hsssSSSk....",
  ".....hsSSSk.....",
  "......hSSk......",
  ".......kk.......",
  "................",
  "................",
  "................",
}, K)), "icon_hull")

out(outlined(icon({
  "................",
  ".......q........",
  "......qpq.......",
  ".....qppPq......",
  "....qpppPPq.....",
  "...qwppPPPPq....",
  "...qppPPPPPq....",
  "...qppPPPPPq....",
  "...qpPPPPPPq....",
  "....qpPPPPq.....",
  ".....qPPPq......",
  "......qPq.......",
  ".......q........",
  "................",
  "................",
  "................",
}, K)), "icon_shard")

out(outlined(icon({
  "................",
  "................",
  ".....RRRRRR.....",
  "....RrrrrrrR....",
  "...RrrrrrrrrR...",
  "...RrwwrrwwrR...",
  "...RrwkrrkwrR...",
  "...RrrrrrrrrR...",
  "....RrrRRrrR....",
  ".....RrrrrR.....",
  ".....RrRrRrR....",
  ".....R.R.R.R....",
  "................",
  "................",
  "................",
  "................",
}, K)), "icon_boss")

out(outlined(icon({
  "................",
  ".....ssssss.....",
  "......shhs......",
  "....SSSSSSSS....",
  "...SccccccccS...",
  "..Scccc.cccccS..",
  "..Sccccw.ccccS..",
  "..Sccccwwww.cS..",
  "..Scccccccc.cS..",
  "..ScccccccccS...",
  "...SccccccccS...",
  "....SSSSSSSS....",
  "................",
  "................",
  "................",
  "................",
}, K)), "icon_timer")

out(outlined(icon({
  "................",
  "................",
  "...cc......cc...",
  "..cllc....cllc..",
  "..cllc....cllc..",
  "...cc......cc...",
  "....C......C....",
  ".....C....C.....",
  "......CllC......",
  "......lwwl......",
  "......CllC......",
  ".....C....C.....",
  "................",
  "................",
  "................",
  "................",
}, K)), "icon_sync")

-- Abilities
out(outlined(icon({
  "................",
  ".........yy.....",
  "........ygG.....",
  ".......ygG......",
  "......ygG.......",
  ".....ygG........",
  "....yggggggy....",
  ".......ygG......",
  "......ygG.......",
  ".....ygG........",
  "....ygG.........",
  "...yG...........",
  "..y.............",
  "................",
  "................",
  "................",
}, K)), "icon_overcharge")

out(outlined(icon({
  "................",
  "......o..o......",
  ".......oo.......",
  "...o..oOOo..o...",
  "....ooOyyOoo....",
  ".....OywwyO.....",
  "..ooOywwwwyOoo..",
  ".....OywwyO.....",
  "....ooOyyOoo....",
  "...o..oOOo..o...",
  ".......oo.......",
  "......o..o......",
  "................",
  "................",
  "................",
  "................",
}, K)), "icon_broadside")

out(outlined(icon({
  "................",
  "................",
  "...RRR....RRR...",
  "...Rrr....rrR...",
  "...Rrr....rrR...",
  "...hhh....hhh...",
  "...Rrr....rrR...",
  "...Rrr....rrR...",
  "...Rrrr..rrrR...",
  "....Rrrrrrrr....",
  ".....RRrrRR.....",
  ".......RR.......",
  "................",
  "................",
  "................",
  "................",
}, K)), "icon_salvage")

out(outlined(icon({
  "................",
  "................",
  "................",
  ".........c......",
  "....cc...cc.....",
  "...cllcc.clc....",
  "..cllllcccllc...",
  "..cllllllllllc..",
  "..cllllcccllc...",
  "...cllcc.clc....",
  "....cc...cc.....",
  ".........c......",
  "................",
  "................",
  "................",
  "................",
}, K)), "icon_warp")

out(outlined(icon({
  "................",
  "................",
  ".......s........",
  "......ss........",
  "....sshs....c...",
  "...shhhs..c..c..",
  "...shhhs...c.c..",
  "...shhhs..c..c..",
  "....sshs....c...",
  "......ss........",
  ".......s........",
  "................",
  "................",
  "................",
  "................",
  "................",
}, K)), "icon_sound_on")

out(outlined(icon({
  "................",
  "................",
  ".......s........",
  "......ss........",
  "....sshs........",
  "...shhhs..R...R.",
  "...shhhs...R.R..",
  "...shhhs....R...",
  "...shhhs...R.R..",
  "....sshs..R...R.",
  "......ss........",
  ".......s........",
  "................",
  "................",
  "................",
  "................",
}, K)), "icon_sound_off")

out(outlined(icon({
  "................",
  "................",
  "......eeee......",
  ".....eEEEEe.....",
  "....eEewweEe....",
  "....eEwEEwEe....",
  "....eEEEEwEe....",
  "....eEEEwEEe....",
  "....eEEwEEEe....",
  "....eEEEEEEe....",
  "....eEEwEEEe....",
  ".....eEEEEe.....",
  "......eeee......",
  "................",
  "................",
  "................",
}, K)), "icon_research")

-- armour plate shield with rivets
out(outlined(icon({
  "................",
  "..hhhhhhhhhhhh..",
  "..hsssssssssSh..",
  "..hsw.sssss.Sh..",
  "..hssssssssssh..",
  "..hsssSSSSssSh..",
  "..hsssSkkSssSh..",
  "..hsssSkkSssSh..",
  "..hssssSSsssSh..",
  "...hssssssssh...",
  "...hsw.ss.sSh...",
  "....hssssssh....",
  ".....hsssSh.....",
  "......hSSh......",
  ".......hh.......",
  "................",
}, K)), "icon_plating")

-- wrench over a green repair cross
out(outlined(icon({
  "................",
  "..........hh....",
  ".........hssh...",
  "....ee...hs.....",
  "....eE..hs...h..",
  "..eeeEEhs..hhh..",
  "..eEEEhs..hss...",
  "....eEs..hss....",
  "....eh..hss.....",
  ".......hss......",
  "......hss.......",
  ".....hss........",
  "....hss.........",
  "...hss..........",
  "...hs...........",
  "................",
}, K)), "icon_repair")

-- retreat: chevrons racing back toward a warp ring
out(outlined(icon({
  "................",
  "................",
  "...l........c...",
  "..lc.......cl...",
  ".lC...c...c.....",
  "lC...cl..cl..b..",
  "C...cC..cC...bb.",
  "l..lC..lC....bbb",
  "C...cC..cC...bb.",
  "lC...cl..cl..b..",
  ".lC...c...c.....",
  "..lc.......cl...",
  "...l........c...",
  "................",
  "................",
  "................",
}, K)), "icon_retreat")

-- menu: three bars
out(outlined(icon({
  "................",
  "................",
  "................",
  "...hhhhhhhhhh...",
  "...hsssssssss...",
  "................",
  "................",
  "...hhhhhhhhhh...",
  "...hsssssssss...",
  "................",
  "................",
  "...hhhhhhhhhh...",
  "...hsssssssss...",
  "................",
  "................",
  "................",
}, K)), "icon_menu")

-- Contact sheet
local k, W = 4, 900
local x, y, rowH, placed = 4, 4, 0, {}
for _, img in ipairs(preview) do
  local w, h = img.width * k, img.height * k
  if x + w > W then x = 4; y = y + rowH + 8; rowH = 0 end
  placed[#placed + 1] = { img = img, x = x, y = y }
  x = x + w + 8; rowH = math.max(rowH, h)
end
local H = y + rowH + 4
local sheet = px.image(W, H)
px.rect(sheet, 0, 0, W, H, "#2a2f3f")
for _, p in ipairs(placed) do px.stamp(sheet, px.scale(p.img, k), p.x, p.y) end
local spr = Sprite(W, H, ColorMode.RGB)
spr.cels[1].image = sheet
spr:saveCopyAs("art/previews/ui.png")
spr:close()
