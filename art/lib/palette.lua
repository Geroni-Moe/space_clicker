-- Void Armada palette. Ramps run dark -> light.
local P = {}

P.outline = "#0a0b14"
P.space = { "#07080f", "#0d1020", "#151a33", "#1f2648", "#2c3463" }

-- Player fleet: cool steel hull, cyan systems.
P.steel = { "#232b42", "#3a4766", "#5a6d94", "#8ea4c8", "#cfdcef" }
P.cyan = { "#0e4a5c", "#1a8aa3", "#40d0e6", "#b5f6ff" }
P.engine = { "#1d4fd8", "#4aa3ff", "#a8e1ff", "#ffffff" }
P.gold = { "#7a4a12", "#c98a1c", "#f2c94c", "#fff2b0" }

-- Faction and race keys, in the order the game lists them. Their ship
-- materials live in art/lib/materials.lua and art/ships/.
P.factionOrder = { "crimson", "verdant", "void", "synod" }

P.raceOrder = { "throne", "tidewall", "concord", "rustborn" }

P.fire = { "#2a0d0a", "#6b1d0f", "#c2401a", "#f0802a", "#ffc75a", "#fff6d6" }
P.smoke = { "#1a1a22", "#2e2e3a", "#4a4a58", "#6e6e7e" }
P.purple = { "#2a1046", "#5a24a0", "#9d5cf0", "#dcbcff" }
P.green = { "#0f3d1e", "#1f8a3c", "#5ee67a", "#d0ffd8" }
P.red = { "#4a0b14", "#a3172b", "#ff4a5a", "#ffc2c2" }
P.ui = { "#0b0e1a", "#141a2e", "#1f2945", "#33416b", "#4d6299", "#7c93c7" }

return P
