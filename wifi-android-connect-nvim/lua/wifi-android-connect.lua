local ffi_lib = require("libwifi_android_connect_nvim")
local utils = require("utils")

local function setup(opts)
    local default_ops = {
        pair_code = nil,
        pair_name = "WIFI Android Connect nvim",
        timeout_in_minutes = 5
    }
    opts = opts or default_ops
    ffi_lib.setup(opts)
end

local function string_to_lines(qrcode_str)
    local lines = {}
    for s in qrcode_str:gmatch("[^\r\n]+") do
        table.insert(lines, s)
    end
    return lines
end


local function show_qr_code(buffer_id, qrcode)
    -- Make it temporarily writable so we don't have warnings.

    local lines = string_to_lines(qrcode)

    local editor_dim = { width = vim.o.columns, height = vim.o.lines }
    local window_dim = { width = #lines * 2 - 1, height = #lines }



    local padding = 1
    local right_bottom_corner = {
        row = editor_dim.height - window_dim.height - padding,
        col = editor_dim.width - window_dim.width - padding,
    }

    local title = "QR Code"
    vim.api.nvim_open_win(buffer_id, false,
        {
            relative = "editor",
            border = "rounded",
            anchor = "NW",
            row = right_bottom_corner.row,
            col = right_bottom_corner.col,
            width = window_dim.width,
            height = window_dim.height,
            style = "minimal",
        })





    -- Make it temporarily writable so we don't have warnings.
    vim.api.nvim_buf_set_option(buffer_id, "readonly", false)

    -- creating centered title
    local centered_title = string.rep(" ", math.floor((right_bottom_corner.row - #title) / 2)) .. title

    vim.api.nvim_buf_set_lines(buffer_id, 0, -1, false, { centered_title, string.rep("_", window_dim.width) })

    -- Append the qrcode.
    vim.api.nvim_buf_set_lines(buffer_id, -1, -1, true, lines)

    -- Make readonly again.
    vim.api.nvim_buf_set_option(buffer_id, "readonly", true)

    -- disable opts
    local disable_opts = { "number", "relativenumber", "modified" }

    for _, opt in ipairs(disable_opts) do
        vim.api.nvim_buf_set_option(buffer_id, opt, false)
    end
end

local function connect(args)
    utils.print_table(args)
    local qrcode_buffer = vim.api.nvim_create_buf(false, true)
    local qrcode_str = ffi_lib.connect(function(msg)
        if msg == "Connected" then
            vim.schedule(function()
                print(msg)
                vim.api.nvim_buf_delete(qrcode_buffer, { force = true })
            end)
        end
    end)

    show_qr_code(qrcode_buffer, qrcode_str)
end
vim.api.nvim_create_user_command("WIFIAndroidConnect", function()
    pcall(connect)
end, {})


return { setup = setup, connect = connect }
