viewport: 1200x750
mode: Zen
-----
# A tab dragged past the window's edge leaves for a window of its own,
# with the button still down: the move is decided when the cursor
# crosses the edge (a release outside a window is never delivered), not
# on the release.
#
# Two local shells, the first renamed so the strips can be told apart.
# The first chip is pressed, dragged along the strip and then 60 px
# above the top edge. The emulator adopts the window that opens (see
# tab-move-windows.ice), which holds the dragged tab alone; the other
# tab stayed where it was.
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
expect "alpha"
expect "bash (default)"

press (135, 20)
move (155, 26)
move (195, 10)
move (126, -20)
move (126, -60)
settle 500
expect "alpha"
absent "bash (default)"
release (126, -60)
settle 300
expect "alpha"
absent "bash (default)"
# It is a window like any other: the tab's menu offers the way back.
click right "alpha"
settle 250
expect "Move to Window: bash (default)"
