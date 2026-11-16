viewport: 1200x750
mode: Zen
-----
# Issue #134: password fields render masked, the eye toggles, and the
# caret survives the round trip. text_input VALUES are invisible to
# `expect`, so the assertions ride on the clipboard: a copy out of a
# masked field yields the bullets that are on screen, and the same copy
# after revealing yields the real value. That also pins the insertion
# point, since a caret sent back to zero would spell "Zhunter2". The
# field is the New Identity form's password (the shared masked input).
expect "Welcome to Obsidian SSH"
click "Skip"
click "Get started"
settle 250
click "Keychain"
settle 250
click "New Identity"
settle 250
click (990, 304)
type "hunter2"
settle 300
screenshot password-mask-masked
click (1165, 303)
settle 300
click (1165, 303)
settle 300
type "Z"
settle 300
clipboard "seed"
type ctrl+a
type ctrl+c
settle 300
clipboard is "••••••••"
screenshot password-mask-after-toggle
click (1165, 303)
settle 300
type ctrl+a
type ctrl+c
settle 300
clipboard is "hunter2Z"
screenshot password-mask-revealed
