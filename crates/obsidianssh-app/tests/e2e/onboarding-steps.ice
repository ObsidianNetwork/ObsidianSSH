viewport: 1240x1500
mode: Zen
-----
# The first-run carousel end to end (onboarding.ice proves Skip jumps
# straight to the last step): welcome, connect, the SSH agent, the
# optional features, then the import offer that creates the vault.
#
# The import step cannot import before the vault exists: accepting it
# records the intent, creates the vault, and the hub opens on the
# vault's first frame, which is what the tail of this test pins.
expect "Welcome to Obsidian SSH"
click "Next"
expect "Connect to anything"
click "Next"
expect "Keys from your SSH agent"
expect "Bitwarden asks you to approve each connection"
click "Next"
settle
# Optional features: live toggles, same switches as Settings.
expect "Make it yours"
expect "AI Assistant"
expect "Host monitoring"
click "Next"
settle
# Import offer, the last step: Get started or Import.
expect "Bring your hosts along"
expect "Get started"
expect "Import my hosts"
click "Import my hosts"
settle
expect "Import hosts"
expect "Choose file..."
