viewport: 1200x900
mode: Zen
-----
# Enter and Add create real tags, removal survives closing the editor,
# and Save includes the unfinished draft. Macro suggestions share the
# same tags as hosts. Enter in a command still inserts a newline.
click "Skip"
click "Get started"
click "Add host"
click "IP or Hostname"
type "192.0.2.50"
click "My Server"
type "BubbleHost"
click #editor-tags
type "prod"
type enter
expect "prod"
expect "New Host"
type "web"
click "Add"
expect "web"
click "Save"
expect "BubbleHost"
click right "BubbleHost"
click "Edit"
expect "prod"
expect "web"
# The first bubble's cross in the host editor.
click (872, 321)
absent "prod"
click #editor-tags
type "database"
click (1169, 72)
click right "BubbleHost"
click "Edit"
expect "web"
expect "database"
absent "prod"
screenshot host-tag-bubbles
click (1169, 72)
click "Macros"
click "MACRO"
click "restart-nginx"
type "bubble-macro"
click #panel-snippet-tags
type "blue"
type enter
expect "blue"
type "green"
click "Add"
expect "green"
# Removing a bubble preserves the other tag.
click (857, 296)
absent "blue"
expect "green"
click "+ web"
click #panel-snippet-tags
type "pending"
click #panel-snippet-step-0
type "echo one"
type enter
type "echo two"
click "Save"
expect "bubble-macro"
expect "#green #web #pending"
click right "bubble-macro"
click "Edit"
expect "green"
expect "web"
expect "pending"
absent "blue"
click #panel-snippet-step-0
type ctrl+a
type ctrl+c
clipboard is "echo one\necho two"
screenshot macro-tag-bubbles
click "Cancel"
# The compact editor uses the same state and wraps bubbles to its width.
# A live shell keeps tasks pending: pace by time from here on.
timeout 500
type ctrl+shift+l
settle 400
expect "bash (default)"
click (1036, 20)
click (911, 64)
settle 300
click "MACRO"
click "restart-nginx"
type "sidebar-bubbles"
click #sidebar-snippet-tags
type "sidebar"
type enter
expect "sidebar"
type "wrapped-tag-one, wrapped-tag-two, wrapped-tag-three, wrapped-tag-four,"
expect "wrapped-tag-four"
# Remove the last bubble, then save a new tag without pressing Enter.
click (994, 378)
absent "wrapped-tag-four"
click #sidebar-snippet-tags
type "unfinished"
click #panel-snippet-step-0
type "echo sidebar"
screenshot sidebar-tag-bubbles
click "Save"
expect "sidebar-bubbles"
expect "#unfinished"
absent "wrapped-tag-four"
