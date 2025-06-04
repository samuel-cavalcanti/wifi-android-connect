local M = {}

local function setup(opts)
        local default_ops = {
                pair_code = nil,
                pair_name = "WIFI Android Connect nvim",
                timeout_in_seconds = 2 * 60
        }


        M.opts = opts or default_ops
end


local function show_qr_code(buffer_id, qrcode)
        local lines = qrcode -- string_to_lines(qrcode)

        local editor_dim = { width = vim.o.columns, height = vim.o.lines }
        local window_dim = { width = #lines * 2 - 3, height = #lines }



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





        local function readonly(value)
                -- vim.api.nvim_set_option_value("readonly", value, { buf = buffer_id })
                vim.api.nvim_buf_set_option(buffer_id, "readonly", value)
        end

        -- Make it temporarily writable so we don't have warnings.
        readonly(false)

        -- creating centered title
        local centered_title = string.rep(" ", math.floor((right_bottom_corner.row - #title) / 2)) .. title

        vim.api.nvim_buf_set_lines(buffer_id, 0, -1, false, { centered_title, string.rep("_", window_dim.width) })

        -- Append the qrcode.
        vim.api.nvim_buf_set_lines(buffer_id, -1, -1, true, lines)

        -- Make readonly again.
        readonly(true)

        -- disable opts
        local disable_opts = { "number", "relativenumber", "modified" }

        for _, opt in ipairs(disable_opts) do
                vim.api.nvim_buf_set_option(buffer_id, opt, false)
                -- vim.api.nvim_set_option_value(opt, false, { buf = buffer_id })
        end
end

local function get_current_file_dir()
        local str = debug.getinfo(1, "S").source:sub(2)
        return vim.fn.fnamemodify(str, ":p:h")
end


local function connect()
        local qrcode_buffer = vim.api.nvim_create_buf(false, true)
        local completed = false
        local current_dir = get_current_file_dir()
        local bin_file = "wifi-android-connect"
        local job_id = vim.fn.jobstart( current_dir .."/" .. bin_file, {
                on_stdout = function(_, data)
                        if data then
                                show_qr_code(qrcode_buffer, data)
                        end
                end,
                on_stderr = function(_, _)
                        -- if data then
                        --         print('error:')
                        --         require('utils').print_table(data)
                        -- end
                end,
                on_exit = function(_, exit_code)
                        vim.api.nvim_buf_delete(qrcode_buffer, { force = true })
                        completed = true
                        if exit_code == 0 then
                                print('Connected')
                        end
                end
        })

        vim.fn.timer_start(M.opts.timeout_in_seconds * 1000, function()
                if not completed then
                        print('timeout: ' .. M.opts.timeout_in_seconds .. ' seconds have been passed')
                        vim.fn.jobstop(job_id)
                end
        end)
end



vim.api.nvim_create_user_command("WIFIAndroidConnect", function()
        pcall(connect)
end, {})

return { setup = setup, connect = connect }
