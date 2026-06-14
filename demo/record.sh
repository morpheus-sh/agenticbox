#!/usr/bin/env bash
# AgenticBox Demo — Recording Pipeline
#
# Records the demo by capturing the screen via ffmpeg gdigrab
# while the demo script plays in a terminal window.
#
# USAGE:
#   bash demo/record.sh              # record full desktop → MP4
#   bash demo/record.sh --gif        # also convert to GIF
#   bash demo/record.sh --region W,H,X,Y  # record specific region

set -euo pipefail

DEMO_DIR="$(cd "$(dirname "$0")" && pwd)"
MP4_FILE="$DEMO_DIR/agenticbox-demo.mp4"
GIF_FILE="$DEMO_DIR/agenticbox-demo.gif"

# ─── ffmpeg detection ──────────────────────────────────────────
# Try: local PATH → Windows ffmpeg via winget path → auto-install
FFMPEG=""

if command -v ffmpeg &>/dev/null; then
    FFMPEG="ffmpeg"
else
    # Search common Windows install locations
    WIN_FFMPEG_DIRS=(
        "/c/Users/$USER/AppData/Local/Microsoft/WinGet/Packages/Gyan.FFmpeg_Microsoft.Winget.Source_8wekyb3d8bbwe/ffmpeg-8.1.1-full_build/bin"
        "/c/Users/$USER/AppData/Local/Microsoft/WinGet/Packages/Gyan.FFmpeg_Microsoft.Winget.Source_8wekyb3d8bbwe/ffmpeg"-*/bin
        "/c/ffmpeg/bin"
        "/c/Program Files/ffmpeg/bin"
    )
    for dir in "${WIN_FFMPEG_DIRS[@]}"; do
        # shellcheck disable=SC2086
        for match in $dir; do
            if [[ -x "$match/ffmpeg.exe" ]]; then
                FFMPEG="$match/ffmpeg.exe"
                break 2
            fi
        done
    done
fi

if [[ -z "$FFMPEG" ]]; then
    echo ">>> ffmpeg not found. Attempting install..."
    if command -v winget &>/dev/null; then
        winget install --id Gyan.FFmpeg -e --accept-source-agreements --accept-package-agreements 2>/dev/null && {
            # Re-scan after install
            for dir in /c/Users/"$USER"/AppData/Local/Microsoft/WinGet/Packages/Gyan.FFmpeg*/ffmpeg-*/bin; do
                if [[ -x "$dir/ffmpeg.exe" ]]; then
                    FFMPEG="$dir/ffmpeg.exe"
                    break
                fi
            done
        }
    elif command -v apt-get &>/dev/null; then
        sudo apt-get update -qq && sudo apt-get install -y -qq ffmpeg
        FFMPEG="ffmpeg"
    elif command -v brew &>/dev/null; then
        brew install ffmpeg
        FFMPEG="ffmpeg"
    fi
fi

if [[ -z "$FFMPEG" ]]; then
    echo ""
    echo "✗ Could not find or install ffmpeg."
    echo "  Install manually:"
    echo "    Windows: winget install Gyan.FFmpeg"
    echo "    Linux:   sudo apt install ffmpeg"
    echo "    macOS:   brew install ffmpeg"
    exit 1
fi

echo ">>> Using ffmpeg: $FFMPEG"
"$FFMPEG" -version | head -1
echo ""

# ─── Parse args ─────────────────────────────────────────────────
RECORD_GIF=false
REGION_ARGS=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --gif)   RECORD_GIF=true; shift ;;
        --region) REGION_ARGS="-video_size $2 -offset_x $3 -offset_y $4"; shift 4 ;;
        *)       shift ;;
    esac
done

# ─── Record ─────────────────────────────────────────────────────
echo "============================================"
echo "  AgenticBox Demo Recorder"
echo "============================================"
echo ""
echo "This will:"
echo "  1. Start ffmpeg screen capture"
echo "  2. Play the demo script (visible on screen)"
echo "  3. Stop recording after ~35 seconds"
echo ""
echo "MAKE SURE your terminal is positioned and sized correctly."
echo "Press ENTER to start, Ctrl+C to cancel..."
read -r

# Start ffmpeg in background — capture desktop or region
if [[ -n "$REGION_ARGS" ]]; then
    echo ">>> Recording region..."
    "$FFMPEG" -y -f gdigrab $REGION_ARGS -framerate 30 -t 35 \
        -c:v libx264 -preset fast -crf 18 -pix_fmt yuv420p \
        "$MP4_FILE" &
else
    echo ">>> Recording full desktop..."
    "$FFMPEG" -y -f gdigrab -i desktop -framerate 30 -t 35 \
        -c:v libx264 -preset fast -crf 18 -pix_fmt yuv420p \
        "$MP4_FILE" &
fi

FFPID=$!
echo ">>> ffmpeg PID: $FFPID"

# Wait for ffmpeg to initialize
sleep 2

# Run the demo
echo ">>> Playing demo..."
bash "$DEMO_DIR/agent_demo.sh"

# Wait a moment for the final frame
sleep 3

# Stop ffmpeg
echo ">>> Stopping recording..."
kill $FFPID 2>/dev/null || true
wait $FFPID 2>/dev/null || true

echo ""
echo ">>> MP4 saved: $MP4_FILE"
ls -lh "$MP4_FILE" 2>/dev/null

# Convert to GIF if requested
if $RECORD_GIF; then
    echo ""
    echo ">>> Converting to GIF..."

    # Generate palette for quality
    PALETTE=$(mktemp --suffix=.png 2>/dev/null || mktemp)
    "$FFMPEG" -y -i "$MP4_FILE" -vf "fps=15,scale=800:-1:flags=lanczos,palettegen" "$PALETTE" 2>/dev/null
    "$FFMPEG" -y -i "$MP4_FILE" -i "$PALETTE" \
        -lavfi "fps=15,scale=800:-1:flags=lanczos [x]; [x][1:v] paletteuse" \
        "$GIF_FILE" 2>/dev/null
    rm -f "$PALETTE"

    echo ">>> GIF saved: $GIF_FILE"
    ls -lh "$GIF_FILE" 2>/dev/null
fi

echo ""
echo "Done! Files in $DEMO_DIR/"
