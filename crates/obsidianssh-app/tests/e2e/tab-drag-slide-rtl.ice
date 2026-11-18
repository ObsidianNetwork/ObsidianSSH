viewport: 1200x750
mode: Zen
-----
# The reorder drag across a right-to-left strip: the strip runs from the
# Home tab leftwards, so the far half of a chip, the one past its middle
# in strip order, is its LEFT half.
#
# Three local shells, then Settings > Interface > Layout Direction =
# Right to Left: alpha sits rightmost, then beta, then gamma. beta is
# dragged onto gamma's left half and lands after it (to its left); the
# release selects it. `order` reads left to right, so the strip lists in
# reverse. The `move` before the press hovers the chip: the harness
# press does not, and the drag arms on the hovered chip. The `find`
# after the promoting move rebuilds the widget tree (see
# tab-drag-slide.ice).
expect "Welcome to Obsidian SSH"
click "Skip"
click "Get started"
expect "Create host"
timeout 800
type ctrl+shift+l
settle 500
click right "bash (default)"
settle 250
click "Rename Tab"
settle 250
type ctrl+a
type "alpha"
type enter
settle 300
type ctrl+shift+l
settle 500
click right "bash (default)"
settle 250
click "Rename Tab"
settle 250
type ctrl+a
type "beta"
type enter
settle 300
type ctrl+shift+l
settle 700
click right "bash (default)"
settle 250
click "Rename Tab"
settle 250
type ctrl+a
type "gamma"
type enter
settle 300
order "alpha" "beta" "gamma"
expect "● gamma"
click (19, 20)
settle 300
click "Settings"
settle 500
scroll (0, 60) (600, 400)
settle 300
click (1038, 184)
settle 300
click "Right to Left"
settle 500
click "alpha"
settle 500
order "gamma" "beta" "alpha"
# The narrower Home tab moves RTL chips right by 74 px. Measured before
# dragging: beta's label starts at x 785, gamma's at x 674.
move (794, 22)
settle 200
press (794, 20)
move (786, 22)
find "gamma"
move (664, 20)
settle 200
screenshot rtl-1-beta-dragged-left
release (664, 20)
settle 400
order "beta" "gamma" "alpha"
expect "● beta"
screenshot rtl-2-released
