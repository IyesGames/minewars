#!/bin/bash

RELDIR="release"

EXE_MAIN="minewars"
EXE_EXTRA="mw_host mw_auth mw_hostrpc mw_cert mw_datatool"

MYOS="$(uname)"

export CARGO_TARGET_DIR="./target-mwrel"

macos() {
    APPNAME="MineWars"

    if [ "$MYOS" != "Darwin" ]; then
        echo "MacOS builds can only be made on MacOS! Run this script on MacOS!"
        exit 1
    fi

    RELDIR_MAC="${RELDIR}/macos"
    mkdir -p "${RELDIR_MAC}/${APPNAME}.app/Contents/MacOS" || exit 2
    mkdir -p "${RELDIR_MAC}/${APPNAME}.app/Contents/Resources" || exit 2

    cargo build --workspace --target aarch64-apple-darwin --release || exit 2
    cargo build --workspace --target x86_64-apple-darwin --release || exit 2

    lipo \
      "${CARGO_TARGET_DIR}/aarch64-apple-darwin/release/${EXE_MAIN}" \
      "${CARGO_TARGET_DIR}/x86_64-apple-darwin/release/${EXE_MAIN}" \
      -create -output "${RELDIR_MAC}/${APPNAME}.app/Contents/MacOS/${APPNAME}" || exit 2

    for exe in $EXE_EXTRA; do
        lipo \
          "${CARGO_TARGET_DIR}/aarch64-apple-darwin/release/${exe}" \
          "${CARGO_TARGET_DIR}/x86_64-apple-darwin/release/${exe}" \
          -create -output "${RELDIR_MAC}/${APPNAME}.app/Contents/MacOS/${exe}" || exit 2
    done

    cp -R "assets" "${RELDIR_MAC}/${APPNAME}.app/Contents/MacOS/" || exit 2

    cp "dist/AppIcon.icns" "${RELDIR_MAC}/${APPNAME}.app/Resources/AppIcon.icns" || exit 2
    cp "dist/Info.plist" "${RELDIR_MAC}/${APPNAME}.app/Info.plist" || exit 2

    create-dmg \
      --volname "${APPNAME}" \
      --volicon "dist/AppIcon.icns" \
      --background "dist/DMG-background.png" \
      --window-size 800 400 \
      --icon-size 128 \
      --icon "${APPNAME}.app" 200 200 \
      --hide-extension "${APPNAME}.app" \
      --app-drop-link 600 200 \
      "${RELDIR}/${APPNAME}.dmg" \
      "${RELDIR_MAC}" || exit 2
}

windows() {
    cargo build --workspace --target "${TARGET}" --release || exit 2
    for exe in ${EXE_MAIN} ${EXE_EXTRA}; do
        cp "${CARGO_TARGET_DIR}/${TARGET}/release/${exe}.exe" "${RELDIR_WIN}/" || exit 2
    done

    cp -R "assets" "${RELDIR_WIN}/" || exit 2
}

linux() {
    if [ "$MYOS" != "Linux" ]; then
        echo "Linux builds can only be made on Linux! Run this script on Linux!"
        exit 1
    fi

    cargo build --workspace --target "${TARGET}" --release || exit 2
    for exe in ${EXE_MAIN} ${EXE_EXTRA}; do
        cp "${CARGO_TARGET_DIR}/${TARGET}/release/${exe}" "${RELDIR_LIN}/" || exit 2
    done

    cp -R "assets" "${RELDIR_LIN}/" || exit 2
}

win64() {
    RELDIR_WIN="${RELDIR}/win64"
    TARGET="x86_64-pc-windows-msvc"
    windows
}

win32() {
    RELDIR_WIN="${RELDIR}/win32"
    TARGET="i686-pc-windows-msvc"
    windows
}

linux-x86() {
    RELDIR_LIN="${RELDIR}/linux-x86"
    TARGET="i686-unknown-linux-gnu"
    linux
}

linux-x64() {
    RELDIR_LIN="${RELDIR}/linux-x64"
    TARGET="x86_64-unknown-linux-gnu"
    linux
}

linux-arm() {
    RELDIR_LIN="${RELDIR}/linux-arm"
    TARGET="armv7-unknown-linux-gnueabihf"
    linux
}

linux-arm64() {
    RELDIR_LIN="${RELDIR}/linux-arm64"
    TARGET="aarch64-unknown-linux-gnu"
    linux
}

case "${1}" in
    macos)
        macos
        ;;
    win32)
        win32
        ;;
    win64)
        win64
        ;;
    linux-x86)
        linux-x86
        ;;
    linux-x64)
        linux-x64
        ;;
    linux-arm)
        linux-arm
        ;;
    linux-arm64)
        linux-arm64
        ;;
esac
