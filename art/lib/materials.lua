-- Material ramps for forge.lua ships (dark -> light, hue-shifted: cool
-- shadows, warm highlights), plus engine flame ramps per culture.

local M = {
  -- Rustborn
  rust = { "#1e0f0c", "#3d1d14", "#64301c", "#8f4524", "#b8642e", "#d98f4a", "#f2c27e" },
  scrap = { "#15171c", "#2a2d35", "#454a55", "#676e7a", "#959ca7", "#c8ccd2" },
  warred = { "#300806", "#6e130c", "#a8241a", "#d8432c", "#ff7f56", "#ffc0a0" },
  bone = { "#3e3528", "#7a6a4e", "#b7a57c", "#e2d4ac", "#fbf3dc" },
  ember = { "#5a1406", "#b8380c", "#ff7a1c", "#ffc35c", "#fff2c8" },
  -- Solar Throne
  gold = { "#2a1306", "#552a0d", "#8a4a12", "#c47c20", "#eaab3a", "#ffd86e", "#fff6cc" },
  marble = { "#3a2e38", "#625260", "#93838a", "#c2b5b2", "#e6ddd2", "#fbf7ee" },
  ruby = { "#4a0716", "#9c1030", "#e8304a", "#ff8a7a", "#ffe4d6" },
  sun = { "#7a3008", "#d06a10", "#ffb02e", "#ffe27a", "#ffffff" },
  -- Tidewall State
  navy = { "#0b1020", "#151d38", "#223056", "#334a7a", "#4f6aa0", "#7f9ac6" },
  white = { "#2f3648", "#5a6680", "#929eb6", "#c8d1e0", "#eef2f8", "#ffffff" },
  cyan = { "#05303c", "#0b6a80", "#1db2cc", "#6ae6f5", "#dcfcff" },
  orange = { "#5a2006", "#b04a0c", "#ff8a1c", "#ffc070" },
  warhead = { "#ff8a1c", "#ffe0b0" },
  -- Free Concord
  teal = { "#081c1e", "#103636", "#1a5a52", "#2a8472", "#46ae92", "#7fd8b2", "#ccf6df" },
  chrome = { "#232833", "#434c5c", "#6e7a8c", "#a2aebd", "#d6dee8", "#ffffff" },
  azure = { "#0a2552", "#1856c0", "#42a0ff", "#a2dcff", "#f0faff" },
  plasma = { "#0b3a1a", "#1a8a3c", "#5ee67a", "#c8ffd8", "#ffffff" },
  leaf = { "#0c2a14", "#17502a", "#2a7a36", "#4aa84a", "#8ad86a" },
  window = "#ffd98a",
  -- shared
  hazard = "#ff9a1c",
  commandGold = { "#552a0d", "#c47c20", "#ffd86e", "#fff6cc" },
  -- repair emitters (player), mint
  mend = { "#0c3a2a", "#1a8a5a", "#5ef0b0", "#c8ffe6", "#ffffff" },
}

M.flame = {
  rust = { "#c2401a", "#ff8a3c", "#ffd27a", "#ffffff" },
  throne = { "#c25a12", "#ff9a3c", "#ffd98a", "#ffffff" },
  tide = { "#1d4fd8", "#4aa3ff", "#a8e1ff", "#ffffff" },
  concord = { "#1a8a4a", "#5ee67a", "#c8ffd8", "#ffffff" },
}

-- class sizes (w, odd h) shared by every faction, so formations line up
M.CLASS = {
  interceptor = { 26, 15 }, frigate = { 36, 21 }, destroyer = { 46, 25 }, cruiser = { 58, 31 },
  battlecruiser = { 70, 35 }, battleship = { 86, 43 }, carrier = { 104, 51 }, dreadnought = { 112, 51 },
  titan = { 124, 55 },
  logi_frigate = { 36, 21 }, logi_cruiser = { 58, 31 }, fax = { 104, 51 },
}
M.ORDER = { "interceptor", "frigate", "destroyer", "cruiser", "battlecruiser", "battleship", "carrier", "dreadnought", "titan" }
-- logistics hulls, after the escorts in the game's hull index
M.LOGI = { "logi_frigate", "logi_cruiser", "fax" }
-- flagship hull tier -> the class design it is built on
M.FLAGSHIP = { "frigate", "destroyer", "cruiser", "battlecruiser", "battleship", "titan" }

return M
