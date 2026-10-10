"""
ARBELUNE Brand Asset Generator & Packaging Script
Reproducibly generates the complete ARBELUNE brand asset package from source assets:
- Primary wordmarks (transparent, dark, light)
- Standalone geometric 'A' mark (SVG, PNG, Windows multi-resolution .ico)
- Application squircle icons
- GitHub social preview banner & avatar
"""

import os
import shutil
import math
from PIL import Image, ImageDraw, ImageFilter

ROOT_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
TEMP_DIR = os.path.join(ROOT_DIR, "temp")
PUBLIC_DIR = os.path.join(ROOT_DIR, "public")
SRC_UI_PUBLIC_DIR = os.path.join(ROOT_DIR, "src-ui", "public")
ICONS_DIR = os.path.join(ROOT_DIR, "src-tauri", "icons")

os.makedirs(PUBLIC_DIR, exist_ok=True)
os.makedirs(SRC_UI_PUBLIC_DIR, exist_ok=True)
os.makedirs(ICONS_DIR, exist_ok=True)

print("=== ARBELUNE Brand Package Generator ===")

# 1. Primary Wordmark & Avatar from source
src_wordmark = os.path.join(TEMP_DIR, "arbelune.png")
src_avatar = os.path.join(TEMP_DIR, "avatar.png")

if os.path.exists(src_wordmark):
    dest_logo = os.path.join(PUBLIC_DIR, "arbelune-logo.png")
    shutil.copy2(src_wordmark, dest_logo)
    print(f"  [OK] Copied master wordmark -> {dest_logo}")

if os.path.exists(src_avatar):
    dest_avatar = os.path.join(PUBLIC_DIR, "arbelune-avatar.png")
    shutil.copy2(src_avatar, dest_avatar)
    print(f"  [OK] Copied master avatar -> {dest_avatar}")

# 2. Vector SVG Geometric 'A' Mark
svg_markup = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <linearGradient id="arbeluneChevron" x1="0%" y1="100%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="#FF106E" />
      <stop offset="30%" stop-color="#FF2F88" />
      <stop offset="65%" stop-color="#FF80B2" />
      <stop offset="90%" stop-color="#FFE4EF" />
      <stop offset="100%" stop-color="#FFFFFF" />
    </linearGradient>
    <linearGradient id="arbeluneDot" x1="0%" y1="100%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="#FF2F88" />
      <stop offset="100%" stop-color="#FFA8CE" />
    </linearGradient>
    <filter id="neonGlow" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="8" result="blur" />
      <feMerge>
        <feMergeNode in="blur" />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
  </defs>
  <!-- Geometric Chevron A Mark -->
  <path d="M 256,76 L 436,424 L 358,424 L 256,220 L 154,424 L 76,424 Z" fill="url(#arbeluneChevron)" />
  <!-- Central Floating Dot -->
  <circle cx="256" cy="336" r="33" fill="url(#arbeluneDot)" />
</svg>'''

svg_dest_1 = os.path.join(PUBLIC_DIR, "arbelune-icon.svg")
svg_dest_2 = os.path.join(SRC_UI_PUBLIC_DIR, "favicon.svg")
with open(svg_dest_1, "w", encoding="utf-8") as f:
    f.write(svg_markup)
with open(svg_dest_2, "w", encoding="utf-8") as f:
    f.write(svg_markup)
print(f"  [OK] Generated vector SVG -> {svg_dest_1}")

# 3. Supersampled Raster Generation of Geometric 'A' Mark (1024x1024 supersampled -> 512x512)
def render_geometric_a(size=512, background=None, round_rect=False, padding=0.18):
    scale = 4  # 4x supersampling
    s = size * scale
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    if round_rect and background:
        # Draw rounded squircle
        radius = int(s * 0.22)
        draw.rounded_rectangle([0, 0, s, s], radius=radius, fill=background)
        # Subtle glowing rim
        draw.rounded_rectangle([int(s*0.01), int(s*0.01), int(s*0.99), int(s*0.99)], radius=radius, outline=(255, 42, 133, 70), width=int(scale * 3))

    # Calculate coordinates
    pad = int(s * padding)
    usable_h = s - (pad * 2)
    top_y = pad
    bot_y = s - pad
    mid_x = s // 2

    width = usable_h * 1.05
    half_w = width / 2
    outer_left_x = mid_x - half_w
    outer_right_x = mid_x + half_w

    leg_thickness = width * 0.22
    inner_left_x = outer_left_x + leg_thickness
    inner_right_x = outer_right_x - leg_thickness
    inner_apex_y = top_y + (usable_h * 0.42)

    poly = [
        (mid_x, top_y),
        (outer_right_x, bot_y),
        (inner_right_x, bot_y),
        (mid_x, inner_apex_y),
        (inner_left_x, bot_y),
        (outer_left_x, bot_y)
    ]

    # Create gradient mask
    mask = Image.new("L", (s, s), 0)
    draw_mask = ImageDraw.Draw(mask)
    draw_mask.polygon(poly, fill=255)

    dot_r = usable_h * 0.098
    dot_cy = top_y + (usable_h * 0.745)
    dot_box = [mid_x - dot_r, dot_cy - dot_r, mid_x + dot_r, dot_cy + dot_r]
    draw_mask.ellipse(dot_box, fill=255)

    # Render gradient: bottom-left (#FF106E) to top-right (#FFFFFF)
    grad = Image.new("RGBA", (s, s))
    grad_data = grad.load()
    c_start = (255, 16, 110)
    c_mid = (255, 90, 160)
    c_end = (255, 255, 255)

    for y in range(s):
        for x in range(s):
            # Normalized diagonal parameter t from 0 (bottom-left) to 1 (top-right)
            t = ((x / s) + (1.0 - (y / s))) / 2.0
            t = max(0.0, min(1.0, t))
            if t < 0.5:
                sub_t = t * 2.0
                r = int(c_start[0] + sub_t * (c_mid[0] - c_start[0]))
                g = int(c_start[1] + sub_t * (c_mid[1] - c_start[1]))
                b = int(c_start[2] + sub_t * (c_mid[2] - c_start[2]))
            else:
                sub_t = (t - 0.5) * 2.0
                r = int(c_mid[0] + sub_t * (c_end[0] - c_mid[0]))
                g = int(c_mid[1] + sub_t * (c_end[1] - c_mid[1]))
                b = int(c_mid[2] + sub_t * (c_end[2] - c_mid[2]))
            grad_data[x, y] = (r, g, b, 255)

    # Composite gradient with mask
    rendered_shape = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    rendered_shape.paste(grad, (0, 0), mask)

    if background:
        final_img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
        if round_rect:
            # apply squircle mask
            bg_mask = Image.new("L", (s, s), 0)
            draw_bg = ImageDraw.Draw(bg_mask)
            radius = int(s * 0.22)
            draw_bg.rounded_rectangle([0, 0, s, s], radius=radius, fill=255)

            bg_layer = Image.new("RGBA", (s, s), background)
            final_img.paste(bg_layer, (0, 0), bg_mask)

            # Draw glowing rim
            draw_final = ImageDraw.Draw(final_img)
            draw_final.rounded_rectangle([int(s*0.015), int(s*0.015), int(s*0.985), int(s*0.985)], radius=radius, outline=(255, 42, 133, 100), width=int(scale * 3))
        else:
            final_img = Image.new("RGBA", (s, s), background)
        final_img.paste(rendered_shape, (0, 0), rendered_shape)
    else:
        final_img = rendered_shape

    return final_img.resize((size, size), Image.Resampling.LANCZOS)

# Generate transparent standalone icon (512x512)
icon_512 = render_geometric_a(size=512, background=None, round_rect=False, padding=0.15)
icon_512.save(os.path.join(PUBLIC_DIR, "arbelune-icon.png"))
print("  [OK] Generated transparent icon -> public/arbelune-icon.png")

# Generate obsidian app squircle icon (512x512)
app_icon_512 = render_geometric_a(size=512, background=(10, 6, 8, 255), round_rect=True, padding=0.22)
app_icon_512.save(os.path.join(PUBLIC_DIR, "arbelune-app-icon.png"))
print("  [OK] Generated app squircle icon -> public/arbelune-app-icon.png")

# Generate Windows multi-resolution icon.ico
ico_sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)]
ico_images = [render_geometric_a(size=sz[0], background=(10, 6, 8, 255), round_rect=True, padding=0.18) for sz in ico_sizes]

dest_ico_1 = os.path.join(ICONS_DIR, "icon.ico")
dest_ico_2 = os.path.join(PUBLIC_DIR, "favicon.ico")
dest_ico_3 = os.path.join(SRC_UI_PUBLIC_DIR, "favicon.ico")

ico_images[0].save(dest_ico_1, format="ICO", sizes=ico_sizes)
ico_images[0].save(dest_ico_2, format="ICO", sizes=ico_sizes)
ico_images[0].save(dest_ico_3, format="ICO", sizes=ico_sizes)
print(f"  [OK] Generated multi-resolution Windows ICO -> {dest_ico_1} (16 to 256px)")

# Save PNG favicons
fav_32 = render_geometric_a(size=32, background=None, round_rect=False, padding=0.08)
fav_32.save(os.path.join(SRC_UI_PUBLIC_DIR, "favicon.png"))
fav_192 = render_geometric_a(size=192, background=(10, 6, 8, 255), round_rect=True, padding=0.18)
fav_192.save(os.path.join(SRC_UI_PUBLIC_DIR, "logo192.png"))
print("  [OK] Generated web favicons -> src-ui/public/favicon.png, logo192.png")

# 4. Wordmark Background Variants (Dark and Light)
if os.path.exists(src_wordmark):
    wm = Image.open(src_wordmark).convert("RGBA")
    
    # Dark variant (Obsidian #090508)
    wm_dark = Image.new("RGBA", wm.size, (9, 5, 8, 255))
    wm_dark.paste(wm, (0, 0), wm)
    wm_dark.save(os.path.join(PUBLIC_DIR, "arbelune-wordmark-dark.png"))
    print("  [OK] Generated dark wordmark -> public/arbelune-wordmark-dark.png")

    # Light variant (Clean White #FFFFFF)
    wm_light = Image.new("RGBA", wm.size, (255, 255, 255, 255))
    wm_light.paste(wm, (0, 0), wm)
    wm_light.save(os.path.join(PUBLIC_DIR, "arbelune-wordmark-light.png"))
    print("  [OK] Generated light wordmark -> public/arbelune-wordmark-light.png")

# 5. GitHub Social Preview Banner (1280x640)
banner = Image.new("RGBA", (1280, 640), (8, 4, 7, 255))
draw_b = ImageDraw.Draw(banner)

# Radial neon-pink ambient background glow
glow = Image.new("RGBA", (1280, 640), (0, 0, 0, 0))
draw_glow = ImageDraw.Draw(glow)
draw_glow.ellipse([440, 100, 840, 500], fill=(255, 20, 110, 38))
draw_glow.ellipse([200, 150, 1080, 480], fill=(180, 10, 70, 22))
glow = glow.filter(ImageFilter.GaussianBlur(50))
banner.paste(glow, (0, 0), glow)

# Place Geometric 'A' Mark (large, centered at top/left)
banner_icon = render_geometric_a(size=140, background=None, round_rect=False, padding=0.08)
banner.paste(banner_icon, (640 - 70, 120), banner_icon)

# Resize & paste wordmark (centered in lower half)
if os.path.exists(src_wordmark):
    wm_scaled = wm.resize((960, 320), Image.Resampling.LANCZOS)
    banner.paste(wm_scaled, ((1280 - 960) // 2, 260), wm_scaled)

# Subtle border
draw_b.rectangle([0, 0, 1279, 639], outline=(255, 42, 133, 40), width=1)
banner.save(os.path.join(PUBLIC_DIR, "arbelune-banner.png"))
print("  [OK] Generated GitHub social preview banner -> public/arbelune-banner.png (1280x640)")

print("\n=== Brand Package Generation Complete ===")
