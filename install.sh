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

    if [ "$ANSI" = "1" ]; then
        printf '\r%60s\r\033[?25h' ''
        if [ "$success" = "1" ]; then
            printf '  \033[32m✅\033[0m  \033[32m%s\033[0m\n' "$label"
        else
            printf '  \033[31m❌\033[0m  \033[31m%s\033[0m\n' "$label"
        fi
    else
        if [ "$success" = "1" ]; then
            printf '  [OK] %s\n' "$label"
        else
            printf '  [FAILED] %s\n' "$label"
        fi
    fi
}

show_header() {
    if [ "$RTB_QUIET" = "1" ]; then
        printf 'RTB Setup Wizard\n'
        return
    fi
    c="$(esc '36m')"
    b="$(esc '1m')"
    r="$(esc '0m')"
    d="$(esc '90m')"
    printf '\n'
    printf '  %s%s██████╗ ████████╗██████╗ %s\n' "$b" "$c" "$r"
    printf '  %s%s██╔══██╗╚══██╔══╝██╔══██╗%s\n' "$b" "$c" "$r"
    printf '  %s%s██████╔╝   ██║   ██████╔╝%s\n' "$b" "$c" "$r"
    printf '  %s%s██╔══██╗   ██║   ██╔══██╗%s\n' "$b" "$c" "$r"
    printf '  %s%s██║  ██║   ██║   ██████╔╝%s\n' "$b" "$c" "$r"
    printf '  %s%s╚═╝  ╚═╝   ╚═╝   ╚═════╝ %s  Setup Wizard\n' "$b" "$c" "$r"
    printf '\n'
    printf '  %sRTB — Repository & Tooling Base%s\n' "$c" "$r"
    printf '  %sLinux / macOS installer%s\n\n' "$d" "$r"
}

prompt_input() {
    p_text="$1"
    p_default="$2"

    if [ "$RTB_QUIET" = "1" ] || [ "$IS_TTY" = "0" ]; then
        echo "$p_default"
        return
    fi

    printf '%s' "$p_text" >&2

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
            # Windows POSIX environment (Git Bash / MSYS)
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

ensure_pwsh() {
    if command -v pwsh >/dev/null 2>&1; then
        return 0
    fi

    write_warn "PowerShell (pwsh) was not found on your system. RTB requires pwsh."

    if [ "$RTB_QUIET" = "1" ] || [ "$IS_TTY" = "0" ]; then
        write_fail "PowerShell (pwsh) is required. Install from https://aka.ms/install-powershell"
    fi

    printf '  %s?%s Install PowerShell now? [Y/n] ' "$(esc '32m')" "$(esc '0m')" >&2
    if [ -t 0 ]; then
        read -r ans || ans="n"
    elif [ -r /dev/tty ]; then
        read -r ans < /dev/tty 2>/dev/null || ans="n"
    else
        ans="n"
    fi

    case "$ans" in
        [Nn]*) write_fail "PowerShell installation declined. Install from https://aka.ms/install-powershell and re-run installer." ;;
    esac

    start_spinner "Bootstrapping PowerShell (pwsh)"
    installed=0

    if [ "$OS_SLUG" = "macos" ] && command -v brew >/dev/null 2>&1; then
        brew install --cask powershell >/dev/null 2>&1 && installed=1
    elif command -v apt-get >/dev/null 2>&1; then
        UBUNTU_VER=""
        if [ -f /etc/os-release ]; then
            UBUNTU_VER="$(grep -E '^VERSION_ID=' /etc/os-release 2>/dev/null | cut -d= -f2 | tr -d '\"')"
        fi
        [ -z "$UBUNTU_VER" ] && UBUNTU_VER="22.04"
        DEB_URL="https://packages.microsoft.com/config/ubuntu/${UBUNTU_VER}/packages-microsoft-prod.deb"
        TMP_DEB="/tmp/ms-prod-$$.deb"
        if curl -fsSL "$DEB_URL" -o "$TMP_DEB" 2>/dev/null || wget -q "$DEB_URL" -O "$TMP_DEB" 2>/dev/null; then
            sudo dpkg -i "$TMP_DEB" >/dev/null 2>&1 && \
            sudo apt-get update -q >/dev/null 2>&1 && \
            sudo apt-get install -y powershell >/dev/null 2>&1 && installed=1
            rm -f "$TMP_DEB"
        fi
    elif command -v dnf >/dev/null 2>&1; then
        sudo dnf install -y powershell >/dev/null 2>&1 && installed=1
    elif command -v snap >/dev/null 2>&1; then
        sudo snap install powershell --classic >/dev/null 2>&1 && installed=1
    fi

    if [ "$installed" = "1" ] && command -v pwsh >/dev/null 2>&1; then
        stop_spinner 1 "PowerShell installed successfully"
    else
        stop_spinner 0 "PowerShell installation failed"
        write_fail "Could not automatically install pwsh. Please install manually from https://aka.ms/install-powershell"
    fi
}

show_summary() {
    ipath="$1"
    g="$(esc '32m')"
    b="$(esc '1m')"
    c="$(esc '36m')"
    d="$(esc '90m')"
    r="$(esc '0m')"

    printf '\n'
    printf '  %s%s✔ RTB installed successfully!%s\n\n' "$b" "$g" "$r"
    printf '  %sInstall path:%s  %s\n' "$c" "$r" "$ipath"
    printf '  %sPowerShell:%s    %s\n\n' "$c" "$r" "$(command -v pwsh || echo 'pwsh')"
    printf '  %sNext steps:%s\n' "$b" "$r"
    printf '    %srtb init%s  %s- configure your project workspace%s\n' "$g" "$r" "$d" "$r"
    printf '    %srtb help%s  %s- explore available commands%s\n' "$g" "$r" "$d" "$r"
    printf '    %srtb ui%s    %s- open the interactive terminal dashboard%s\n\n' "$g" "$r" "$d" "$r"
}

install_steps() {
    TOTAL=5
    XDG_CFG="${XDG_CONFIG_HOME:-$HOME/.config}"
    DEFAULT_DIR="$XDG_CFG/rtb"
    PROMPT_TEXT="  $(esc '32m')?$(esc '0m') Install location $(esc '90m')(Enter to accept)$(esc '0m')\n    $(esc '90m')$DEFAULT_DIR$(esc '0m')\n  › "

    RTB_DIR="${RTB_INSTALL_PATH:-$(prompt_input "$PROMPT_TEXT" "$DEFAULT_DIR")}"
    MODULE_HOME="$RTB_DIR/module"
    BIN_DIR="${RTB_BIN_DIR:-$RTB_DIR/bin}"

    # Step 1: Directories
    write_step 1 $TOTAL 'Creating directories'
    start_spinner 'Setting up install directories'
    mkdir -p "$RTB_DIR" "$MODULE_HOME" "$BIN_DIR" || {
        stop_spinner 0 'Setup install directories'
        write_fail "Cannot create install directories in $RTB_DIR"
    }
    stop_spinner 1 'Created install directories'

    # Step 2: Module Deployment
    write_step 2 $TOTAL 'Deploying RTB module'
    SCRIPT_DIR="$(cd "$(dirname "$0")" 2>/dev/null && pwd || echo "")"
    IS_STANDALONE=1
    if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/cli/rtb.psd1" ]; then
        IS_STANDALONE=0
    fi

    if [ "$IS_STANDALONE" = "1" ]; then
        ZIP_URL='https://github.com/3mr-5aled/rtb/releases/latest/download/rtb-cli.zip'
        TMP_ZIP="/tmp/rtb-$$.zip"
        TMP_EXT="/tmp/rtb-ext-$$"
        start_spinner 'Downloading rtb-cli.zip'
        if curl -fsSL --max-time 120 "$ZIP_URL" -o "$TMP_ZIP" 2>/dev/null || wget -q "$ZIP_URL" -O "$TMP_ZIP" 2>/dev/null; then
            stop_spinner 1 'Downloaded rtb-cli.zip'
            start_spinner 'Extracting module files'
            mkdir -p "$TMP_EXT"
            if command -v unzip >/dev/null 2>&1; then
                unzip -q -o "$TMP_ZIP" -d "$TMP_EXT"
            else
                pwsh -NoProfile -NonInteractive -Command "Expand-Archive -Path '$TMP_ZIP' -DestinationPath '$TMP_EXT' -Force"
            fi
            if [ -d "$TMP_EXT/cli" ]; then
                cp -r "$TMP_EXT/cli/." "$MODULE_HOME/"
            fi
            if [ -f "$TMP_EXT/logo.txt" ]; then
                cp "$TMP_EXT/logo.txt" "$BIN_DIR/logo.txt"
            fi
            if [ -f "$TMP_EXT/uninstall.ps1" ]; then
                cp "$TMP_EXT/uninstall.ps1" "$BIN_DIR/uninstall.ps1"
                cp "$TMP_EXT/uninstall.ps1" "$RTB_DIR/uninstall.ps1"
            fi
            rm -f "$TMP_ZIP"
            rm -rf "$TMP_EXT"
            stop_spinner 1 'Extracted module files'
        else
            stop_spinner 0 'Download rtb-cli.zip'
            rm -f "$TMP_ZIP"
            write_fail "Download failed from $ZIP_URL. Check https://github.com/3mr-5aled/rtb/releases"
        fi
    else
        start_spinner 'Copying local CLI module'
        if [ -d "$SCRIPT_DIR/cli" ]; then
            cp -r "$SCRIPT_DIR/cli/." "$MODULE_HOME/"
            if [ -f "$SCRIPT_DIR/logo.txt" ]; then
                cp "$SCRIPT_DIR/logo.txt" "$BIN_DIR/logo.txt"
            fi
            if [ -f "$SCRIPT_DIR/uninstall.ps1" ]; then
                cp "$SCRIPT_DIR/uninstall.ps1" "$BIN_DIR/uninstall.ps1"
                cp "$SCRIPT_DIR/uninstall.ps1" "$RTB_DIR/uninstall.ps1"
            fi
            stop_spinner 1 'Copied local CLI module'
        else
            stop_spinner 0 'Copy local CLI module'
            write_fail "cli/ directory not found in $SCRIPT_DIR"
        fi
    fi

    # Step 3: TUI Binary (Non-critical)
    write_step 3 $TOTAL 'Installing rtbtui binary'
    if [ "$IS_STANDALONE" = "1" ]; then
        BIN_URL="https://github.com/3mr-5aled/rtb/releases/latest/download/rtbtui-${OS_SLUG}-${ARCH_SLUG}"
        start_spinner "Downloading rtbtui ($OS_SLUG/$ARCH_SLUG)"
        if curl -fsSL --max-time 180 "$BIN_URL" -o "$BIN_DIR/rtbtui" 2>/dev/null || wget -q "$BIN_URL" -O "$BIN_DIR/rtbtui" 2>/dev/null; then
            chmod +x "$BIN_DIR/rtbtui"
            cp "$BIN_DIR/rtbtui" "$BIN_DIR/devtui" 2>/dev/null || true
            stop_spinner 1 'Installed rtbtui binary'
        elif [ "$OS_SLUG" = "macos" ] && [ "$ARCH_SLUG" = "arm64" ]; then
            FALLBACK_URL="https://github.com/3mr-5aled/rtb/releases/latest/download/rtbtui-macos-amd64"
            if curl -fsSL --max-time 180 "$FALLBACK_URL" -o "$BIN_DIR/rtbtui" 2>/dev/null; then
                chmod +x "$BIN_DIR/rtbtui"
                cp "$BIN_DIR/rtbtui" "$BIN_DIR/devtui" 2>/dev/null || true
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
                    cp "$BIN_DIR/rtbtui" "$BIN_DIR/devtui" 2>/dev/null || true
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
            cp "$BIN_DIR/rtbtui" "$BIN_DIR/devtui" 2>/dev/null || true
            write_warn 'cargo not found - copied prebuilt binary.'
        else
            write_warn "cargo not found and no prebuilt binary - 'rtb ui' will not work."
        fi
    fi

    # Step 4: Shell Integration (PATH & Launcher Script)
    write_step 4 $TOTAL 'Configuring shell PATH & wrapper'
    start_spinner 'Updating shell environment'

    # Create standalone executable wrapper script
    cat << 'EOF' > "$BIN_DIR/rtb"
#!/usr/bin/env sh
RTB_PSD1_PATH="__MODULE_HOME__/rtb.psd1"
exec pwsh -NoProfile -NonInteractive -Command "& { param(\$args) Import-Module '$RTB_PSD1_PATH' -DisableNameChecking; & rtb @args }" "$@"
EOF
    sed -i "s|__MODULE_HOME__|$MODULE_HOME|g" "$BIN_DIR/rtb" 2>/dev/null || sed -i '' "s|__MODULE_HOME__|$MODULE_HOME|g" "$BIN_DIR/rtb" 2>/dev/null || true
    chmod +x "$BIN_DIR/rtb"

    EXPORT_LINE="export PATH=\"\$PATH:$BIN_DIR\""
    ALIAS_LINE="alias rtb='pwsh -NoProfile -NonInteractive -Command \"Import-Module '\''$MODULE_HOME/rtb.psd1'\'' -DisableNameChecking; rtb\"'"

    inject_rc() {
        rc_file="$1"
        [ -f "$rc_file" ] || return 0
        if ! grep -qF "$BIN_DIR" "$rc_file" 2>/dev/null; then
            printf '\n# RTB CLI\n%s\n%s\n' "$EXPORT_LINE" "$ALIAS_LINE" >> "$rc_file" 2>/dev/null || true
        fi
    }

    inject_rc "$HOME/.bashrc"
    inject_rc "$HOME/.bash_profile"
    inject_rc "$HOME/.zshrc"
    inject_rc "$HOME/.profile"

    export PATH="$PATH:$BIN_DIR"
    stop_spinner 1 'Shell configuration updated'

    # Step 5: PowerShell Profile Configuration
    write_step 5 $TOTAL 'Configuring PowerShell profile'
    PWSH_PROFILE_DIR="${XDG_CFG}/powershell"
    PWSH_PROFILE="$PWSH_PROFILE_DIR/Microsoft.PowerShell_profile.ps1"
    MODULE_PSD="$MODULE_HOME/rtb.psd1"
    if [ -f "$MODULE_PSD" ]; then
        start_spinner 'Configuring pwsh profile'
        mkdir -p "$PWSH_PROFILE_DIR"
        touch "$PWSH_PROFILE"
        TMP_PROF="/tmp/pwsh-prof-$$"
        if [ -f "$PWSH_PROFILE" ]; then
            grep -v -E "Import-Module.*(rtb|dev-tools|dev-cli).*rtb\.psd1|#\s*RTB.*?Module" "$PWSH_PROFILE" > "$TMP_PROF" 2>/dev/null || cp "$PWSH_PROFILE" "$TMP_PROF"
            mv "$TMP_PROF" "$PWSH_PROFILE"
        fi
        printf '\n# RTB CLI Module\nImport-Module '\''%s'\'' -DisableNameChecking -Force\n' "$MODULE_PSD" >> "$PWSH_PROFILE" 2>/dev/null || true
        stop_spinner 1 'PowerShell profile configured'
    else
        write_warn "rtb.psd1 not found in $MODULE_HOME - skipping profile injection."
    fi
}

main() {
    show_header
    detect_os_arch
    ensure_pwsh
    install_steps
    show_summary "$RTB_DIR"

    if [ "$RTB_QUIET" != "1" ] && [ "$IS_TTY" = "1" ]; then
        printf '\n  %s?%s Run %s'\''rtb init'\''%s now? [Y/n] ' "$(esc '32m')" "$(esc '0m')" "$(esc '1m')" "$(esc '0m')" >&2
        if [ -t 0 ]; then
            read -r init_ans || init_ans="n"
        elif [ -r /dev/tty ]; then
            read -r init_ans < /dev/tty 2>/dev/null || init_ans="n"
        else
            init_ans="n"
        fi
        case "$init_ans" in
            [Nn]*) ;;
            *)
                if command -v pwsh >/dev/null 2>&1 && [ -f "$MODULE_PSD" ]; then
                    pwsh -NoProfile -NonInteractive -Command "Import-Module '$MODULE_PSD' -DisableNameChecking -Force; rtb init" || true
                fi
                ;;
        esac
    fi
}

main "$@"
