alias ..="cd .."
alias ...="cd ../.."
alias l='eza -a --icons=always'
alias ls='eza --icons=always'
alias ll='eza -lh --icons=always --git'
alias la='eza -lah --icons=always --git'
alias l-a='eza -a --icons=always'
alias tree='eza --tree --icons=always'

if command -v batcat &> /dev/null; then
  alias bat='batcat'
fi
alias cat='bat --paging=never --style=plain'

if command -v systemctl &> /dev/null; then
  alias service='systemctl'
fi
