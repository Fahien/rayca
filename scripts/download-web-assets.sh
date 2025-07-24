#!/bin/sh
# Downloads web assets for Rayca web

set -ex

site="https://antoniocaggiano.eu"

stylesheets=(
  "main.css"
  "fonts.css"
  "theme/dark.css"
  "css/override.css"
)

icons=(
  "android-icon-144x144.png"
  "android-icon-192x192.png"
  "android-icon-48x48.png"
  "apple-icon-152x152.png"
  "android-icon-72x72.png"
  "android-icon-36x36.png"
  "apple-icon-180x180.png"
  "apple-icon-57x57.png"
  "apple-icon-60x60.png"
  "apple-icon-72x72.png"
  "android-icon-96x96.png"
  "apple-icon-76x76.png"
  "apple-icon-114x114.png"
  "apple-icon-precomposed.png"
  "apple-icon-120x120.png"
  "apple-icon-144x144.png"
  "apple-icon.png"
  "browserconfig.xml"
  "favicon.png"
  "favicon-16x16.png"
  "favicon-32x32.png"
  "favicon-96x96.png"
  "manifest.json"
  "mc-white.svg"
  "ms-icon-150x150.png"
  "ms-icon-310x310.png"
  "ms-icon-70x70.png"
  "ms-icon-144x144.png"
)

jetbrains_fonts=(
  "JetBrainsMono-Bold.ttf"
  "JetBrainsMono-BoldItalic.ttf"
  "JetBrainsMono-ExtraBold.ttf"
  "JetBrainsMono-ExtraBoldItalic.ttf"
  "JetBrainsMono-ExtraLight.ttf"
  "JetBrainsMono-ExtraLightItalic.ttf"
  "JetBrainsMono-Italic.ttf"
  "JetBrainsMono-Light.ttf"
  "JetBrainsMono-LightItalic.ttf"
  "JetBrainsMono-Medium.ttf"
  "JetBrainsMono-MediumItalic.ttf"
  "JetBrainsMono-Regular.ttf"
  "JetBrainsMono-SemiBold.ttf"
  "JetBrainsMono-SemiBoldItalic.ttf"
  "JetBrainsMono-Thin.ttf"
  "JetBrainsMono-ThinItalic.ttf"
)

spacegrotesk_fonts=(
  "SpaceGrotesk-Bold.ttf"
  "SpaceGrotesk-Light.ttf"
  "SpaceGrotesk-Medium.ttf"
  "SpaceGrotesk-Regular.ttf"
  "SpaceGrotesk-SemiBold.ttf"
)

# Create directories if they do not exist
static_dir="./rayca-soft/www/static"

mkdir -p "$static_dir/css"
mkdir -p "$static_dir/theme"
mkdir -p "$static_dir/icons"
mkdir -p "$static_dir/fonts/JetbrainsMono"
mkdir -p "$static_dir/fonts/SpaceGrotesk"

# Use curl to download the stylesheets
for stylesheet in "${stylesheets[@]}"; do
  echo "Downloading $stylesheet..."
  curl -s -L -o "$static_dir/$stylesheet" "$site/$stylesheet"
done

# Download icons
for icon in "${icons[@]}"; do
  echo "Downloading $icon..."
  curl -s -L -o "$static_dir/icons/$icon" "$site/icons/$icon"
done

# Download JetBrains Mono fonts
for font in "${jetbrains_fonts[@]}"; do
  echo "Downloading JetBrains Mono font: $font..."
  curl -s -L -o "$static_dir/fonts/JetbrainsMono/$font" "$site/fonts/JetbrainsMono/$font"
done

# Download Space Grotesk fonts
for font in "${spacegrotesk_fonts[@]}"; do
  echo "Downloading Space Grotesk font: $font..."
  curl -s -L -o "$static_dir/fonts/SpaceGrotesk/$font" "$site/fonts/SpaceGrotesk/$font"
done
