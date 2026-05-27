#!/bin/zsh

# Basic heredoc
cat <<EOF
Hello, world!
This is a heredoc.
EOF

# Heredoc with variable expansion
cat <<EOF
User: $USER
Home: $HOME
EOF

# Heredoc with indentation stripping
cat <<-EOF
	This line is indented with a tab
	So is this one
EOF

# Quoted heredoc (no expansion)
cat <<'EOF'
$USER will not be expanded
$(date) will not run
EOF
