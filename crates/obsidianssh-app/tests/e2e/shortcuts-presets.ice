viewport: 1200x750
mode: Zen
-----
# Settings > Shortcuts: a preset is picked in the selector, confirmed
# (the dialog names the actions it leaves with no shortcut), applied, and
# the selector then names it; picking Obsidian SSH (default) the same way brings
# the factory table back. The selector's options are clicked by position
# (a dropdown's rows are not text-selectable): Obsidian SSH first, then the
# platform's presets, Tabby alone on Linux. The rows a preset changes and
# what it keeps (mouse and wheel chords) are pinned by the unit tests in
# hotkeys/presets.rs; this pins the door.
expect "Welcome to Obsidian SSH"
click "Skip"
click "Continue without password"
settle 250
# Toolbar gear (icon only, no text selector).
click (1175, 64)
settle
click "Shortcuts"
settle
expect "Obsidian SSH (default)"
expect "Export…"
expect "Import…"
click (1075.00, 132.00)
settle 300
click (1075.00, 214.00)
settle 300
expect "Apply"
click "Apply"
expect "Shortcuts set: Tabby"
settle 300
expect "Tabby"
absent "Obsidian SSH (default)"
# Back to Obsidian SSH's own through the same selector.
click (1075.00, 132.00)
settle 300
click (1075.00, 173.00)
settle 300
expect "Apply"
click "Apply"
expect "Shortcuts set: Obsidian SSH"
settle 300
expect "Obsidian SSH (default)"
