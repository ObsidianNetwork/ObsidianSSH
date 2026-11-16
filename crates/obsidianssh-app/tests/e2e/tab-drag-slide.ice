viewport: 1200x750
mode: Zen
-----
# In-strip reorder drag, Chrome's rule: while the button is held, the
# dragged tab lands beside the chip under the cursor, on the side of its
# middle the cursor is on, and the release selects it.
#
# Three local shells, alpha / beta / gamma (gamma active). alpha is
# dragged right: past gamma's middle it lands last; the release lights
# it up. gamma is dragged left onto beta's left half: it lands first.
#
# After the move that promotes the drag, a `find` rebuilds the widget
# tree: the chips hear the cursor only during a drag (`on_move`, set in
# `view()`), and the emulator does not rebuild the view after an update
# no widget event caused (the promotion comes from the global cursor
# subscription). The windowed app rebuilds after every update.
#
# Then the strip overflows and sits scrolled to its end. A drag there
# used to reset the scroll (the ghost's Stack came and went, and widget
# state is matched by position) and land the tab by unscrolled
# rectangles, several slots off; epsilon dragged onto delta's left half
# lands right before delta.
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
# alpha (~151..265) right, past gamma's middle (gamma ~368..606).
press (116, 20)
move (126, 22)
find "beta"
move (256, 20)
move (426, 20)
settle 200
screenshot slide-1-alpha-dragged-right
release (426, 20)
settle 400
order "beta" "gamma" "alpha"
expect "● alpha"
# gamma (now ~262..366) left onto beta's left half (beta ~151..259).
press (226, 20)
move (216, 22)
find "beta"
move (106, 20)
settle 200
release (106, 20)
settle 400
order "gamma" "beta" "alpha"
expect "● gamma"
screenshot slide-2-gamma-dragged-left
# Overflow: ten more shells, then delta and epsilon at the strip's end
# (epsilon active, the strip scrolled to show it).
type ctrl+shift+l
settle 300
type ctrl+shift+l
settle 300
type ctrl+shift+l
settle 300
type ctrl+shift+l
settle 300
type ctrl+shift+l
settle 300
type ctrl+shift+l
settle 300
type ctrl+shift+l
settle 300
type ctrl+shift+l
settle 300
type ctrl+shift+l
settle 300
type ctrl+shift+l
settle 300
type ctrl+shift+l
settle 500
click right (686, 20)
settle 250
click "Rename Tab"
settle 250
type ctrl+a
type "epsilon"
type enter
settle 300
click right (516, 20)
settle 250
click "Rename Tab"
settle 250
type ctrl+a
type "delta"
type enter
settle 300
order "gamma" "delta" "epsilon"
screenshot slide-3-overflow-scrolled
# epsilon (~645..885) onto delta's left half (delta ~545..640).
press (686, 20)
move (676, 22)
find "delta"
move (506, 20)
settle 200
screenshot slide-4-scrolled-drag
release (506, 20)
settle 400
order "gamma" "epsilon" "delta"
expect "● epsilon"
screenshot slide-5-scrolled-released
