#!/usr/bin/env sh
# RTB (رتّب) Setup Wizard — Linux / macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/3mr-5aled/rtb/main/install.sh | sh
set -e

# Non-interactive / CI / Quiet Detection
RTB_QUIET="${RTB_QUIET:-0}"
if [ -n "$CI" ] || [ -n "$GITHUB_ACTIONS" ] || [ "$RTB_NON_INTERACTIVE" = "true" ] || [ "$RTB_NON_INTERACTIVE" = "1" ]; then
    RTB_QUIET="1"
fi

# TTY & Interactive Resolution
IS_TTY=0
if [ -t 1 ] && { [ -t 0 ] || [ -r /dev/tty ]; }; then
    IS_TTY=1
fi

# ANSI Capability Detection
if [ "$RTB_QUIET" = "1" ] || [ ! -t 1 ]; then
    ANSI=0
else
    case "${TERM:-}" in
        xterm*|screen*|*256color*|*color*|alacritty*|kitty*) ANSI=1 ;;
        *) ANSI=0 ;;
    esac
fi

esc() {
    if [ "$ANSI" = "1" ]; then
        printf '\033[%sm' "$1"
    fi
}

write_step() {
    if [ "$RTB_QUIET" = "1" ]; then
        printf '[%s/%s] %s\n' "$1" "$2" "$3"
    else
        printf '  %s%s[%s/%s]%s ◆ %s\n' "$(esc '1m')" "$(esc '36m')" "$1" "$2" "$(esc '0m')" "$3"
    fi
}

write_warn() {
    printf '  %s⚠  %s%s\n' "$(esc '33m')" "$1" "$(esc '0m')"
}

write_fail() {
    printf '  %s✗  %s%s\n' "$(esc '31m')" "$1" "$(esc '0m')"
    exit 1
}

# Spinner State & Traps
SPINNER_PID=""
SPINNER_LABEL=""

cleanup() {
    if [ -n "$SPINNER_PID" ]; then
        kill "$SPINNER_PID" 2>/dev/null || true
        wait "$SPINNER_PID" 2>/dev/null || true
        SPINNER_PID=""
    fi
    if [ "$ANSI" = "1" ]; then
        printf '\033[?25h\r%60s\r' ''
    fi
}

trap cleanup EXIT INT TERM

start_spinner() {
    SPINNER_LABEL="$1"
    if [ "$RTB_QUIET" = "1" ] || [ "$ANSI" != "1" ]; then
        printf '  ... %s\n' "$SPINNER_LABEL"
        return
    fi

    printf '\033[?25l'
    (
        trap 'exit 0' TERM INT
        set -- '⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏'
        while true; do
            for frame in "$@"; do
                printf '\r  \033[36m%s\033[0m  \033[90m%s\033[0m' "$frame" "$SPINNER_LABEL"
                sleep 0.08 2>/dev/null || sleep 1 2>/dev/null || true
            done
        done
    ) &
    SPINNER_PID=$!
}

stop_spinner() {
    success="$1"
    label="${2:-$SPINNER_LABEL}"

    if [ -n "$SPINNER_PID" ]; then
        kill "$SPINNER_PID" 2>/dev/null || true
        wait "$SPINNER_PID" 2>/dev/null || true
        SPINNER_PID=""
    fi

    if [ "$RTB_QUIET" = "1" ]; then
        return
    fi

    printf '\r%60s\r' ''
    if [ "$success" = "1" ]; then
        printf '  %s✔%s  %s\n' "$(esc '32m')" "$(esc '0m')" "$label"
    else
        printf '  %s✗%s  %s\n' "$(esc '31m')" "$(esc '0m')" "$label"
    fi
}

get_rtb_version() {
    for f in "./VERSION" "../VERSION" "$SCRIPT_DIR/VERSION" "core/package.json" "../core/package.json"; do
        if [ -f "$f" ]; then
            case "$f" in
                *.json)
                    v=$(grep '"version"' "$f" | head -n 1 | sed -E 's/.*"version": "([^"]+)".*/\1/')
                    if [ -n "$v" ]; then echo "$v"; return; fi
                    ;;
                *)
                    v=$(head -n 1 "$f" | tr -d '\r\n ' | sed 's/^v//')
                    if [ -n "$v" ]; then echo "$v"; return; fi
                    ;;
            esac
        fi
    done
    echo "0.5.2"
}
RTB_VERSION="$(get_rtb_version)"

show_header() {
    if [ "$RTB_QUIET" = "1" ]; then
        return
    fi
    c="$(esc '36m')"
    b="$(esc '1m')"
    d="$(esc '90m')"
    g="$(esc '32m')"
    r="$(esc '0m')"

    printf '\n'
    printf '  %s%sRTB%s %s(رتّب) Setup Wizard%s %sv%s%s\n' "$b" "$c" "$r" "$b" "$r" "$g" "$RTB_VERSION" "$r"
    printf '  %sCross-platform developer tooling & workspace manager (v%s)%s\n' "$d" "$RTB_VERSION" "$r"
    printf '\n'
}

prompt_input() {
    p_text="$1"
    p_default="$2"

    if [ "$RTB_QUIET" = "1" ] || [ "$IS_TTY" = "0" ]; then
        echo "$p_default"
        return
    fi

    printf '%b' "$p_text" >&2

    p_ans=""
    if [ -t 0 ]; then
        read -r p_ans || p_ans=""
    elif [ -r /dev/tty ]; then
        read -r p_ans < /dev/tty 2>/dev/null || p_ans=""
    fi

    if [ -n "$p_ans" ]; then
        echo "$p_ans"
    else
        echo "$p_default"
    fi
}

detect_os_arch() {
    RAW_OS="${RTB_OS_OVERRIDE:-$(uname -s 2>/dev/null || echo 'Unknown')}"
    case "$RAW_OS" in
        Linux*)  OS_SLUG="linux" ;;
        Darwin*) OS_SLUG="macos" ;;
        MINGW*|MSYS*|CYGWIN*)
            OS_SLUG="linux"
            ;;
        *)       write_fail "Unsupported operating system: $RAW_OS. RTB supports Linux and macOS (use install.ps1 on Windows)." ;;
    esac

    RAW_ARCH="${RTB_ARCH_OVERRIDE:-$(uname -m 2>/dev/null || echo 'unknown')}"
    case "$RAW_ARCH" in
        x86_64|amd64|x64)     ARCH_SLUG="amd64" ;;
        aarch64|arm64|armv8*) ARCH_SLUG="arm64" ;;
        *)                    ARCH_SLUG="amd64" ;;
    esac
}

ensure_node() {
    if command -v node >/dev/null 2>&1; then
        NODE_MAJOR="$(node -v 2>/dev/null | tr -d 'v' | cut -d. -f1)"
        if [ -n "$NODE_MAJOR" ] && [ "$NODE_MAJOR" -ge 18 ]; then
            return 0
        fi
        write_warn "Node.js is installed but version ($NODE_MAJOR) is less than required (>= 18)."
    else
        write_warn "Node.js (>= 18) was not found on your system."
    fi

    if [ "$RTB_QUIET" = "1" ] || [ "$IS_TTY" = "0" ]; then
        write_fail "Node.js >= 18 is required. Install from https://nodejs.org or via your package manager."
    fi

    printf '  %s?%s Node.js is required. Would you like installation instructions? [Y/n] ' "$(esc '32m')" "$(esc '0m')" >&2
    if [ -t 0 ]; then
        read -r ans || ans="y"
    elif [ -r /dev/tty ]; then
        read -r ans < /dev/tty 2>/dev/null || ans="y"
    else
        ans="y"
    fi

    case "$ans" in
        [Nn]*) write_fail "Node.js installation declined. Install Node.js >= 18 and re-run installer." ;;
    esac

    printf '\n'
    if [ "$OS_SLUG" = "macos" ]; then
        printf '  %sInstall via Homebrew:%s\n' "$(esc '36m')" "$(esc '0m')"
        printf '    brew install node\n\n'
    else
        printf '  %sInstall via NodeSource (Ubuntu/Debian):%s\n' "$(esc '36m')" "$(esc '0m')"
        printf '    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -\n'
        printf '    sudo apt-get install -y nodejs\n\n'
    fi
    printf '  %sOr via fnm / nvm:%s\n' "$(esc '36m')" "$(esc '0m')"
    printf '    curl -fsSL https://fnm.vercel.app/install | bash\n'
    printf '    fnm install 20\n\n'

    write_fail "Please install Node.js >= 18 and rerun this installer."
}

show_summary() {
    ipath="$1"
    g="$(esc '32m')"
    b="$(esc '1m')"
    c="$(esc '36m')"
    d="$(esc '90m')"
    r="$(esc '0m')"

    printf '\n'
    printf '  %s%s✔ RTB v%s installed successfully!%s\n\n' "$b" "$g" "$RTB_VERSION" "$r"
    printf '  %sRTB Version:%s   v%s\n' "$c" "$r" "$RTB_VERSION"
    printf '  %sInstall path:%s  %s\n' "$c" "$r" "$ipath"
    printf '  %sNode runtime:%s  %s\n\n' "$c" "$r" "$(node -v 2>/dev/null || echo 'node')"
    printf '  %sNext steps:%s\n' "$b" "$r"
    printf '    %srtb doctor%s  %s- check workspace health and tools%s\n' "$g" "$r" "$d" "$r"
    printf '    %srtb goto%s    %s- jump across projects with fuzzy matching%s\n' "$g" "$r" "$d" "$r"
    printf '    %srtb agent%s   %s- launch AI agents (agy, claude, etc.)%s\n' "$g" "$r" "$d" "$r"
    printf '    %srtb ui%s      %s- open the interactive terminal dashboard%s\n\n' "$g" "$r" "$d" "$r"
}

install_steps() {
    TOTAL=4
    XDG_CFG="${XDG_CONFIG_HOME:-$HOME/.config}"
    DEFAULT_DIR="$XDG_CFG/rtb"
    PROMPT_TEXT="  $(esc '32m')?$(esc '0m') Install location $(esc '90m')(Enter to accept)$(esc '0m')\n    $(esc '90m')$DEFAULT_DIR$(esc '0m')\n  › "

    RTB_DIR="${RTB_INSTALL_PATH:-$(prompt_input "$PROMPT_TEXT" "$DEFAULT_DIR")}"
    LIB_DIR="$RTB_DIR/lib"
    BIN_DIR="${RTB_BIN_DIR:-$RTB_DIR/bin}"

    # Step 1: Directories
    write_step 1 $TOTAL 'Creating directories'
    start_spinner 'Setting up install directories'
    mkdir -p "$RTB_DIR" "$LIB_DIR" "$BIN_DIR" || {
        stop_spinner 0 'Setup install directories'
        write_fail "Cannot create install directories in $RTB_DIR"
    }
    stop_spinner 1 'Created install directories'

    # Step 2: Deploy TypeScript CLI Bundle
    write_step 2 $TOTAL 'Deploying RTB CLI engine'
    SCRIPT_DIR="$(cd "$(dirname "$0")" 2>/dev/null && pwd || echo "")"
    IS_STANDALONE=1
    if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/core/package.json" ]; then
        IS_STANDALONE=0
    fi

    if [ "$IS_STANDALONE" = "1" ]; then
        RELEASE_URL='https://github.com/3mr-5aled/rtb/releases/latest/download/rtb-cli.js'
        start_spinner 'Downloading rtb-cli.js'
        if curl -fsSL --max-time 120 "$RELEASE_URL" -o "$LIB_DIR/rtb.js" 2>/dev/null || wget -q "$RELEASE_URL" -O "$LIB_DIR/rtb.js" 2>/dev/null; then
            curl -fsSL --max-time 15 "https://raw.githubusercontent.com/3mr-5aled/rtb/main/VERSION" -o "$RTB_DIR/VERSION" 2>/dev/null || true
            stop_spinner 1 'Downloaded RTB CLI engine'
        else
            stop_spinner 0 'Download RTB CLI engine'
            write_fail "Download failed from $RELEASE_URL. Check https://github.com/3mr-5aled/rtb/releases"
        fi
    else
        start_spinner 'Deploying local CLI bundle'
        if [ ! -f "$SCRIPT_DIR/core/dist/index.js" ]; then
            (cd "$SCRIPT_DIR/core" && npm install --silent && npm run build --silent) || {
                stop_spinner 0 'Build CLI bundle'
                write_fail "Failed to build core CLI with npm"
            }
        fi
        cp "$SCRIPT_DIR/core/dist/index.js" "$LIB_DIR/rtb.js"
        if [ -f "$SCRIPT_DIR/VERSION" ]; then
            cp "$SCRIPT_DIR/VERSION" "$RTB_DIR/VERSION"
            cp "$SCRIPT_DIR/VERSION" "$LIB_DIR/VERSION" 2>/dev/null || true
        fi
        stop_spinner 1 'Deployed local CLI bundle'
    fi

    # Create executable launcher script
    cat << 'EOF' > "$BIN_DIR/rtb"
#!/usr/bin/env sh
RTB_LIB_PATH="__LIB_DIR__/rtb.js"
exec node "$RTB_LIB_PATH" "$@"
EOF
    sed -i "s|__LIB_DIR__|$LIB_DIR|g" "$BIN_DIR/rtb" 2>/dev/null || sed -i '' "s|__LIB_DIR__|$LIB_DIR|g" "$BIN_DIR/rtb" 2>/dev/null || true
    chmod +x "$BIN_DIR/rtb"

    # Step 3: TUI Binary (Non-critical)
    write_step 3 $TOTAL 'Installing rtbtui binary'
    if [ "$IS_STANDALONE" = "1" ]; then
        BIN_URL="https://github.com/3mr-5aled/rtb/releases/latest/download/rtbtui-${OS_SLUG}-${ARCH_SLUG}"
        start_spinner "Downloading rtbtui ($OS_SLUG/$ARCH_SLUG)"
        if curl -fsSL --max-time 180 "$BIN_URL" -o "$BIN_DIR/rtbtui" 2>/dev/null || wget -q "$BIN_URL" -O "$BIN_DIR/rtbtui" 2>/dev/null; then
            chmod +x "$BIN_DIR/rtbtui"
            stop_spinner 1 'Installed rtbtui binary'
        elif [ "$OS_SLUG" = "macos" ] && [ "$ARCH_SLUG" = "arm64" ]; then
            FALLBACK_URL="https://github.com/3mr-5aled/rtb/releases/latest/download/rtbtui-macos-amd64"
            if curl -fsSL --max-time 180 "$FALLBACK_URL" -o "$BIN_DIR/rtbtui" 2>/dev/null; then
                chmod +x "$BIN_DIR/rtbtui"
                stop_spinner 1 'Installed rtbtui binary (x86_64 via Rosetta 2)'
            else
                stop_spinner 0 'Download rtbtui binary'
                write_warn "TUI binary download failed - 'rtb ui' unavailable, CLI is fine."
            fi
        else
            stop_spinner 0 'Download rtbtui binary'
            write_warn "TUI binary download failed - 'rtb ui' unavailable, CLI is fine."
        fi
    else
        TUI_DIR="$SCRIPT_DIR/tui"
        if command -v cargo >/dev/null 2>&1 && [ -f "$TUI_DIR/Cargo.toml" ]; then
            start_spinner 'Building rtbtui with Cargo'
            if (cd "$TUI_DIR" && cargo build --release >/dev/null 2>&1); then
                if [ -f "$TUI_DIR/target/release/rtbtui" ]; then
                    cp "$TUI_DIR/target/release/rtbtui" "$BIN_DIR/rtbtui"
                    chmod +x "$BIN_DIR/rtbtui"
                    stop_spinner 1 'Built and installed rtbtui binary'
                else
                    stop_spinner 0 'Build rtbtui'
                    write_warn 'Cargo build succeeded but binary not found in target/release.'
                fi
            else
                stop_spinner 0 'Build rtbtui'
                write_warn 'Cargo build failed - retaining existing binary if present.'
            fi
        elif [ -f "$TUI_DIR/target/release/rtbtui" ]; then
            cp "$TUI_DIR/target/release/rtbtui" "$BIN_DIR/rtbtui"
            chmod +x "$BIN_DIR/rtbtui"
            write_warn 'cargo not found - copied prebuilt binary.'
        else
            write_warn "cargo not found and no prebuilt binary - 'rtb ui' will not work."
        fi
    fi

    # Step 4: Shell Integration (PATH & shell-init hook)
    write_step 4 $TOTAL 'Configuring shell PATH & integration'
    start_spinner 'Updating shell profiles'

    EXPORT_LINE="export PATH=\"\$PATH:$BIN_DIR\""

    inject_rc() {
        rc_file="$1"
        shell_type="$2"
        [ -f "$rc_file" ] || return 0
        if ! grep -qF "$BIN_DIR" "$rc_file" 2>/dev/null; then
            printf '\n# RTB CLI\n%s\neval "$(rtb shell-init %s)"\n' "$EXPORT_LINE" "$shell_type" >> "$rc_file" 2>/dev/null || true
        fi
    }

    inject_rc "$HOME/.bashrc" "bash"
    inject_rc "$HOME/.bash_profile" "bash"
    inject_rc "$HOME/.zshrc" "zsh"
    inject_rc "$HOME/.profile" "bash"

    # Fish shell integration
    if [ -d "$HOME/.config/fish" ]; then
        FISH_CONF="$HOME/.config/fish/config.fish"
        if [ -f "$FISH_CONF" ] && ! grep -qF "$BIN_DIR" "$FISH_CONF" 2>/dev/null; then
            printf '\n# RTB CLI\nset -gx PATH $PATH %s\nrtb shell-init fish | source\n' "$BIN_DIR" >> "$FISH_CONF" 2>/dev/null || true
        fi
    fi

    export PATH="$PATH:$BIN_DIR"
    stop_spinner 1 'Shell configuration updated'
}

main() {
    show_header
    detect_os_arch
    ensure_node
    install_steps
    show_summary "$RTB_DIR"
}

main "$@"
