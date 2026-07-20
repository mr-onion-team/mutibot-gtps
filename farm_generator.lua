-- ============================================================
-- World Farm Generator
-- ============================================================
-- Generates horizontal platform farm layout:
--   Platform row -> Empty row -> Platform row -> ...
--
-- Usage:
--   1. Warp bot to target world
--   2. Set BLOCK_ID below to your desired block
--   3. Run this script
-- ============================================================

local bot = getBot()
local BLOCK_ID = 2          -- Platform block (2 = Dirt, change as needed)
local PLACE_DELAY = 80      -- ms between placements
local ROW_DELAY = 200       -- ms between rows

-- ── Check we're in a world ──────────────────────────────────
local world = getWorld()
if not world then
    console("`4Error: Bot is not in a world!")
    return
end

local W = world.x
local H = world.y
console("`2Farm gen started: " .. W .. "x" .. H .. " block=" .. BLOCK_ID)

-- ── Check inventory ─────────────────────────────────────────
local inv = bot:getInventory()
if not inv then
    console("`4Error: Could not read inventory!")
    return
end

local count = inv:getItemCount(BLOCK_ID)
console("`oInventory check: have " .. count .. "x block " .. BLOCK_ID)
if count == 0 then
    console("`4Error: No blocks (id=" .. BLOCK_ID .. ") in inventory! Buy/pickup some first.")
    return
end

local placed = 0
local skipped = 0
local errors = 0

-- ── Build farm ──────────────────────────────────────────────
-- y=0,2,4... = platform (solid)
-- y=1,3,5... = empty (walkway)
for y = 0, H - 1 do
    if y % 2 == 0 then
        -- Platform row: walk left to right, place blocks
        -- Teleport to leftmost tile of this row
        console("`oWalking to row " .. y .. "...")
        bot.moveTile(0, y)
        sleep(ROW_DELAY)

        -- Walk right across the row placing blocks
        for x = 0, W - 1 do
            local tile = getTile(x, y)
            if not tile or tile.foreground ~= BLOCK_ID then
                bot.place(0, 0, BLOCK_ID)
                placed = placed + 1
            else
                skipped = skipped + 1
            end

            -- Move right (except at last column)
            if x < W - 1 then
                bot.moveRight(1)
                sleep(PLACE_DELAY)
            end
        end

        -- Progress every 5 rows
        if math.floor(y / 2) % 5 == 0 then
            console("`2Row " .. y .. " done (" .. placed .. " placed, " .. skipped .. " skipped)")
        end
    end
end

-- ── Done ────────────────────────────────────────────────────
console("`2Farm done! " .. placed .. " blocks placed, " .. skipped .. " skipped.")
