---@diagnostic disable:missing-return
---@diagnostic disable:lowercase-global
---@diagnostic disable:unused-local
gtk                        = {}

---@enum Orientation
gtk.Orientation            = {
    Horizontal = 0,
    Vertical = 1,
}

---@enum Align
gtk.Align                  = {
    Fill = 0,
    Start = 1,
    End = 2,
    Center = 3,
    Baseline = 4,
}

---@enum EllipsizeMode
gtk.EllipsizeMode          = {
    None = 0,
    Start = 1,
    Middle = 2,
    End = 3
}

---@enum Operator
gtk.Operator               = {
    Clear = 0,
    Source = 1,
    Over = 2,
    In = 3,
    Out = 4,
    Atop = 5,
    Dest = 6,
    DestOver = 7,
    DestIn = 8,
    DestOut = 9,
    DestAtop = 10,
    Xor = 11,
    Add = 12,
    Saturate = 13,
    Multiply = 14,
    Screen = 15,
    Overlay = 16,
    Darken = 17,
    Lighten = 18,
    ColorDodge = 19,
    ColorBurn = 20,
    HardLight = 21,
    SoftLight = 22,
    Difference = 23,
    Exclusion = 24,
    HslHue = 25,
    HslSaturation = 26,
    HslColor = 27,
    HslLuminosity = 28
}

---@class Priority
local Priority             = {}

gtk.Priority               = {
    ---@type Priority
    HIGH = nil,
    ---@type Priority
    DEFAULT = nil,
    ---@type Priority
    HIGH_IDLE = nil,
    ---@type Priority
    DEFAULT_IDLE = nil,
    ---@type Priority
    LOW = nil,
}

---@enum RevealerTransitionType
gtk.RevealerTransitionType = {
    None = 0,
    Crossfade = 1,
    SlideRight = 2,
    SlideLeft = 3,
    SlideUp = 4,
    SlideDown = 5,
    SwingRight = 6,
    SwingLeft = 7,
    SwingUp = 8,
    SwingDown = 9
}

---@class MainContext
gtk.MainContext            = {
    ---@return MainContext
    new = function() end,

    ---@return MainContext
    default = function() end,

    ---@return MainContext?
    thread_default = function() end,

    ---@param self MainContext
    ---@param callback fun():nil
    spawn_local = function(self, callback) end,

    ---@param self MainContext
    ---@param priority Priority
    ---@param callback fun():nil
    spawn_local_with_priority = function(self, priority, callback) end
}

---@class ApplicationFlags
---@field is_service boolean?
---@field is_launcher boolean?
---@field handles_open boolean?
---@field handles_command_line boolean?
---@field send_environment boolean?
---@field non_unique boolean?
---@field can_override_app_id boolean?
---@field allow_replacement boolean?
---@field replace boolean?

---@class Application
gtk.Application            = {
    ---@param id string
    ---@param flags ApplicationFlags?
    ---@return Application
    new = function(id, flags) end,

    ---@param self Application
    ---@param callback fun():nil
    connect_activate = function(self, callback) end,

    ---@param self Application
    ---@param callback fun():nil
    connect_startup = function(self, callback) end,

    ---@param self Application
    ---@param callback fun():nil
    connect_shutdown = function(self, callback) end,

    ---@param self Application
    run = function(self) end
}

---@class WidgetImpl
local WidgetImpl           = {
    ---@param self WidgetImpl
    ---@return Widget
    upcast = function(self) end,

    ---@param self WidgetImpl
    ---@param controller EventController
    add_controller = function(self, controller) end,

    ---@param self WidgetImpl
    ---@param controller EventController
    remove_controller = function(self, controller) end,

    ---@param self WidgetImpl
    ---@param visible boolean
    ---@return Widget
    set_visible = function(self, visible) end,

    ---@param self WidgetImpl
    ---@return boolean
    get_visible = function(self) end,

    ---@param self WidgetImpl
    ---@param class string
    set_css_class = function(self, class) end,

    ---@param self WidgetImpl
    ---@param classes string[]
    set_css_classes = function(self, classes) end,

    ---@param self WidgetImpl
    ---@param class string
    add_css_class = function(self, class) end,

    ---@param self WidgetImpl
    ---@param class string
    remove_css_class = function(self, class) end,

    ---@param self WidgetImpl
    ---@param sensitive boolean
    set_sensitive = function(self, sensitive) end,

    ---@param self WidgetImpl
    ---@return boolean
    is_sensitive = function(self) end,

    ---@param self WidgetImpl
    ---@param align Align
    set_valign = function(self, align) end,

    ---@param self WidgetImpl
    ---@param align Align
    set_halign = function(self, align) end,

    ---@param self WidgetImpl
    ---@param expand boolean
    set_vexpand = function(self, expand) end,

    ---@param self WidgetImpl
    ---@param expand boolean
    set_hexpand = function(self, expand) end,

    ---@param self WidgetImpl
    ---@param margin integer
    set_margin_bottom = function(self, margin) end,

    ---@param self WidgetImpl
    ---@param margin integer
    set_margin_end = function(self, margin) end,

    ---@param self WidgetImpl
    ---@param margin integer
    set_margin_start = function(self, margin) end,

    ---@param self WidgetImpl
    ---@param margin integer
    set_margin_top = function(self, margin) end,

    ---@param self WidgetImpl
    ---@param width integer
    ---@param height integer
    set_size_request = function(self, width, height) end,

    ---@param self WidgetImpl
    queue_draw = function(self) end,

    ---@param self WidgetImpl
    grab_focus = function(self) end,

    ---@param self WidgetImpl
    ---@return Settings
    settings = function(self) end,

    ---@param self WidgetImpl
    ---@return RGBA
    color = function(self) end,

    ---@param self WidgetImpl
    ---@return number
    margin_bottom = function(self) end,

    ---@param self WidgetImpl
    ---@return number
    margin_end = function(self) end,

    ---@param self WidgetImpl
    ---@return number
    margin_start = function(self) end,

    ---@param self WidgetImpl
    ---@return number
    margin_top = function(self) end,

    ---@param self WidgetImpl
    ---@return integer
    allocated_width = function(self) end,

    ---@param self WidgetImpl
    ---@return integer
    allocated_height = function(self) end,
}

---@class Widget
local Widget               = {}

---@class ApplicationWindow : WidgetImpl
gtk.ApplicationWindow      = {
    ---@param app Application
    ---@return ApplicationWindow
    new = function(app) end,

    ---@param self ApplicationWindow
    ---@param title string?
    set_title = function(self, title) end,

    ---@param self ApplicationWindow
    ---@param child Widget?
    set_child = function(self, child) end,

    ---@param self ApplicationWindow
    close = function(self) end,

    ---@param self ApplicationWindow
    present = function(self) end
}

---@class Box : WidgetImpl
gtk.Box                    = {
    ---@param orientation Orientation
    ---@param spacing number?
    ---@return Box
    new = function(orientation, spacing) end,

    ---@param self Box
    ---@param widget Widget
    append = function(self, widget) end,

    ---@param self Box
    ---@param widget Widget
    remove = function(self, widget) end,

    ---@param self Box
    remove_all = function(self) end
}
---@class Grid : WidgetImpl
gtk.Grid                   = {
    ---@return Grid
    new = function() end,

    ---@param self Grid
    ---@param widget Widget
    ---@param column integer
    ---@param row integer
    ---@param width integer?
    ---@param height integer?
    attach = function(self, widget, column, row, width, height) end,

    ---@param self Grid
    ---@param widget Widget
    remove = function(self, widget) end,

    ---@param self Grid
    remove_all = function(self) end,

    ---@param self Grid
    ---@param spacing integer
    set_column_spacing = function(self, spacing) end,

    ---@param self Grid
    ---@param homogeneous boolean
    set_column_homogeneous = function(self, homogeneous) end,

    ---@param self Grid
    ---@param spacing integer
    set_row_spacing = function(self, spacing) end,

    ---@param self Grid
    ---@param homogeneous boolean
    set_row_homogeneous = function(self, homogeneous) end
}

---@class CenterBox : WidgetImpl
gtk.CenterBox              = {
    ---@return CenterBox
    new = function() end,

    ---@param self CenterBox
    ---@param widget? Widget
    set_start_widget = function(self, widget) end,

    ---@param self CenterBox
    ---@param widget? Widget
    set_center_widget = function(self, widget) end,

    ---@param self CenterBox
    ---@param widget? Widget
    set_end_widget = function(self, widget) end
}

---@class Button : WidgetImpl
gtk.Button                 = {
    ---@return Button
    new = function() end,

    ---@param label string
    ---@return Button
    with_label = function(label) end,

    ---@param self Button
    ---@param callback fun():nil
    connect_clicked = function(self, callback) end,

    ---@param self Button
    ---@param label string
    set_label = function(self, label) end,

    ---@param self Button
    ---@param child Widget
    set_child = function(self, child) end
}

---@class CheckButton : WidgetImpl
gtk.CheckButton            = {
    ---@return CheckButton
    new              = function() end,

    ---@param label string
    ---@return CheckButton
    with_label       = function(label) end,

    ---@param self CheckButton
    ---@param callback fun():nil
    connect_toggled  = function(self, callback) end,

    ---@param self CheckButton
    ---@param setting boolean
    set_active       = function(self, setting) end,

    ---@param self CheckButton
    ---@param child Widget?
    set_child        = function(self, child) end,

    ---@param self CheckButton
    ---@param group CheckButton?
    set_group        = function(self, group) end,

    ---@param self CheckButton
    ---@param inconsistent boolean
    set_inconsistent = function(self, inconsistent) end,

    ---@param self CheckButton
    ---@param label string?
    set_label        = function(self, label) end,
}

---@class Overlay : WidgetImpl
gtk.Overlay                = {
    ---@return Overlay
    new = function() end,

    ---@param self Overlay
    ---@param child Widget
    set_child = function(self, child) end,

    ---@param self Overlay
    ---@param widget Widget
    add_overlay = function(self, widget) end,

    ---@param self Overlay
    ---@param widget Widget
    remove_overlay = function(self, widget) end,

    ---@param self Overlay
    ---@param widget Widget
    ---@param measure boolean
    set_measure_overlay = function(self, widget, measure) end,

    ---@param self Overlay
    ---@param widget Widget
    ---@param clip_overlay boolean
    set_clip_overlay = function(self, widget, clip_overlay) end,
}

---@class Label : WidgetImpl
gtk.Label                  = {
    ---@param str? string
    ---@return Label
    new = function(str) end,

    ---@param self Label
    ---@param str string
    set_label = function(self, str) end,

    ---@param self Label
    ---@param markup string
    set_markup = function(self, markup) end,

    ---@param self Label
    ---@param mode EllipsizeMode
    set_ellipsize = function(self, mode) end
}

---@class EntryBuffer
local EntryBuffer          = {
    ---@param self EntryBuffer
    ---@return string
    text = function(self) end,

    ---@param self EntryBuffer
    ---@param callback fun(position: integer, n_chars: integer):nil
    ---@param after boolean?
    connect_deleted_text = function(self, callback, after) end,

    ---@param self EntryBuffer
    ---@param callback fun(position: integer, chars:string, n_chars: integer):nil
    ---@param after boolean?
    connect_inserted_text = function(self, callback, after) end,

    ---@param self EntryBuffer
    ---@param chars string
    set_text = function(self, chars) end
}

---@class Entry : WidgetImpl
gtk.Entry                  = {
    ---@return Entry
    new = function() end,

    ---@param self Entry
    ---@param callback fun():nil
    connect_activate = function(self, callback) end,

    ---@param self Entry
    ---@return EntryBuffer
    buffer = function(self) end,

    ---@param self Entry
    ---@param text? string
    set_placeholder_text = function(self, text) end,

    ---@param self Entry
    ---@param xalign number
    set_alignment = function(self, xalign) end,

    ---@param self Entry
    ---@param visible boolean
    set_visibility = function(self, visible) end,

    ---@param self Entry
    ---@param max integer
    set_max_length = function(self, max) end,

    ---@param self Entry
    grab_focus_without_selecting = function(self) end,
}

---@class CairoContext
gtk.CairoContext           = {
    ---@param self CairoContext
    ---@param red number
    ---@param green number
    ---@param blue number
    set_source_rgb = function(self, red, green, blue) end,

    ---@param self CairoContext
    ---@param red number
    ---@param green number
    ---@param blue number
    ---@param alpha number
    set_source_rgba = function(self, red, green, blue, alpha) end,

    ---@param self CairoContext
    ---@param x number
    ---@param y number
    move_to = function(self, x, y) end,

    ---@param self CairoContext
    ---@param dx number
    ---@param dy number
    rel_move_to = function(self, dx, dy) end,

    ---@param self CairoContext
    ---@param x number
    ---@param y number
    line_to = function(self, x, y) end,

    ---@param self CairoContext
    ---@param dx number
    ---@param dy number
    rel_line_to = function(self, dx, dy) end,

    ---@param self CairoContext
    ---@param xc number
    ---@param yc number
    ---@param radius number
    ---@param angle1 number
    ---@param angle2 number
    arc = function(self, xc, yc, radius, angle1, angle2) end,

    ---@param self CairoContext
    ---@param xc number
    ---@param yc number
    ---@param radius number
    ---@param angle1 number
    ---@param angle2 number
    arc_negative = function(self, xc, yc, radius, angle1, angle2) end,

    ---@param self CairoContext
    ---@param x1 number
    ---@param y1 number
    ---@param x2 number
    ---@param y2 number
    ---@param x3 number
    ---@param y3 number
    curve_to = function(self, x1, y1, x2, y2, x3, y3) end,

    ---@param self CairoContext
    ---@param dx1 number
    ---@param dy1 number
    ---@param dx2 number
    ---@param dy2 number
    ---@param dx3 number
    ---@param dy3 number
    rel_curve_to = function(self, dx1, dy1, dx2, dy2, dx3, dy3) end,

    ---@param self CairoContext
    ---@param x number
    ---@param y number
    ---@param width number
    ---@param height number
    rectangle = function(self, x, y, width, height) end,

    ---@param self CairoContext
    ---@param tx number
    ---@param ty number
    translate = function(self, tx, ty) end,

    ---@param self CairoContext
    ---@param sx number
    ---@param sy number
    scale = function(self, sx, sy) end,

    ---@param self CairoContext
    ---@param angle number
    rotate = function(self, angle) end,

    ---@param self CairoContext
    new_path = function(self) end,

    ---@param self CairoContext
    new_sub_path = function(self) end,

    ---@param self CairoContext
    close_path = function(self) end,

    ---@param self CairoContext
    clip = function(self) end,

    ---@param self CairoContext
    paint = function(self) end,

    ---@param self CairoContext
    ---@param alpha number
    paint_with_alpha = function(self, alpha) end,

    ---@param self CairoContext
    stroke = function(self) end,

    ---@param self CairoContext
    fill = function(self) end,

    ---@param self CairoContext
    save = function(self) end,

    ---@param self CairoContext
    restore = function(self) end,
}

---@class DrawingArea : WidgetImpl
gtk.DrawingArea            = {
    ---@return DrawingArea
    new = function() end,

    ---@param self DrawingArea
    ---@param width integer
    set_content_width = function(self, width) end,

    ---@param self DrawingArea
    ---@param height integer
    set_content_height = function(self, height) end,

    ---@param self DrawingArea
    ---@param callback fun(ctx: CairoContext, width: integer, height: integer):nil
    set_draw_func = function(self, callback) end,

    ---@param self DrawingArea
    unset_draw_func = function(self) end,
}

---@class Image : WidgetImpl
gtk.Image                  = {
    ---@return Image
    new = function() end,

    ---@param path string
    ---@return Image
    from_file = function(path) end,

    ---@param icon_name string
    ---@return Image
    from_icon_name = function(icon_name) end,

    ---@param icon Icon
    ---@return Image
    from_gicon = function(icon) end,

    ---@param self Image
    ---@param pixel_size integer
    set_pixel_size = function(self, pixel_size) end,

    ---@param self Image
    ---@param path string
    set_from_file = function(self, path) end,

    ---@param self Image
    ---@param icon_name string
    set_from_icon_name = function(self, icon_name) end,

    ---@param self Image
    ---@param icon Icon
    set_from_gicon = function(self, icon) end,

    ---@param self Image
    clear = function(self) end
}

---@class Scale : WidgetImpl
gtk.Scale                  = {
    ---@param orientation Orientation
    ---@param min number
    ---@param max number
    ---@param step? number 1.0 by default
    ---@return Scale
    with_range = function(orientation, min, max, step) end,

    ---@param self Scale
    ---@param callback fun():nil
    connect_value_changed = function(self, callback) end,

    ---@param self Scale
    ---@param callback fun(value: number):nil
    connect_adjust_bounds = function(self, callback) end,

    ---@param self Scale
    ---@param value number
    set_value = function(self, value) end,

    ---@param self Scale
    ---@return number
    value = function(self) end,

    ---@param self Scale
    ---@param min number
    ---@param max number
    set_range = function(self, min, max) end,

    ---@param self Scale
    ---@param setting boolean
    set_inverted = function(self, setting) end,

    ---@param self Scale
    ---@return boolean
    is_inverted = function(self) end,

    ---@param self Scale
    ---@return integer
    round_digits = function(self) end,

    ---@param self Scale
    ---@param round_digits integer
    set_round_digits = function(self, round_digits) end,

    ---@param self Scale
    ---@return integer
    digits = function(self) end,

    ---@param self Scale
    ---@param digits integer
    set_digits = function(self, digits) end,

    ---@param self Scale
    ---@return boolean
    draws_value = function(self) end,

    ---@param self Scale
    ---@param draw_value boolean
    set_draw_value = function(self, draw_value) end,

    ---@param self Scale
    ---@param step number
    ---@param page number
    set_increments = function(self, step, page) end,

    ---@param self Scale
    ---@param fill_level number
    set_fill_level = function(self, fill_level) end,

    ---@param self Scale
    ---@return number
    fill_level = function(self) end,

    ---@param self Scale
    ---@param restrict_to_fill_level boolean
    set_restrict_to_fill_level = function(self, restrict_to_fill_level) end,

    ---@param self Scale
    ---@return boolean
    restricts_to_fill_level = function(self) end,
}

---@class Revealer : WidgetImpl
gtk.Revealer               = {
    ---@return Revealer
    new = function() end,

    ---@param self Revealer
    ---@param child Widget
    set_child = function(self, child) end,

    ---@param self Revealer
    ---@param reveal_child boolean
    set_reveal_child = function(self, reveal_child) end,

    ---@param self Revealer
    ---@param duration integer
    set_transition_duration = function(self, duration) end,

    ---@param self Revealer
    ---@param transition RevealerTransitionType
    set_transition_type = function(self, transition) end,
}

---@class EventControllerImpl
local EventControllerImpl  = {
    ---@param self EventControllerImpl
    ---@return EventController
    upcast = function(self) end
}

---@class EventController = {}

---@class EventControllerScrollFlags
---@field vertical boolean?
---@field horizontal boolean?
---@field discrete boolean?
---@field kinetic boolean?
---@field both_axes boolean?

---@class ModifierType
---@field shift boolean?
---@field lock boolean?
---@field control boolean?
---@field alt boolean?
---@field button1 boolean?
---@field button2 boolean?
---@field button3 boolean?
---@field button4 boolean?
---@field button5 boolean?
---@field super boolean?
---@field hyper boolean?
---@field meta boolean?

---@class EventControllerKey : EventControllerImpl
gtk.EventControllerKey     = {
    ---@return EventControllerKey
    new = function() end,

    ---@param self EventControllerKey
    ---@param callback fun(key_name: string, keycode: integer, modifier_type: ModifierType):boolean?
    connect_key_pressed = function(self, callback) end,

    ---@param self EventControllerKey
    ---@param callback fun(key_name: string, keycode: integer, modifier_type: ModifierType):boolean?
    connect_key_released = function(self, callback) end,

    ---@param self EventControllerKey
    forward = function(self) end
}

---@class EventControllerScroll : EventControllerImpl
gtk.EventControllerScroll  = {
    ---@param flags EventControllerScrollFlags?
    ---@return EventControllerScroll
    new = function(flags) end,

    ---@param self EventControllerScroll
    ---@param callback fun():nil
    connect_scroll_begin = function(self, callback) end,

    ---@param self EventControllerScroll
    ---@param callback fun():nil
    connect_scroll_end = function(self, callback) end,

    ---@param self EventControllerScroll
    ---@param callback fun(dx: number, dy: number):boolean?
    connect_scroll = function(self, callback) end,

    ---@param self EventControllerScroll
    ---@param callback fun(vel_x: number, vel_y: number):nil
    connect_decelerate = function(self, callback) end,
}

---@class EventControllerMotion : EventControllerImpl
gtk.EventControllerMotion  = {
    ---@return EventControllerMotion
    new = function() end,

    ---@param self EventControllerMotion
    ---@param callback fun():nil
    connect_leave = function(self, callback) end,

    ---@param self EventControllerMotion
    ---@param callback fun(x: number, y: number):nil
    connect_enter = function(self, callback) end,

    ---@param self EventControllerMotion
    ---@param callback fun(x: number, y: number):nil
    connect_motion = function(self, callback) end,
}

---@class EventControllerFocus : EventControllerImpl
gtk.EventControllerFocus   = {
    ---@return EventControllerFocus
    new = function() end,

    ---@param self EventControllerFocus
    ---@param callback fun():nil
    connect_enter = function(self, callback) end,

    ---@param self EventControllerFocus
    ---@param callback fun():nil
    connect_leave = function(self, callback) end,
}

---@class Settings
gtk.Settings               = {
    ---@param self Settings
    ---@return string
    gtk_cursor_theme_name = function(self) end,

    ---@param self Settings
    ---@param cursor_theme_name string
    set_gtk_cursor_theme_name = function(self, cursor_theme_name) end,

    ---@param self Settings
    ---@return integer
    gtk_cursor_theme_size = function(self) end,

    ---@param self Settings
    ---@param cursor_theme_size integer
    set_gtk_cursor_theme_size = function(self, cursor_theme_size) end,

    ---@param self Settings
    ---@return string
    gtk_theme_name = function(self) end,

    ---@param self Settings
    ---@param theme_name string
    set_gtk_theme_name = function(self, theme_name) end,

    ---@param self Settings
    ---@return string
    gtk_icon_theme_name = function(self) end,

    ---@param self Settings
    ---@param icon_theme_name string
    set_gtk_icon_theme_name = function(self, icon_theme_name) end,
}

---@class CssProvider
gtk.CssProvider            = {
    ---@return CssProvider
    new = function() end,

    ---@param self CssProvider
    ---@param data string
    load_from_data = function(self, data) end,

    ---@param self CssProvider
    ---@param path string
    load_from_file = function(self, path) end,
}

---@param provider CssProvider
function gtk.style_context_add_provider(provider) end

gtk.layer_shell = {}

---@enum Layer
gtk.layer_shell.Layer = {
    Background = 0,
    Bottom = 1,
    Top = 2,
    Overlay = 3
}

---@enum Edge
gtk.layer_shell.Edge = {
    Left = 0,
    Right = 1,
    Top = 2,
    Bottom = 3
}

---@enum KeyboardMode
gtk.layer_shell.KeyboardMode = {
    None = 0,
    Exclusive = 1,
    OnDemand = 2,
}

---@param window ApplicationWindow
function gtk.layer_shell.init_for_window(window) end

---@param window ApplicationWindow
---@param layer Layer
function gtk.layer_shell.set_layer(window, layer) end

---@param window ApplicationWindow
function gtk.layer_shell.auto_exclusive_zone_enable(window) end

---@param window ApplicationWindow
---@param exclusive_zone integer
function gtk.layer_shell.set_exclusive_zone(window, exclusive_zone) end

---@param window ApplicationWindow
---@param edge Edge
---@param margin_size integer
function gtk.layer_shell.set_margin(window, edge, margin_size) end

---@param window ApplicationWindow
---@param edge Edge
---@param anchor_to_edge boolean
function gtk.layer_shell.set_anchor(window, edge, anchor_to_edge) end

---@param window ApplicationWindow
---@param mode KeyboardMode
function gtk.layer_shell.set_keyboard_mode(window, mode) end

---@param window ApplicationWindow
---@param namespace string
function gtk.layer_shell.set_namespace(window, namespace) end
